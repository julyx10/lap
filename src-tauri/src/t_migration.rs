use rusqlite::Connection;

struct Migration {
    version: i32,
    description: &'static str,
    sql: &'static str,
}

fn get_migrations() -> Vec<Migration> {
    vec![
        // v0.1.0 — initial release schema
        Migration {
            version: 1,
            description: "Create deduplication tables",
            sql: "
                CREATE TABLE IF NOT EXISTS file_hashes (
                    file_id INTEGER PRIMARY KEY,
                    hash TEXT NOT NULL,
                    file_size INTEGER NOT NULL,
                    mtime INTEGER NOT NULL,
                    computed_at INTEGER NOT NULL,
                    FOREIGN KEY (file_id) REFERENCES afiles(id) ON DELETE CASCADE
                );
                CREATE INDEX IF NOT EXISTS idx_file_hashes_hash_size ON file_hashes(hash, file_size);
                CREATE INDEX IF NOT EXISTS idx_file_hashes_mtime ON file_hashes(mtime);

                CREATE TABLE IF NOT EXISTS duplicate_groups (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    hash TEXT NOT NULL,
                    file_size INTEGER NOT NULL,
                    file_count INTEGER NOT NULL,
                    total_size INTEGER NOT NULL,
                    reviewed INTEGER NOT NULL DEFAULT 0,
                    updated_at INTEGER NOT NULL
                );
                CREATE UNIQUE INDEX IF NOT EXISTS uidx_duplicate_groups_hash_size ON duplicate_groups(hash, file_size);

                CREATE TABLE IF NOT EXISTS duplicate_group_items (
                    group_id INTEGER NOT NULL,
                    file_id INTEGER NOT NULL,
                    is_keep INTEGER NOT NULL DEFAULT 0,
                    is_selected INTEGER NOT NULL DEFAULT 0,
                    score REAL NOT NULL DEFAULT 0,
                    PRIMARY KEY (group_id, file_id),
                    FOREIGN KEY (group_id) REFERENCES duplicate_groups(id) ON DELETE CASCADE,
                    FOREIGN KEY (file_id) REFERENCES afiles(id) ON DELETE CASCADE
                );
                CREATE INDEX IF NOT EXISTS idx_dup_items_group ON duplicate_group_items(group_id);
                CREATE INDEX IF NOT EXISTS idx_dup_items_file ON duplicate_group_items(file_id);
            ",
        },
        // v0.1.x — RAW / format_label support
        Migration {
            version: 2,
            description: "Add afiles.format_label column",
            sql: "",
        },
        // v0.2.2 — thumbnail cache
        Migration {
            version: 3,
            description: "Add thumbnail cache metadata columns",
            sql: "",
        },
        // v0.2.2 — incremental scan foundation
        Migration {
            version: 4,
            description: "Add last_scan_time for file and album sync",
            sql: "",
        },
        // v0.2.4 — folder exclusion, inode, search exclusion
        Migration {
            version: 5,
            description: "Post v0.2.2 schema updates",
            sql: "",
        },
        // Post v0.2.4 — collections, unique folder names, Live Photo,
        // case-insensitive index, folder scan state
        Migration {
            version: 9,
            description: "Post v0.2.4 schema updates",
            sql: "",
        },
        Migration {
            version: 10,
            description: "Create visual similarity review tables",
            sql: "
                CREATE TABLE IF NOT EXISTS similarity_scans (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    scope_key TEXT NOT NULL UNIQUE,
                    source_version INTEGER NOT NULL,
                    status TEXT NOT NULL,
                    file_count INTEGER NOT NULL DEFAULT 0,
                    group_count INTEGER NOT NULL DEFAULT 0,
                    created_at INTEGER NOT NULL,
                    completed_at INTEGER
                );
                CREATE TABLE IF NOT EXISTS similarity_groups (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    scan_id INTEGER NOT NULL,
                    representative_file_id INTEGER NOT NULL,
                    file_count INTEGER NOT NULL,
                    latest_taken_date INTEGER NOT NULL DEFAULT 0,
                    min_score REAL NOT NULL DEFAULT 0,
                    max_score REAL NOT NULL DEFAULT 0,
                    FOREIGN KEY (scan_id) REFERENCES similarity_scans(id) ON DELETE CASCADE
                );
                CREATE INDEX IF NOT EXISTS idx_similarity_groups_scan ON similarity_groups(scan_id);
                CREATE TABLE IF NOT EXISTS similarity_group_items (
                    group_id INTEGER NOT NULL,
                    file_id INTEGER NOT NULL,
                    score REAL NOT NULL,
                    is_keep INTEGER NOT NULL DEFAULT 0,
                    PRIMARY KEY (group_id, file_id),
                    FOREIGN KEY (group_id) REFERENCES similarity_groups(id) ON DELETE CASCADE,
                    FOREIGN KEY (file_id) REFERENCES afiles(id) ON DELETE CASCADE
                );
                CREATE INDEX IF NOT EXISTS idx_similarity_items_group ON similarity_group_items(group_id);
                CREATE INDEX IF NOT EXISTS idx_similarity_items_file ON similarity_group_items(file_id);
            ",
        },
        Migration {
            version: 11,
            description: "Add file culling status",
            sql: "",
        },
        Migration {
            version: 12,
            description: "Cache direct subfolder state on folders",
            sql: "",
        },
        Migration {
            version: 13,
            description: "Persist folder filesystem identity",
            sql: "",
        },
        Migration {
            version: 14,
            description: "Store scan totals",
            sql: "",
        },
        Migration {
            version: 15,
            description: "Persist visual similarity keep state",
            sql: "",
        },
        Migration {
            version: 16,
            description: "Add motion photo offset",
            sql: "",
        },
        Migration {
            version: 17,
            description: "Add tag groups and persistent ordering",
            sql: "",
        },
        Migration {
            version: 18,
            description: "Album scan scope and pixel filters",
            sql: "",
        },
        Migration {
            version: 19,
            description: "Cloud-only file state",
            sql: "",
        },
        Migration {
            version: 20,
            description: "Stored image color profile, color space and bit depth",
            sql: "",
        },
        Migration {
            version: 21,
            description: "Replace tag groups with hierarchical tags",
            sql: "",
        },
    ]
}

