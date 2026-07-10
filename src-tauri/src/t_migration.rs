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
            version: 6,
            description: "Post v0.2.4 schema updates",
            sql: "",
        },
    ]
}

fn migrate_unique_album_files(conn: &Connection) -> Result<(), String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Migration 6 (deduplicate album files) failed starting transaction: {}", e))?;

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
    .map_err(|e| format!("Migration 6 (deduplicate album files) failed: {}", e))?;

    tx.commit()
        .map_err(|e| format!("Migration 6 (deduplicate album files) failed committing transaction: {}", e))
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
            } else if migration.version == 6 {
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
                .map_err(|e| format!("Migration 6 failed creating collections: {}", e))?;

                // --- deduplicate album files ---
                migrate_unique_album_files(conn)?;

                // --- Live Photo pairing columns ---
                if !table_has_column(conn, "afiles", "content_identifier")? {
                    conn.execute("ALTER TABLE afiles ADD COLUMN content_identifier TEXT", [])
                        .map_err(|e| format!("Migration 6 failed adding content_identifier: {}", e))?;
                }
                if !table_has_column(conn, "afiles", "media_subtype")? {
                    conn.execute("ALTER TABLE afiles ADD COLUMN media_subtype TEXT", [])
                        .map_err(|e| format!("Migration 6 failed adding media_subtype: {}", e))?;
                }
                if !table_has_column(conn, "afiles", "live_photo_video_id")? {
                    conn.execute("ALTER TABLE afiles ADD COLUMN live_photo_video_id INTEGER", [])
                        .map_err(|e| format!("Migration 6 failed adding live_photo_video_id: {}", e))?;
                }
                conn.execute(
                    "CREATE INDEX IF NOT EXISTS idx_afiles_content_identifier ON afiles(content_identifier)",
                    [],
                ).map_err(|e| format!("Migration 6 failed adding content_identifier index: {}", e))?;
                conn.execute(
                    "CREATE INDEX IF NOT EXISTS idx_afiles_live_photo_video_id ON afiles(live_photo_video_id)",
                    [],
                ).map_err(|e| format!("Migration 6 failed adding live_photo_video_id index: {}", e))?;

                // --- case-insensitive filename index ---
                conn.execute(
                    "CREATE INDEX IF NOT EXISTS idx_afiles_folder_name_nocase
                     ON afiles(folder_id, name COLLATE NOCASE)",
                    [],
                ).map_err(|e| format!("Migration 6 failed adding case-insensitive index: {}", e))?;

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
                ).map_err(|e| format!("Migration 6 failed creating folder scan state: {}", e))?;
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
