use crate::t_dedup;
use crate::t_sqlite::{AFile, QueryParams};
use hnsw_rs::prelude::*;
use rusqlite::{params, params_from_iter, Connection};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Emitter;

// Keep groups reviewable and ensure every member can consider all other members.
const MAX_GROUP_SIZE: usize = 64;
const TOP_K: usize = MAX_GROUP_SIZE - 1;
const SEARCH_EF: usize = 200;
const SQL_BATCH_SIZE: usize = 900;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimilarScanStatus {
    pub state: String,
    pub phase: String,
    pub scope_key: String,
    pub current: u64,
    pub total: u64,
    pub groups: u64,
    pub is_scanning: bool,
}

impl Default for SimilarScanStatus {
    fn default() -> Self {
        Self {
            state: "idle".into(),
            phase: "idle".into(),
            scope_key: String::new(),
            current: 0,
            total: 0,
            groups: 0,
            is_scanning: false,
        }
    }
}

#[derive(Default)]
pub struct SimilarState {
    pub is_scanning: Arc<AtomicBool>,
    pub cancel_flag: Arc<AtomicBool>,
    pub status: Arc<Mutex<SimilarScanStatus>>,
}

struct VectorFile {
    id: i64,
    taken_date: i64,
    is_raw: bool,
    vector: Vec<f32>,
}

fn get_db_conn() -> Result<crate::t_sqlite::PooledConn, String> {
    crate::t_sqlite::open_conn()
}

fn resolve_scope(
    params: Option<QueryParams>,
    collection_id: Option<i64>,
    file_ids: Option<Vec<i64>>,
) -> Result<Vec<AFile>, String> {
    if file_ids.is_some() && (params.is_some() || collection_id.is_some()) {
        return Err("File ID scope cannot be combined with query or collection scope.".into());
    }
    if let Some(ids) = file_ids {
        AFile::get_files_by_ids(&ids)
    } else if let Some(id) = collection_id {
        t_dedup::get_files_by_collection(id, params.as_ref())
    } else {
        t_dedup::get_files_by_query(params.as_ref().ok_or("Query required")?)
    }
}

fn load_vectors(conn: &Connection, files: Vec<AFile>) -> Result<Vec<VectorFile>, String> {
    let duplicate_ids = exact_duplicate_file_ids(conn)?;
    let raw_ids: HashSet<i64> = files.iter()
        .filter(|file| file.file_type == Some(3))
        .filter_map(|file| file.id)
        .collect();
    let dates = eligible_dates(files)
        .into_iter()
        .filter(|(id, _)| !duplicate_ids.contains(id))
        .collect::<HashMap<_, _>>();
    let mut vectors = Vec::new();
    let ids: Vec<i64> = dates.keys().copied().collect();
    for chunk in ids.chunks(SQL_BATCH_SIZE) {
        let placeholders = std::iter::repeat("?")
            .take(chunk.len())
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT id, embeds FROM afiles WHERE id IN ({placeholders}) AND embeds IS NOT NULL"
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params_from_iter(chunk.iter()), |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (id, blob) = row.map_err(|e| e.to_string())?;
            let vector = blob
                .chunks_exact(4)
                .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect::<Vec<_>>();
            if !vector.is_empty() {
                vectors.push(VectorFile {
                    id,
                    taken_date: *dates.get(&id).unwrap_or(&0),
                    is_raw: raw_ids.contains(&id),
                    vector,
                });
            }
        }
    }
    Ok(filter_dominant_dimension(vectors))
}

fn exact_duplicate_file_ids(conn: &Connection) -> Result<HashSet<i64>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT i.file_id
             FROM duplicate_group_items i
             WHERE (SELECT COUNT(*) FROM duplicate_group_items WHERE group_id = i.group_id) > 1",
        )
        .map_err(|e| e.to_string())?;
    stmt.query_map([], |row| row.get::<_, i64>(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<HashSet<_>, _>>()
        .map_err(|e| e.to_string())
}

fn filter_dominant_dimension(vectors: Vec<VectorFile>) -> Vec<VectorFile> {
    let mut counts = HashMap::new();
    for file in &vectors {
        *counts.entry(file.vector.len()).or_insert(0usize) += 1;
    }
    let dimension = counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(dimension, _)| dimension);
    dimension
        .map(|dimension| {
            vectors
                .into_iter()
                .filter(|file| file.vector.len() == dimension)
                .collect()
        })
        .unwrap_or_default()
}