fn migrate_unique_album_files(conn: &Connection) -> Result<(), String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Migration 9 (deduplicate album files) failed starting transaction: {}", e))?;

    tx.execute_batch(
        "
        CREATE TEMP TABLE duplicate_afile_map (
            old_id INTEGER PRIMARY KEY,
            new_id INTEGER NOT NULL
        );

        INSERT INTO duplicate_afile_map (old_id, new_id)
        SELECT a.id, duplicates.new_id
        FROM afiles a
        JOIN (
            SELECT folder_id, name, MAX(id) AS new_id
            FROM afiles
            GROUP BY folder_id, name
            HAVING COUNT(*) > 1
        ) duplicates
          ON duplicates.folder_id = a.folder_id
         AND duplicates.name = a.name
        WHERE a.id <> duplicates.new_id;

        UPDATE albums
        SET cover_file_id = (
            SELECT new_id
            FROM duplicate_afile_map
            WHERE old_id = albums.cover_file_id
        )
        WHERE cover_file_id IN (SELECT old_id FROM duplicate_afile_map);

        UPDATE persons
        SET cover_face_id = NULL
        WHERE cover_face_id IN (
            SELECT faces.id
            FROM faces
            JOIN duplicate_afile_map ON duplicate_afile_map.old_id = faces.file_id
        );

        DELETE FROM afiles
        WHERE id IN (SELECT old_id FROM duplicate_afile_map);

        DROP INDEX IF EXISTS idx_afiles_folder_id_name;
        DROP INDEX IF EXISTS uidx_afiles_folder_id_name;
        CREATE UNIQUE INDEX uidx_afiles_folder_id_name
            ON afiles(folder_id, name);

        DROP TABLE duplicate_afile_map;
        ",
    )
    .map_err(|e| format!("Migration 9 (deduplicate album files) failed: {}", e))?;

    tx.commit()
        .map_err(|e| format!("Migration 9 (deduplicate album files) failed committing transaction: {}", e))
}

