//! Transactional catalog merge and local-edit protection.
use super::keywords::push_keyword;
use super::model::{XmpImportMode, XmpMetadata};
use super::schema::CatalogField;
use rusqlite::{Connection, params};

#[cfg(test)]
pub(super) fn apply_xmp_metadata_to_db(
    file_id: i64,
    meta: &XmpMetadata,
    conn: &Connection,
    mode: XmpImportMode,
) -> Result<bool, String> {
    apply_with_state(file_id, meta, conn, mode, None)
}

pub(super) fn apply_with_state(
    file_id: i64,
    meta: &XmpMetadata,
    conn: &Connection,
    mode: XmpImportMode,
    fingerprint: Option<&str>,
) -> Result<bool, String> {
    apply_with_source_text(file_id, meta, conn, mode, fingerprint, None, None, |_| "xmp_sidecar")
}

pub(super) fn apply_with_source_text(
    file_id: i64,
    meta: &XmpMetadata,
    conn: &Connection,
    mode: XmpImportMode,
    fingerprint: Option<&str>,
    source: Option<&super::model::SourceText>,
    capture: Option<super::model::SourceCapture>,
    tag_source: impl Fn(&str) -> &'static str,
) -> Result<bool, String> {
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let dirty: i32 = tx
        .query_row(
            "SELECT dirty FROM axmp_imports WHERE file_id = ?1",
            [file_id],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let result = apply_fields(file_id, meta, &tx, mode, dirty, tag_source)?;
    tx.execute("INSERT INTO axmp_imports(file_id, fingerprint, dirty) VALUES(?1, ?2, ?3)
        ON CONFLICT(file_id) DO UPDATE SET fingerprint = COALESCE(excluded.fingerprint, fingerprint), dirty = excluded.dirty",
        params![file_id, fingerprint, dirty]).map_err(|e| e.to_string())?;
    if let Some(source) = source {
        tx.execute(
            "UPDATE afiles SET e_title=COALESCE(e_title,?1), e_headline=COALESCE(e_headline,?2), e_artist=COALESCE(e_artist,?3), e_copyright=COALESCE(e_copyright,?4), e_software=?5
             WHERE id=?6 AND ((e_title IS NULL AND ?1 IS NOT NULL) OR (e_headline IS NULL AND ?2 IS NOT NULL) OR (e_artist IS NULL AND ?3 IS NOT NULL)
                              OR (e_copyright IS NULL AND ?4 IS NOT NULL) OR e_software IS NOT ?5)",
            params![source.title, source.headline, source.creator, source.copyright, source.software, file_id],
        ).map_err(|e| e.to_string())?;
    }
    if let Some((date, taken)) = capture.as_ref().and_then(|capture| capture.date.as_ref()) {
        tx.execute("UPDATE afiles SET e_date_time=?1, taken_date=?2 WHERE id=?3",
            params![date, taken, file_id]).map_err(|e| e.to_string())?;
    }
    if let Some((lat, lon, alt)) = capture.and_then(|capture| capture.gps) {
        let (name, admin1, admin2, country) = crate::t_sqlite::AFile::geocode(lat, lon);
        tx.execute(
            "UPDATE afiles SET gps_latitude=?1, gps_longitude=?2, gps_altitude=?3,
             geo_name=?4, geo_admin1=?5, geo_admin2=?6, geo_cc=?7 WHERE id=?8",
            params![lat, lon, alt, name, admin1, admin2, country, file_id],
        ).map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(result)
}

fn apply_fields(
    file_id: i64,
    meta: &XmpMetadata,
    conn: &Connection,
    mode: XmpImportMode,
    dirty: i32,
    tag_source: impl Fn(&str) -> &'static str,
) -> Result<bool, String> {
    let (db_rating, db_culling_flag, db_comments, db_favorite): (i32, i32, Option<String>, bool) = conn.query_row(
        "SELECT COALESCE(rating, 0), COALESCE(culling_flag, 0), comments, COALESCE(is_favorite, 0) FROM afiles WHERE id = ?1", [file_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    ).map_err(|e| e.to_string())?;

    if let Some(favorite) = meta.favorite {
        let should_update = match mode {
            XmpImportMode::InitialScan => favorite,
            XmpImportMode::RescanNonDestructive | XmpImportMode::RefreshNonDestructive => {
                dirty & CatalogField::Favorite.mask() == 0 && !db_favorite && favorite
            }
            XmpImportMode::ManualOverwrite => favorite != db_favorite,
        };
        if should_update {
            conn.execute(
                "UPDATE afiles SET is_favorite=?1 WHERE id=?2",
                params![favorite, file_id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // 1. Rating
    if let Some(rating) = meta.rating {
        let clamped = rating.clamp(0, 5);
        let should_update = match mode {
            XmpImportMode::InitialScan => clamped > 0,
            XmpImportMode::RescanNonDestructive | XmpImportMode::RefreshNonDestructive => {
                dirty & CatalogField::Rating.mask() == 0 && db_rating == 0 && clamped > 0
            }
            XmpImportMode::ManualOverwrite => clamped != db_rating,
        };
        if should_update {
            conn.execute(
                "UPDATE afiles SET rating = ?1 WHERE id = ?2",
                params![clamped, file_id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // 2. Culling flag
    if let Some(flag) = meta.culling_flag {
        let clamped = flag.clamp(0, 2);
        let should_update = match mode {
            XmpImportMode::InitialScan => clamped > 0,
            XmpImportMode::RescanNonDestructive | XmpImportMode::RefreshNonDestructive => {
                dirty & CatalogField::Culling.mask() == 0 && db_culling_flag == 0 && clamped > 0
            }
            XmpImportMode::ManualOverwrite => clamped != db_culling_flag,
        };
        if should_update {
            conn.execute(
                "UPDATE afiles SET culling_flag = ?1 WHERE id = ?2",
                params![clamped, file_id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // 3. Comments / description
    if let Some(ref desc) = meta.description {
        let trimmed = desc.trim();
        let protected_clear = matches!(mode, XmpImportMode::RescanNonDestructive | XmpImportMode::RefreshNonDestructive)
            && dirty & CatalogField::Description.mask() != 0;
        if db_comments.is_none() && !trimmed.is_empty() && !protected_clear {
            conn.execute(
                "UPDATE afiles SET comments = ?1 WHERE id = ?2",
                params![trimmed, file_id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // 4. Keywords / Tags
    if let Some(ref tags) = meta.tags {
        if matches!(
            mode,
            XmpImportMode::RescanNonDestructive | XmpImportMode::RefreshNonDestructive
        ) && dirty & CatalogField::Tags.mask() != 0
        {
            return Ok(true);
        }
        if mode == XmpImportMode::ManualOverwrite {
            conn.execute("DELETE FROM afile_tags WHERE file_id = ?1", [file_id])
                .map_err(|e| e.to_string())?;
            conn.execute("UPDATE afiles SET has_tags = 0 WHERE id = ?1", [file_id])
                .map_err(|e| e.to_string())?;
        }
        if tags.is_empty() {
            if mode == XmpImportMode::ManualOverwrite {
                conn.execute(
                    "DELETE FROM afile_tags WHERE file_id = ?1",
                    params![file_id],
                )
                .map_err(|e| e.to_string())?;
                conn.execute(
                    "UPDATE afiles SET has_tags = 0 WHERE id = ?1",
                    params![file_id],
                )
                .map_err(|e| e.to_string())?;
            }
        } else {
            let paths = meta.hierarchical_tags.as_deref().unwrap_or_default();
            // Flat stores may contain every ancestor, not just the assigned leaf.
            let represented_by = |path: &str, tag: &str| {
                path.eq_ignore_ascii_case(tag.trim())
                    || path.split('|').any(|part| part.trim().eq_ignore_ascii_case(tag.trim()))
            };
            let restore_deleted = mode == XmpImportMode::ManualOverwrite;
            let mut imported = Vec::new();
            for path in paths {
                if let Some(tag_id) = crate::t_tags::import_path(conn, path, true, tag_source(path), restore_deleted)? {
                    imported.push(tag_id);
                    // Only a successfully resolved path can replace a flattened association.
                    if dirty & CatalogField::Tags.mask() == 0 {
                        for tag in tags.iter().filter(|tag| represented_by(path, tag)) {
                            conn.execute("DELETE FROM afile_tags WHERE file_id=?1 AND tag_id IN (SELECT id FROM atags WHERE parent_id IS NULL AND name=?2 COLLATE NOCASE)", params![file_id,tag])
                                .map_err(|e|e.to_string())?;
                        }
                    }
                }
            }
            for tag in tags.iter().filter(|tag| !paths.iter().any(|path| represented_by(path, tag))) {
                if let Some(tag_id) = crate::t_tags::import_path(conn, tag, false, tag_source(tag), restore_deleted)? {
                    imported.push(tag_id);
                }
            }
            let added_any = !imported.is_empty();
            for tag_id in imported {
                conn.execute(
                    "INSERT OR IGNORE INTO afile_tags(file_id,tag_id) VALUES(?1,?2)",
                    params![file_id, tag_id],
                )
                .map_err(|e| e.to_string())?;
            }

            if added_any {
                conn.execute(
                    "UPDATE afiles SET has_tags = 1 WHERE id = ?1",
                    params![file_id],
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(true)
}

pub fn merge_system(embedded: &mut XmpMetadata, system: XmpMetadata) {
    if embedded
        .description
        .as_deref()
        .is_none_or(|value| value.trim().is_empty())
    {
        embedded.description = system.description;
    }
    embedded.rating = embedded.rating.or(system.rating);
    if let Some(tags) = system.tags {
        for tag in tags {
            push_keyword(&tag, embedded.tags.get_or_insert_with(Vec::new));
        }
    }
}

/// Explicit sidecar properties take precedence, including empty values. A missing
/// keyword property must preserve both flat and hierarchical embedded tags.
pub(super) fn merge_sidecar(embedded: &mut XmpMetadata, sidecar: XmpMetadata) {
    embedded.favorite = sidecar.favorite.or(embedded.favorite);
    embedded.rating = sidecar.rating.or(embedded.rating);
    embedded.culling_flag = sidecar.culling_flag.or(embedded.culling_flag);
    if sidecar.tags.is_some() {
        embedded.tags = sidecar.tags;
        embedded.hierarchical_tags = sidecar.hierarchical_tags;
    }
    embedded.description = sidecar.description.or(embedded.description.take());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deleted_hierarchy_preserves_flat_association_until_manual_overwrite() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE atags(id INTEGER PRIMARY KEY,name TEXT,parent_id INTEGER,source TEXT);
             CREATE TABLE afiles(id INTEGER PRIMARY KEY,rating INTEGER,culling_flag INTEGER,comments TEXT,is_favorite INTEGER,has_tags INTEGER);
             CREATE TABLE afile_tags(file_id INTEGER,tag_id INTEGER,PRIMARY KEY(file_id,tag_id));
             INSERT INTO afiles VALUES(1,0,0,NULL,0,1);
             INSERT INTO atags VALUES(1,'Anniversary',NULL,NULL);
             INSERT INTO afile_tags VALUES(1,1);",
        ).unwrap();
        crate::t_tags::initialize(&conn).unwrap();
        conn.execute("INSERT INTO axmp_tag_paths VALUES(1,'Context|Anniversary',NULL)", []).unwrap();
        let metadata = XmpMetadata {
            tags: Some(vec!["Anniversary".into(), "Travel".into()]),
            hierarchical_tags: Some(vec!["Context|Anniversary".into()]),
            ..Default::default()
        };
        let assigned_paths = || crate::t_tags::keywords_for_file(&conn, 1).unwrap().1;
        apply_fields(1, &metadata, &conn, XmpImportMode::RescanNonDestructive, 0, |_| "xmp_sidecar").unwrap();
        assert_eq!(assigned_paths(), ["Anniversary", "Travel"]);

        apply_fields(1, &metadata, &conn, XmpImportMode::ManualOverwrite, 0, |_| "xmp_sidecar").unwrap();
        assert_eq!(assigned_paths(), ["Context|Anniversary", "Travel"]);
        assert!(crate::t_tags::import_path(&conn, "Context|Anniversary", true, "xmp_sidecar", false).unwrap().is_some());
    }
}