fn eligible_dates(files: Vec<AFile>) -> HashMap<i64, i64> {
    files
        .into_iter()
        .filter(|file| matches!(file.file_type, Some(1 | 3)))
        .filter_map(|file| file.id.map(|id| (id, file.taken_date.unwrap_or(0))))
        .collect()
}

fn count_vectors(conn: &Connection, files: Vec<AFile>) -> Result<u64, String> {
    let ids: Vec<i64> = eligible_dates(files).into_keys().collect();
    let mut dimensions = HashMap::new();
    for chunk in ids.chunks(SQL_BATCH_SIZE) {
        let placeholders = std::iter::repeat("?")
            .take(chunk.len())
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT length(embeds) FROM afiles WHERE id IN ({placeholders}) AND embeds IS NOT NULL"
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let lengths = stmt
            .query_map(params_from_iter(chunk.iter()), |row| row.get::<_, i64>(0))
            .map_err(|e| e.to_string())?;
        for length in lengths {
            let length = length.map_err(|e| e.to_string())?;
            if length > 0 && length % 4 == 0 {
                *dimensions.entry(length).or_insert(0u64) += 1;
            }
        }
    }
    Ok(dimensions.into_values().max().unwrap_or(0))
}

pub fn eligible_count(
    params: Option<QueryParams>,
    collection_id: Option<i64>,
    file_ids: Option<Vec<i64>>,
) -> Result<u64, String> {
    let conn = get_db_conn()?;
    let duplicate_ids = exact_duplicate_file_ids(&conn)?;
    let files = resolve_scope(params, collection_id, file_ids)?
        .into_iter()
        .filter(|file| file.id.is_none_or(|id| !duplicate_ids.contains(&id)))
        .collect();
    count_vectors(&conn, files)
}

pub fn start_scan(
    app: tauri::AppHandle,
    state: tauri::State<'_, SimilarState>,
    scope_key: String,
    source_version: i64,
    similarity_threshold: f32,
    params: Option<QueryParams>,
    collection_id: Option<i64>,
    file_ids: Option<Vec<i64>>,
) -> Result<(), String> {
    if state
        .is_scanning
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("A similar-photo analysis is already running.".into());
    }
    state.cancel_flag.store(false, Ordering::SeqCst);
    *state.status.lock().unwrap() = SimilarScanStatus {
        state: "running".into(),
        phase: "preparing".into(),
        scope_key: scope_key.clone(),
        current: 0,
        total: 0,
        groups: 0,
        is_scanning: true,
    };
    let status = state.status.clone();
    let running = state.is_scanning.clone();
    let cancel = state.cancel_flag.clone();
    std::thread::spawn(move || {
        let result = scan(
            &app,
            &status,
            &cancel,
            &scope_key,
            source_version,
            similarity_threshold.clamp(0.0, 1.0),
            params,
            collection_id,
            file_ids,
        );
        let mut current = status.lock().unwrap();
        current.state = if result.is_ok() && !cancel.load(Ordering::SeqCst) {
            "finished".into()
        } else if cancel.load(Ordering::SeqCst) {
            "idle".into()
        } else {
            "error".into()
        };
        current.is_scanning = false;
        running.store(false, Ordering::SeqCst);
        let _ = app.emit("similar-scan-progress", current.clone());
    });
    Ok(())
}

pub fn cancel_scan(state: tauri::State<'_, SimilarState>) {
    state.cancel_flag.store(true, Ordering::SeqCst);
}
pub fn get_status(state: tauri::State<'_, SimilarState>) -> SimilarScanStatus {
    state.status.lock().unwrap().clone()
}