fn table_has_column(conn: &Connection, table: &str, column: &str) -> Result<bool, String> {
    let pragma = format!("PRAGMA table_info({})", table);
    let mut stmt = conn.prepare(&pragma).map_err(|e| e.to_string())?;
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;

    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let name: String = row.get(1).map_err(|e| e.to_string())?;
        if name.eq_ignore_ascii_case(column) {
            return Ok(true);
        }
    }

    Ok(false)
}

pub fn check_and_migrate(conn: &Connection) -> Result<(), String> {
    let current_version: i32 = conn
        .query_row("PRAGMA user_version;", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    println!("Current DB version: {}", current_version);

    let migrations = get_migrations();
    let mut new_version = current_version;

    for migration in &migrations {
        if migration.version > current_version {
            println!(
                "Applying migration {}: {}",
                migration.version, migration.description
            );

            // Execute the migration logic
            if migration.version == 2 {
                if !table_has_column(conn, "afiles", "format_label")? {
                    conn.execute("ALTER TABLE afiles ADD COLUMN format_label TEXT", [])
                        .map_err(|e| format!("Migration {} failed: {}", migration.version, e))?;
                }
            } else if migration.version == 3 {
                if !table_has_column(conn, "athumbs", "thumb_key")? {
                    conn.execute("ALTER TABLE athumbs ADD COLUMN thumb_key TEXT", [])
                        .map_err(|e| format!("Migration {} failed: {}", migration.version, e))?;
                }
                if !table_has_column(conn, "athumbs", "thumb_mtime")? {
                    conn.execute("ALTER TABLE athumbs ADD COLUMN thumb_mtime INTEGER", [])
                        .map_err(|e| format!("Migration {} failed: {}", migration.version, e))?;
                }
                if !table_has_column(conn, "athumbs", "thumb_size")? {
                    conn.execute("ALTER TABLE athumbs ADD COLUMN thumb_size INTEGER", [])
                        .map_err(|e| format!("Migration {} failed: {}", migration.version, e))?;
                }
                if !table_has_column(conn, "athumbs", "updated_at")? {
                    conn.execute("ALTER TABLE athumbs ADD COLUMN updated_at INTEGER", [])
                        .map_err(|e| format!("Migration {} failed: {}", migration.version, e))?;
                }
                conn.execute(
                    "CREATE INDEX IF NOT EXISTS idx_athumbs_thumb_key ON athumbs(thumb_key)",
                    [],
                )
                .map_err(|e| format!("Migration {} failed: {}", migration.version, e))?;
            } else if migration.version == 4 {
                if !table_has_column(conn, "afiles", "last_scan_time")? {
                    conn.execute(
                        "ALTER TABLE afiles ADD COLUMN last_scan_time INTEGER DEFAULT 0",
                        [],
                    )
                    .map_err(|e| {
                        format!(
                            "Migration {} failed adding last_scan_time: {}",
                            migration.version, e
                        )
                    })?;
                }
                if !table_has_column(conn, "albums", "last_scan_time")? {
                    conn.execute(
                        "ALTER TABLE albums ADD COLUMN last_scan_time INTEGER DEFAULT 0",
                        [],
                    )
                    .map_err(|e| {
                        format!(
                            "Migration {} failed adding last_scan_time: {}",
                            migration.version, e
                        )
                    })?;
                }
                conn.execute(
                    "CREATE INDEX IF NOT EXISTS idx_afiles_last_scan_time ON afiles(last_scan_time)",
                    [],
                )
                .map_err(|e| format!("Migration {} failed adding last_scan_time index: {}", migration.version, e))?;
            } else if migration.version == 5 {
                if !table_has_column(conn, "afolders", "is_excluded_from_search")? {
                    conn.execute(
                        "ALTER TABLE afolders ADD COLUMN is_excluded_from_search INTEGER DEFAULT 0",
                        [],
                    )
                    .map_err(|e| {
                        format!(
                            "Migration {} failed adding is_excluded_from_search: {}",
                            migration.version, e
                        )
                    })?;
                }
                if !table_has_column(conn, "afiles", "inode")? {
                    conn.execute("ALTER TABLE afiles ADD COLUMN inode INTEGER", [])
                        .map_err(|e| {
                            format!("Migration {} failed adding inode: {}", migration.version, e)
                        })?;
                }
                conn.execute(
                    "CREATE INDEX IF NOT EXISTS idx_afolders_is_excluded_from_search ON afolders(is_excluded_from_search)",
                    [],
                )
                .map_err(|e| {
                    format!(
                        "Migration {} failed adding is_excluded_from_search index: {}",
                        migration.version, e
                    )
                })?;
            } else if migration.version == 9 {
                // --- collections ---
                conn.execute_batch(
                    "CREATE TABLE IF NOT EXISTS acollections (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        name TEXT NOT NULL,
                        sort_order INTEGER NOT NULL DEFAULT 0,
                        created_at INTEGER NOT NULL,
                        updated_at INTEGER NOT NULL
                    );
                    CREATE INDEX IF NOT EXISTS idx_acollections_sort ON acollections(sort_order, id);

                    CREATE TABLE IF NOT EXISTS acollections_files (
                        collection_id INTEGER NOT NULL,
                        file_id INTEGER NOT NULL,
                        added_at INTEGER NOT NULL,
                        PRIMARY KEY (collection_id, file_id),
                        FOREIGN KEY (collection_id) REFERENCES acollections(id) ON DELETE CASCADE,
                        FOREIGN KEY (file_id) REFERENCES afiles(id) ON DELETE CASCADE
                    );
                    CREATE INDEX IF NOT EXISTS idx_acollections_files_file ON acollections_files(file_id);
                    CREATE INDEX IF NOT EXISTS idx_acollections_files_collection_added
                        ON acollections_files(collection_id, added_at DESC, file_id);",
                )
                .map_err(|e| format!("Migration 9 failed creating collections: {}", e))?;

                // --- deduplicate album files ---
                migrate_unique_album_files(conn)?;

                // --- Live Photo pairing columns ---
                if !table_has_column(conn, "afiles", "content_identifier")? {
                    conn.execute("ALTER TABLE afiles ADD COLUMN content_identifier TEXT", [])
                        .map_err(|e| format!("Migration 9 failed adding content_identifier: {}", e))?;
                }
                if !table_has_column(conn, "afiles", "media_subtype")? {
                    conn.execute("ALTER TABLE afiles ADD COLUMN media_subtype TEXT", [])
                        .map_err(|e| format!("Migration 9 failed adding media_subtype: {}", e))?;
                }
                if !table_has_column(conn, "afiles", "live_photo_video_id")? {
                    conn.execute("ALTER TABLE afiles ADD COLUMN live_photo_video_id INTEGER", [])
                        .map_err(|e| format!("Migration 9 failed adding live_photo_video_id: {}", e))?;
                }
                conn.execute(
                    "CREATE INDEX IF NOT EXISTS idx_afiles_content_identifier ON afiles(content_identifier)",
                    [],
                ).map_err(|e| format!("Migration 9 failed adding content_identifier index: {}", e))?;
                conn.execute(
                    "CREATE INDEX IF NOT EXISTS idx_afiles_live_photo_video_id ON afiles(live_photo_video_id)",
                    [],
                ).map_err(|e| format!("Migration 9 failed adding live_photo_video_id index: {}", e))?;

                // --- case-insensitive filename index ---
                conn.execute(
                    "CREATE INDEX IF NOT EXISTS idx_afiles_folder_name_nocase
                     ON afiles(folder_id, name COLLATE NOCASE)",
                    [],
                ).map_err(|e| format!("Migration 9 failed adding case-insensitive index: {}", e))?;

                // --- per-folder scan state ---
                conn.execute_batch(
                    "CREATE TABLE IF NOT EXISTS folder_scan_state (
                        folder_id INTEGER NOT NULL,
                        scanner TEXT NOT NULL,
                        version INTEGER NOT NULL DEFAULT 0,
                        updated_at INTEGER NOT NULL,
                        PRIMARY KEY (folder_id, scanner),
                        FOREIGN KEY (folder_id) REFERENCES afolders(id) ON DELETE CASCADE
                    );
                    CREATE INDEX IF NOT EXISTS idx_folder_scan_state_scanner_version
                        ON folder_scan_state(scanner, version);",
                ).map_err(|e| format!("Migration 9 failed creating folder scan state: {}", e))?;
            } else if migration.version == 11 {
                if !table_has_column(conn, "afiles", "culling_flag")? {
                    conn.execute(
                        "ALTER TABLE afiles ADD COLUMN culling_flag INTEGER NOT NULL DEFAULT 0",
                        [],
                    )
                    .map_err(|e| format!("Migration 11 failed adding culling_flag: {}", e))?;
                }
                conn.execute(
                    "CREATE INDEX IF NOT EXISTS idx_afiles_culling_flag ON afiles(culling_flag)",
                    [],
                )
                .map_err(|e| format!("Migration 11 failed adding culling_flag index: {}", e))?;
            } else if migration.version == 12 {
                if !table_has_column(conn, "afolders", "has_subfolders")? {
                    conn.execute("ALTER TABLE afolders ADD COLUMN has_subfolders INTEGER", [])
                        .map_err(|e| format!("Migration 12 failed adding has_subfolders: {}", e))?;
                }
            } else if migration.version == 13 {
                if !table_has_column(conn, "afolders", "inode")? {
                    conn.execute("ALTER TABLE afolders ADD COLUMN inode INTEGER", [])
                        .map_err(|e| format!("Migration 13 failed adding inode: {}", e))?;
                }
                conn.execute(
                    "CREATE INDEX IF NOT EXISTS idx_afolders_album_inode ON afolders(album_id, inode)",
                    [],
                )
                .map_err(|e| format!("Migration 13 failed creating inode index: {}", e))?;
            } else if migration.version == 14 {
                for column in ["skipped_count", "skipped_size", "failed_count", "failed_size", "merged_count", "merged_size"] {
                    if !table_has_column(conn, "albums", column)? {
                        conn.execute(
                            &format!("ALTER TABLE albums ADD COLUMN {} INTEGER NOT NULL DEFAULT 0", column),
                            [],
                        )
                        .map_err(|e| format!("Migration 14 failed adding {}: {}", column, e))?;
                    }
                }
            } else if migration.version == 15 {
                if !table_has_column(conn, "similarity_group_items", "is_keep")? {
                    conn.execute(
                        "ALTER TABLE similarity_group_items ADD COLUMN is_keep INTEGER NOT NULL DEFAULT 0",
                        [],
                    )
                    .map_err(|e| format!("Migration 15 failed adding is_keep: {}", e))?;
                    conn.execute_batch(
                        "UPDATE similarity_group_items
                         SET is_keep = 1
                         WHERE EXISTS (
                             SELECT 1
                             FROM similarity_groups g
                             WHERE g.id = similarity_group_items.group_id
                               AND g.representative_file_id = similarity_group_items.file_id
                         );",
                    )
                    .map_err(|e| format!("Migration 15 failed initializing keep state: {}", e))?;
                }
            } else if migration.version == 16 {
                if !table_has_column(conn, "afiles", "motion_photo_offset")? {
                    conn.execute("ALTER TABLE afiles ADD COLUMN motion_photo_offset INTEGER", [])
                        .map_err(|e| {
                            format!("Migration 16 failed adding motion_photo_offset: {}", e)
                        })?;
                }
            } else if migration.version == 17 {
                migrate_tag_groups(conn)?;
            } else if migration.version == 18 {
                let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
                for (column, definition) in [
                    ("file_types", "INTEGER NOT NULL DEFAULT 7"),
                    ("small_image_filter", "INTEGER NOT NULL DEFAULT 0"),
                    ("excluded_folders", "TEXT NOT NULL DEFAULT '[]'"),
                ] {
                    if !table_has_column(&tx, "albums", column)? {
                        tx.execute(&format!("ALTER TABLE albums ADD COLUMN {column} {definition}"), [])
                            .map_err(|e| e.to_string())?;
                    }
                }
                tx.commit().map_err(|e| e.to_string())?;
            } else if migration.version == 19 {
                let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
                for column in ["cloud_only", "metadata_pending"] {
                    if !table_has_column(&tx, "afiles", column)? {
                        tx.execute(&format!("ALTER TABLE afiles ADD COLUMN {column} INTEGER NOT NULL DEFAULT 0"), [])
                            .map_err(|e| e.to_string())?;
                    }
                }
                tx.commit().map_err(|e| e.to_string())?;
            } else if migration.version == 20 {
                let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
                for (column, definition) in [("icc_profile", "TEXT"), ("color_space", "TEXT"), ("bit_depth", "INTEGER")] {
                    if !table_has_column(&tx, "afiles", column)? {
                        tx.execute(&format!("ALTER TABLE afiles ADD COLUMN {column} {definition}"), [])
                            .map_err(|e| format!("Migration {} failed adding {column}: {e}", migration.version))?;
                    }
                }
                // Schedule one content read; empty color fields are valid after extraction.
                tx.execute("UPDATE afiles SET metadata_pending=1 WHERE file_type IN (1,3)", [])
                    .map_err(|e| format!("Migration 20 failed marking metadata pending: {e}"))?;
                // Schema, pending markers and version advance together.
                tx.pragma_update(None, "user_version", migration.version).map_err(|e| e.to_string())?;
                tx.commit().map_err(|e| e.to_string())?;
            } else if migration.version == 21 {
                migrate_tag_tree(conn)?;
            } else if !migration.sql.trim().is_empty() {
                conn.execute_batch(migration.sql)
                    .map_err(|e| format!("Migration {} failed: {}", migration.version, e))?;
            }

            new_version = migration.version;
        }
    }

    if new_version > current_version {
        let update_version_sql = format!("PRAGMA user_version = {};", new_version);
        conn.execute_batch(&update_version_sql)
            .map_err(|e| format!("Failed to update user_version: {}", e))?;

        println!("Database successfully migrated to version {}", new_version);
    } else {
        println!("Database is up to date.");
    }

    Ok(())
}

