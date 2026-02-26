use rusqlite::Connection;

struct Migration {
    version: i32,
    description: &'static str,
    sql: &'static str,
}

fn get_migrations() -> Vec<Migration> {
    vec![
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
    ]
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
            conn.execute_batch(migration.sql)
                .map_err(|e| format!("Migration {} failed: {}", migration.version, e))?;

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