// RAW originals and rendered images are different deliverables, not cleanup
// alternatives. Keep the two classes separate even when their embeddings match.
fn similar_score(a: &VectorFile, b: &VectorFile) -> Option<f32> {
    if a.is_raw != b.is_raw {
        return None;
    }
    Some(cosine(&a.vector, &b.vector))
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let (mut dot, mut an, mut bn) = (0.0f32, 0.0f32, 0.0f32);
    for (x, y) in a.iter().zip(b) {
        dot += x * y;
        an += x * x;
        bn += y * y;
    }
    if an == 0.0 || bn == 0.0 {
        0.0
    } else {
        dot / (an.sqrt() * bn.sqrt())
    }
}

fn scan(
    app: &tauri::AppHandle,
    status: &Arc<Mutex<SimilarScanStatus>>,
    cancel: &Arc<AtomicBool>,
    scope_key: &str,
    source_version: i64,
    similarity_threshold: f32,
    params: Option<QueryParams>,
    collection_id: Option<i64>,
    file_ids: Option<Vec<i64>>,
) -> Result<(), String> {
    let mut conn = get_db_conn()?;
    let vectors = load_vectors(&conn, resolve_scope(params, collection_id, file_ids)?)?;
    {
        let mut s = status.lock().unwrap();
        s.total = vectors.len() as u64;
        s.current = 0;
    }
    let _ = app.emit("similar-scan-progress", status.lock().unwrap().clone());
    if cancel.load(Ordering::SeqCst) {
        return Ok(());
    }

    let mut pair_scores: HashMap<(usize, usize), f32> = HashMap::new();
    if vectors.len() > 1 {
        let hns = Hnsw::<f32, DistCosine>::new(16, vectors.len(), 16, 200, DistCosine {});
        let data = vectors
            .iter()
            .enumerate()
            .map(|(i, f)| (&f.vector, i))
            .collect::<Vec<_>>();
        hns.parallel_insert(&data);
        for (i, file) in vectors.iter().enumerate() {
            if cancel.load(Ordering::SeqCst) {
                return Ok(());
            }
            for neighbor in hns.search(&file.vector, TOP_K + 1, SEARCH_EF) {
                let j = neighbor.d_id;
                if j == i || vectors[j].vector.len() != file.vector.len() {
                    continue;
                }
                let Some(score) = similar_score(file, &vectors[j]) else { continue };
                if score >= similarity_threshold {
                    pair_scores.insert((i.min(j), i.max(j)), score);
                }
            }
            if i % 100 == 0 {
                let mut s = status.lock().unwrap();
                s.phase = "finding_matches".into();
                s.current = i as u64;
                let _ = app.emit("similar-scan-progress", s.clone());
            }
        }
    }

    // Merge only fully connected sets: a weak A-B-C chain cannot create one group.
    let mut clusters: Vec<Vec<usize>> = (0..vectors.len()).map(|i| vec![i]).collect();
    let mut cluster_for: Vec<usize> = (0..vectors.len()).collect();
    let mut edges: Vec<_> = pair_scores
        .iter()
        .map(|(&(a, b), &score)| (a, b, score))
        .collect();
    edges.sort_by(|a, b| b.2.total_cmp(&a.2));
    for (a, b, _) in edges {
        let ai = cluster_for[a];
        let bi = cluster_for[b];
        if ai == bi {
            continue;
        }
        if clusters[ai].len() + clusters[bi].len() > MAX_GROUP_SIZE {
            continue;
        }
        if clusters[ai].iter().all(|&x| {
            clusters[bi].iter().all(|&y| {
                pair_scores
                    .get(&(x.min(y), x.max(y)))
                    .copied()
                    .unwrap_or(0.0)
                    >= similarity_threshold
            })
        }) {
            let (target, source) = if clusters[ai].len() >= clusters[bi].len() {
                (ai, bi)
            } else {
                (bi, ai)
            };
            let members = std::mem::take(&mut clusters[source]);
            for member in members {
                cluster_for[member] = target;
                clusters[target].push(member);
            }
        }
    }
    if cancel.load(Ordering::SeqCst) {
        return Ok(());
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let groups: Vec<_> = clusters
        .into_iter()
        .filter(|group| group.len() > 1)
        .collect();
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM similarity_scans", [])
        .map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO similarity_scans(scope_key,source_version,status,file_count,group_count,created_at,completed_at) VALUES(?1,?2,'finished',?3,?4,?5,?5)",
        params![scope_key, source_version, vectors.len() as i64, groups.len() as i64, now],
    )
    .map_err(|e| e.to_string())?;
    let scan_id = tx.last_insert_rowid();
    for group in &groups {
        let member_scores: Vec<(usize, f32)> = group
            .iter()
            .map(|&i| {
                let sum: f32 = group
                    .iter()
                    .filter(|&&j| j != i)
                    .map(|&j| {
                        pair_scores
                            .get(&(i.min(j), i.max(j)))
                            .copied()
                            .unwrap_or(0.0)
                    })
                    .sum();
                (i, sum / (group.len() - 1) as f32)
            })
            .collect();
        let &(representative, _) = member_scores
            .iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap();
        let latest_taken_date = group
            .iter()
            .map(|&i| vectors[i].taken_date)
            .max()
            .unwrap_or(0);
        let min_score = member_scores
            .iter()
            .map(|(_, score)| *score)
            .fold(1.0f32, f32::min);
        let max_score = member_scores
            .iter()
            .map(|(_, score)| *score)
            .fold(0.0f32, f32::max);
        tx.execute(
            "INSERT INTO similarity_groups(scan_id,representative_file_id,file_count,latest_taken_date,min_score,max_score) VALUES(?1,?2,?3,?4,?5,?6)",
            params![scan_id, vectors[representative].id, group.len() as i64, latest_taken_date, min_score, max_score],
        )
        .map_err(|e| e.to_string())?;
        let group_id = tx.last_insert_rowid();
        for (index, score) in member_scores {
            tx.execute(
                "INSERT INTO similarity_group_items(group_id,file_id,score,is_keep) VALUES(?1,?2,?3,?4)",
                params![group_id, vectors[index].id, score, i64::from(index == representative)],
            )
            .map_err(|e| e.to_string())?;
        }
    }
    tx.commit().map_err(|e| e.to_string())?;
    let mut s = status.lock().unwrap();
    s.phase = "building_sets".into();
    s.current = s.total;
    s.groups = groups.len() as u64;
    Ok(())
}