// Idempotent and atomic, including when a previous migration run was interrupted.
pub(crate) fn migrate_tag_groups(conn: &Connection) -> Result<(), String> {
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    tx.execute_batch("CREATE TABLE IF NOT EXISTS atag_groups (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL COLLATE NOCASE UNIQUE,
        sort_order INTEGER NOT NULL DEFAULT 0,
        is_default INTEGER NOT NULL DEFAULT 0 CHECK(is_default IN (0, 1))
    );
    CREATE UNIQUE INDEX IF NOT EXISTS idx_atag_groups_default
        ON atag_groups(is_default) WHERE is_default = 1;
    INSERT INTO atag_groups(name, is_default)
        SELECT 'Default', 1 WHERE NOT EXISTS (SELECT 1 FROM atag_groups WHERE is_default = 1);")
        .map_err(|e| e.to_string())?;
    if !table_has_column(&tx, "atags", "group_id")? {
        tx.execute_batch("ALTER TABLE atags ADD COLUMN group_id INTEGER REFERENCES atag_groups(id);")
            .map_err(|e| e.to_string())?;
    }
    tx.execute_batch("UPDATE atags SET group_id = (SELECT id FROM atag_groups WHERE is_default = 1)
        WHERE group_id IS NULL;
        CREATE INDEX IF NOT EXISTS idx_atags_group_id ON atags(group_id);
        CREATE TRIGGER IF NOT EXISTS protect_default_tag_group_delete BEFORE DELETE ON atag_groups
        WHEN OLD.is_default = 1 BEGIN SELECT RAISE(ABORT, 'Cannot delete Default'); END;
        CREATE TRIGGER IF NOT EXISTS protect_default_tag_group_update BEFORE UPDATE ON atag_groups
        WHEN NEW.is_default != OLD.is_default OR (OLD.is_default = 1 AND NEW.name != OLD.name)
        BEGIN SELECT RAISE(ABORT, 'Cannot rename or replace Default'); END;
        CREATE TRIGGER IF NOT EXISTS assign_default_tag_group AFTER INSERT ON atags
        WHEN NEW.group_id IS NULL BEGIN UPDATE atags
        SET group_id = (SELECT id FROM atag_groups WHERE is_default = 1) WHERE id = NEW.id; END;
        CREATE TRIGGER IF NOT EXISTS require_tag_group BEFORE UPDATE OF group_id ON atags
        WHEN NEW.group_id IS NULL BEGIN SELECT RAISE(ABORT, 'Tag group is required'); END;")
        .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

#[cfg(test)]
mod album_filter_migration_tests {
    use super::*;

    #[test]
    fn image_properties_schedule_one_backfill_without_clearing_other_pending_work() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE atags(id INTEGER PRIMARY KEY,name TEXT NOT NULL UNIQUE);").unwrap();
        conn.execute_batch("CREATE TABLE afiles(id INTEGER PRIMARY KEY, file_type INTEGER, metadata_pending INTEGER NOT NULL DEFAULT 0);
            INSERT INTO afiles VALUES(1,1,0),(2,3,0),(3,2,0),(4,2,1);
            PRAGMA user_version=19;").unwrap();
        check_and_migrate(&conn).unwrap();
        let pending: Vec<i64> = conn.prepare("SELECT metadata_pending FROM afiles ORDER BY id").unwrap()
            .query_map([], |row| row.get(0)).unwrap().map(Result::unwrap).collect();
        assert_eq!(pending, vec![1, 1, 0, 1]);
        conn.execute("UPDATE afiles SET metadata_pending=0", []).unwrap();
        check_and_migrate(&conn).unwrap();
        assert_eq!(conn.query_row("SELECT SUM(metadata_pending) FROM afiles", [], |row| row.get::<_, i64>(0)).unwrap(), 0);
    }

    #[test]
    fn album_filter_migration_is_restartable_and_defaults_to_off() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE atags(id INTEGER PRIMARY KEY,name TEXT NOT NULL UNIQUE);").unwrap();
        conn.execute_batch("CREATE TABLE albums(id INTEGER PRIMARY KEY); CREATE TABLE afiles(id INTEGER PRIMARY KEY, file_type INTEGER); INSERT INTO albums VALUES(1); PRAGMA user_version=17;").unwrap();
        check_and_migrate(&conn).unwrap();
        let settings: (i64, Option<i64>, String) = conn.query_row(
            "SELECT file_types,small_image_filter,excluded_folders FROM albums", [],
            |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?))).unwrap();
        assert_eq!(settings, (7,Some(0),"[]".into()));
        // Simulate migration completing before the version was persisted.
        conn.execute_batch("PRAGMA user_version=17;").unwrap();
        check_and_migrate(&conn).unwrap();
        assert_eq!(conn.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0)).unwrap(),21);
    }

    #[test]
    fn cloud_state_migration_is_restartable_and_defaults_to_local() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE atags(id INTEGER PRIMARY KEY,name TEXT NOT NULL UNIQUE);").unwrap();
        conn.execute_batch("CREATE TABLE afiles(id INTEGER PRIMARY KEY, file_type INTEGER); INSERT INTO afiles(id) VALUES(1); PRAGMA user_version=18;").unwrap();
        check_and_migrate(&conn).unwrap();
        let state: (i64, i64) = conn.query_row("SELECT cloud_only, metadata_pending FROM afiles", [],
            |row| Ok((row.get(0)?, row.get(1)?))).unwrap();
        assert_eq!(state, (0, 0));
        // Simulate migration completing before the version was persisted.
        conn.execute_batch("PRAGMA user_version=18;").unwrap();
        check_and_migrate(&conn).unwrap();
        assert_eq!(conn.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0)).unwrap(), 21);
    }
}


