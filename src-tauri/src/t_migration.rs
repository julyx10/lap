use rusqlite::Connection;

struct Migration {
    version: i32,
    description: &'static str,
    sql: &'static str,
}

fn get_migrations() -> Vec<Migration> {
    vec![
        // Initial setup is version 0 (implicit).
        // Version 1 example:
        // Migration {
        //     version: 1,
        //     description: "Create initial tables",
        //     sql: "CREATE TABLE IF NOT EXISTS test (id INTEGER PRIMARY KEY);",
        // },
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