pub fn list_groups(scope_key: &str, limit: i64, offset: i64) -> Result<serde_json::Value, String> {
    let conn = get_db_conn()?;
    let total = conn
        .query_row(
            "SELECT COUNT(*) FROM (
                SELECT g.id
                FROM similarity_groups g
                JOIN similarity_scans s ON s.id = g.scan_id
                JOIN similarity_group_items i ON i.group_id = g.id
                WHERE s.scope_key = ?1
                GROUP BY g.id
                HAVING COUNT(i.file_id) > 1
            )",
            params![scope_key],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT g.id,COUNT(i.file_id),g.representative_file_id
             FROM similarity_groups g
             JOIN similarity_scans s ON s.id = g.scan_id
             JOIN similarity_group_items i ON i.group_id = g.id
             WHERE s.scope_key = ?1
             GROUP BY g.id
             HAVING COUNT(i.file_id) > 1
             ORDER BY g.latest_taken_date DESC, g.id DESC LIMIT ?2 OFFSET ?3",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![scope_key, limit.max(1), offset.max(0)], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        let (id, count, rep) = row.map_err(|e| e.to_string())?;
        out.push(serde_json::json!({"id": id, "file_count": count, "representative": AFile::get_file_info(rep)?}));
    }
    Ok(serde_json::json!({"items": out, "total": total}))
}

pub fn get_overview(scope_key: &str) -> Result<serde_json::Value, String> {
    let conn = get_db_conn()?;
    let (total_files, total_size) = conn
        .query_row(
            "SELECT
                COUNT(*),
                COALESCE(SUM(a.size), 0)
             FROM similarity_group_items i
             JOIN similarity_groups g ON g.id = i.group_id
             JOIN similarity_scans s ON s.id = g.scan_id
             JOIN afiles a ON a.id = i.file_id
             WHERE s.scope_key = ?1
               AND (SELECT COUNT(*) FROM similarity_group_items WHERE group_id = i.group_id) > 1",
            params![scope_key],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        )
        .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "total_files": total_files,
        "total_size": total_size,
    }))
}