/// Rebuild only the tag table: preserve IDs, spelling and every file association.
pub(crate) fn migrate_tag_tree(conn: &Connection) -> Result<(), String> {
    if table_has_column(conn, "atags", "parent_id")? {
        return crate::t_tags::initialize(conn);
    }
    let foreign_keys: bool = conn
        .pragma_query_value(None, "foreign_keys", |r| r.get(0))
        .map_err(|e| e.to_string())?;
    conn.pragma_update(None, "foreign_keys", false)
        .map_err(|e| e.to_string())?;
    let result = (|| {
        let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
        tx.execute_batch(
            "CREATE TABLE atags_tree (
            id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL,
            parent_id INTEGER REFERENCES atags(id) ON DELETE CASCADE);
            INSERT INTO atags_tree(id,name) SELECT id,name FROM atags;
            UPDATE sqlite_sequence SET seq=MAX(seq,COALESCE(
                (SELECT seq FROM sqlite_sequence WHERE name='atags'),0)) WHERE name='atags_tree';
            DROP TABLE atags;
            ALTER TABLE atags_tree RENAME TO atags;
            DROP TRIGGER IF EXISTS protect_default_tag_group_delete;
            DROP TRIGGER IF EXISTS protect_default_tag_group_update;
            DROP TABLE IF EXISTS atag_groups;",
        )
        .map_err(|e| e.to_string())?;
        crate::t_tags::initialize(&tx)?;
        tx.pragma_update(None, "user_version", 21)
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())
    })();
    conn.pragma_update(None, "foreign_keys", foreign_keys)
        .map_err(|e| e.to_string())?;
    result
}

