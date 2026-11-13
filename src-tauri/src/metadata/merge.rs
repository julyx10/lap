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
            "UPDATE afiles SET e_title=?1, e_headline=?2, e_artist=?3, e_copyright=?4, e_software=?5
             WHERE id=?6 AND (e_title IS NOT ?1 OR e_headline IS NOT ?2 OR e_artist IS NOT ?3
                              OR e_copyright IS NOT ?4 OR e_software IS NOT ?5)",
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
        let db_comment_empty = db_comments.as_deref().is_none_or(|c| c.trim().is_empty());

        let should_update = match mode {
            XmpImportMode::InitialScan => !trimmed.is_empty(),
            XmpImportMode::RescanNonDestructive | XmpImportMode::RefreshNonDestructive => {
                dirty & CatalogField::Description.mask() == 0
                    && db_comment_empty
                    && !trimmed.is_empty()
            }
            XmpImportMode::ManualOverwrite => true,
        };
        if should_update {
            let new_val: Option<&str> = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            };
            conn.execute(
                "UPDATE afiles SET comments = ?1 WHERE id = ?2",
                params![new_val, file_id],
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
            let represented = |tag: &str| {
                paths.iter().any(|path| {
                    path.eq_ignore_ascii_case(tag.trim())
                        || path
                            .split('|')
                            .any(|part| part.trim().eq_ignore_ascii_case(tag.trim()))
                })
            };
            // Only explicit source paths justify replacing old flattened associations.
            // Keep the old root nodes themselves; they may be used on other files.
            if dirty & CatalogField::Tags.mask() == 0 {
                for tag in tags.iter().filter(|tag| represented(tag)) {
                    conn.execute("DELETE FROM afile_tags WHERE file_id=?1 AND tag_id IN (SELECT id FROM atags WHERE parent_id IS NULL AND name=?2 COLLATE NOCASE)", params![file_id,tag])
                        .map_err(|e|e.to_string())?;
                }
            }
            let mut added_any = false;
            for value in paths.iter().map(|p| (p, true)).chain(
                tags.iter()
                    .filter(|tag| !represented(tag))
                    .map(|tag| (tag, false)),
            ) {
                let Some(tag_id) = crate::t_tags::import_path(conn, value.0, value.1, tag_source(value.0))? else {
                    continue;
                };
                conn.execute(
                    "INSERT OR IGNORE INTO afile_tags(file_id,tag_id) VALUES(?1,?2)",
                    params![file_id, tag_id],
                )
                .map_err(|e| e.to_string())?;
                added_any = true;
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