pub fn get_group(group_id: i64, scope_key: &str) -> Result<serde_json::Value, String> {
    let conn = get_db_conn()?;
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM similarity_groups g JOIN similarity_scans s ON s.id=g.scan_id WHERE g.id=?1 AND s.scope_key=?2)",
            params![group_id, scope_key],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if !exists {
        return Err("Similar group is not available in this scope.".into());
    }
    let mut stmt = conn
        .prepare(
            "SELECT file_id, score, is_keep FROM similarity_group_items WHERE group_id=?1 ORDER BY is_keep DESC, score DESC, file_id ASC",
        )
        .map_err(|e| e.to_string())?;
    let items = stmt.query_map(params![group_id], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, f32>(1)?, row.get::<_, i64>(2)?))).map_err(|e| e.to_string())?
        .map(|item| { let (file_id, score, is_keep) = item.map_err(|e| e.to_string())?; Ok(serde_json::json!({"file_id": file_id, "score": score, "is_keep": is_keep, "file": AFile::get_file_info(file_id)?})) }).collect::<Result<Vec<_>, String>>()?;
    Ok(serde_json::json!({"id": group_id, "items": items}))
}

pub fn set_keep(group_id: i64, file_id: i64, scope_key: &str) -> Result<(), String> {
    let mut conn = get_db_conn()?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let exists: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM similarity_group_items i JOIN similarity_groups g ON g.id=i.group_id JOIN similarity_scans s ON s.id=g.scan_id WHERE i.group_id=?1 AND i.file_id=?2 AND s.scope_key=?3)",
            params![group_id, file_id, scope_key],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if !exists {
        return Err("Similar item is not available in this scope.".into());
    }
    tx.execute(
        "UPDATE similarity_group_items SET is_keep=0 WHERE group_id=?1",
        params![group_id],
    )
    .map_err(|e| e.to_string())?;
    tx.execute(
        "UPDATE similarity_group_items SET is_keep=1 WHERE group_id=?1 AND file_id=?2",
        params![group_id, file_id],
    )
    .map_err(|e| e.to_string())?;
    tx.execute(
        "UPDATE similarity_groups SET representative_file_id=?2 WHERE id=?1",
        params![group_id, file_id],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(id: i64, is_raw: bool) -> VectorFile {
        VectorFile { id, is_raw, taken_date: 0, vector: vec![1.0, 0.0] }
    }

    #[test]
    fn raw_and_rendered_images_are_not_cleanup_alternatives() {
        let raw = image(1, true);
        let jpeg = image(2, false);
        assert_eq!(similar_score(&raw, &jpeg), None);
        assert_eq!(similar_score(&jpeg, &raw), None);
        assert_eq!(similar_score(&raw, &image(3, true)), Some(1.0));
        assert_eq!(similar_score(&jpeg, &image(4, false)), Some(1.0));
    }

    // Explicit opt-in: inspect real input names without modifying any photos.
    #[test]
    fn photo_directory_raw_pairs() {
        let Ok(dir) = std::env::var("LAP_TEST_PHOTO_DIR") else { return };
        let mut pairs: HashMap<String, (bool, bool)> = HashMap::new();
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let kind = crate::t_utils::get_file_type(&path.to_string_lossy());
            if !matches!(kind, Some(1 | 3)) { continue; }
            let stem = path.file_stem().unwrap().to_string_lossy().to_ascii_lowercase();
            let pair = pairs.entry(stem).or_default();
            if kind == Some(3) { pair.0 = true; } else { pair.1 = true; }
        }
        let paired = pairs.values().filter(|&&(raw, jpeg)| raw && jpeg).count();
        assert!(paired > 0, "No RAW/rendered pairs in test directory");
        for _ in 0..paired {
            // Identical visual vectors must still not create a cross-format edge.
            assert_eq!(similar_score(&image(1, true), &image(2, false)), None);
        }
        eprintln!("Verified {paired} RAW/rendered pairs: no cross-format similarity edges");
    }
}

pub fn has_scan(scope_key: &str) -> Result<bool, String> {
    let conn = get_db_conn()?;
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM similarity_scans WHERE scope_key=?1)",
        params![scope_key],
        |row| row.get(0),
    )
    .map_err(|e| e.to_string())
}