#[cfg(test)]
mod tag_tree_migration_tests {
    use super::*;

    fn fixture() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;
            CREATE TABLE atag_groups(id INTEGER PRIMARY KEY,name TEXT);
            INSERT INTO atag_groups VALUES(1,'Default'),(2,'Travel');
            CREATE TABLE atags(id INTEGER PRIMARY KEY AUTOINCREMENT,name TEXT NOT NULL UNIQUE,group_id INTEGER REFERENCES atag_groups(id));
            INSERT INTO atags VALUES(10,'Paris',2),(20,'literal|text',1);
            UPDATE sqlite_sequence SET seq=99 WHERE name='atags';
            CREATE TABLE afiles(id INTEGER PRIMARY KEY);INSERT INTO afiles VALUES(1);
            CREATE TABLE afile_tags(file_id INTEGER REFERENCES afiles(id) ON DELETE CASCADE,tag_id INTEGER REFERENCES atags(id) ON DELETE CASCADE,PRIMARY KEY(file_id,tag_id));
            INSERT INTO afile_tags VALUES(1,10),(1,20);
            CREATE TABLE axmp_imports(file_id INTEGER PRIMARY KEY,dirty INTEGER);INSERT INTO axmp_imports VALUES(1,0);
            PRAGMA user_version=20;").unwrap();
        conn
    }

    #[test]
    fn migration_keeps_ids_links_and_literal_names_and_removes_groups() {
        let conn = fixture();
        migrate_tag_tree(&conn).unwrap();
        migrate_tag_tree(&conn).unwrap();
        let tags: Vec<(i64, String, Option<i64>)> = conn
            .prepare("SELECT id,name,parent_id FROM atags ORDER BY id")
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert_eq!(
            tags,
            vec![
                (10, "Paris".into(), None),
                (20, "literal|text".into(), None)
            ]
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM afile_tags", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            2
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE name='atag_groups'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            0
        );
        assert_eq!(
            conn.query_row("PRAGMA foreign_keys", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            conn.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            21
        );
        assert_eq!(
            conn.query_row("SELECT dirty FROM axmp_imports", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert!(!conn
            .prepare("PRAGMA foreign_key_check")
            .unwrap()
            .query([])
            .unwrap()
            .next()
            .unwrap()
            .is_some());
        assert!(conn
            .execute("INSERT INTO atags(name) VALUES('paris')", [])
            .is_err());
        conn.execute("INSERT INTO atags(name,parent_id) VALUES('Paris',20)", [])
            .unwrap();
        assert!(conn.last_insert_rowid() > 99);
    }

    #[test]
    fn failed_migration_rolls_back_and_restores_foreign_keys() {
        let conn = fixture();
        conn.execute("INSERT INTO atags(name,group_id) VALUES('PARIS',1)", [])
            .unwrap();
        assert!(migrate_tag_tree(&conn).is_err());
        assert!(!table_has_column(&conn, "atags", "parent_id").unwrap());
        assert_eq!(
            conn.query_row("PRAGMA foreign_keys", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            conn.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            20
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM afile_tags", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            2
        );
    }
}
