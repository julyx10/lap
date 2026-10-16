use crate::t_sqlite::{ATag, open_conn};
use rusqlite::{Connection, params};
use serde::Serialize;

#[derive(Serialize)]
pub struct TagGroup {
    pub id: i64,
    pub name: String,
    pub is_default: bool,
    pub count: i64,
}

pub fn get_all() -> Result<Vec<TagGroup>, String> {
    let counts = ATag::get_group_counts()?;
    let conn = open_conn()?;
    let mut stmt = conn.prepare("SELECT id, name, is_default FROM atag_groups ORDER BY is_default DESC, sort_order, name COLLATE NOCASE, id").map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            let id = row.get(0)?;
            Ok(TagGroup {
                id,
                name: row.get(1)?,
                is_default: row.get(2)?,
                count: *counts.get(&id).unwrap_or(&0),
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn get_name(id: i64) -> Result<String, String> {
    open_conn()?
        .query_row("SELECT name FROM atag_groups WHERE id = ?", [id], |row| {
            row.get(0)
        })
        .map_err(|e| e.to_string())
}

pub fn save(id: Option<i64>, name: &str) -> Result<i64, String> {
    save_on(&*open_conn()?, id, name)
}

fn save_on(conn: &Connection, id: Option<i64>, name: &str) -> Result<i64, String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 255 {
        return Err("Group name must contain 1–255 characters".into());
    }
    match id {
        Some(id) => {
            let changed = conn
                .execute(
                    "UPDATE atag_groups SET name = ?1 WHERE id = ?2",
                    params![name, id],
                )
                .map_err(|e| e.to_string())?;
            if changed == 0 {
                return Err("Group no longer exists".into());
            }
            Ok(id)
        }
        None => {
            conn.execute("INSERT INTO atag_groups(name, sort_order) SELECT ?1, COALESCE(MAX(sort_order), -1) + 1 FROM atag_groups", params![name])
                .map_err(|e| e.to_string())?;
            Ok(conn.last_insert_rowid())
        }
    }
}

pub fn reorder(ids: &[i64]) -> Result<(), String> {
    reorder_on(&*open_conn()?, ids)
}

fn reorder_on(conn: &Connection, ids: &[i64]) -> Result<(), String> {
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let default_id: i64 = tx
        .query_row("SELECT id FROM atag_groups WHERE is_default = 1", [], |r| {
            r.get(0)
        })
        .map_err(|e| e.to_string())?;
    let count: i64 = tx
        .query_row("SELECT COUNT(*) FROM atag_groups", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let unique: std::collections::HashSet<i64> = ids.iter().copied().collect();
    if ids.first() != Some(&default_id) || unique.len() != ids.len() || ids.len() as i64 != count {
        return Err(
            "Invalid group order; Default must remain first and all groups must be included once"
                .into(),
        );
    }
    for (position, id) in ids.iter().enumerate() {
        if tx
            .execute(
                "UPDATE atag_groups SET sort_order = ?1 WHERE id = ?2",
                params![position as i64, id],
            )
            .map_err(|e| e.to_string())?
            != 1
        {
            return Err("Group no longer exists".into());
        }
    }
    tx.commit().map_err(|e| e.to_string())
}

pub fn delete(id: i64) -> Result<(), String> {
    delete_on(&*open_conn()?, id)
}

fn delete_on(conn: &Connection, id: i64) -> Result<(), String> {
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let is_default: bool = tx
        .query_row(
            "SELECT is_default FROM atag_groups WHERE id = ?",
            [id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if is_default {
        return Err("Cannot delete Default".into());
    }
    tx.execute("UPDATE atags SET group_id = (SELECT id FROM atag_groups WHERE is_default = 1) WHERE group_id = ?", [id]).map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM atag_groups WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

pub fn move_tags(tag_ids: &[i64], group_id: i64) -> Result<(), String> {
    move_on(&*open_conn()?, tag_ids, group_id)
}

fn move_on(conn: &Connection, tag_ids: &[i64], group_id: i64) -> Result<(), String> {
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let exists: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM atag_groups WHERE id = ?)",
            [group_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if !exists {
        return Err("Group no longer exists".into());
    }
    for id in tag_ids {
        if tx
            .execute(
                "UPDATE atags SET group_id = ?1 WHERE id = ?2",
                params![group_id, id],
            )
            .map_err(|e| e.to_string())?
            == 0
        {
            return Err("Tag no longer exists".into());
        }
    }
    tx.commit().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn database() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;
            CREATE TABLE atags(id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE);
            CREATE TABLE afile_tags(file_id INTEGER, tag_id INTEGER REFERENCES atags(id), PRIMARY KEY(file_id, tag_id));
            INSERT INTO atags VALUES (10, 'old'), (11, 'second');
            INSERT INTO afile_tags VALUES(100, 10), (100, 11);").unwrap();
        crate::t_migration::migrate_tag_groups(&conn).unwrap();
        conn
    }
    #[test]
    fn migration_preserves_tags_and_is_repeatable() {
        let conn = database();
        crate::t_migration::migrate_tag_groups(&conn).unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM atags t JOIN atag_groups g ON g.id = t.group_id WHERE g.is_default = 1", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 2);
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM afile_tags", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            2
        );
    }
    #[test]
    fn move_and_delete_preserve_associations_and_protect_default() {
        let conn = database();
        let id = save_on(&conn, None, "Style").unwrap();
        move_on(&conn, &[10, 11], id).unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(DISTINCT ft.file_id) FROM afile_tags ft JOIN atags t ON t.id = ft.tag_id WHERE t.group_id = ?", [id], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
        assert!(move_on(&conn, &[10], 999).is_err());
        delete_on(&conn, id).unwrap();
        let default_id: i64 = conn
            .query_row("SELECT id FROM atag_groups WHERE is_default = 1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert!(delete_on(&conn, default_id).is_err());
        assert!(save_on(&conn, Some(default_id), "Renamed").is_err());
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM atags WHERE group_id = ?",
                [default_id],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            2
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM afile_tags", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            2
        );
    }
    #[test]
    fn failed_batch_move_rolls_back_and_duplicate_names_fail() {
        let conn = database();
        let id = save_on(&conn, None, "Style").unwrap();
        assert!(save_on(&conn, None, "style").is_err());
        assert!(move_on(&conn, &[10, 999], id).is_err());
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM atags WHERE group_id = ?", [id], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
    #[test]
    fn reorder_persists_and_rejects_invalid_lists() {
        let conn = database();
        let a = save_on(&conn, None, "A").unwrap();
        let b = save_on(&conn, None, "B").unwrap();
        let default: i64 = conn
            .query_row("SELECT id FROM atag_groups WHERE is_default = 1", [], |r| {
                r.get(0)
            })
            .unwrap();
        reorder_on(&conn, &[default, b, a]).unwrap();
        assert!(reorder_on(&conn, &[a, default, b]).is_err());
        assert!(reorder_on(&conn, &[default, a, a]).is_err());
        assert!(reorder_on(&conn, &[default, a]).is_err());
        assert!(reorder_on(&conn, &[default, a, 999]).is_err());
        let ids = conn
            .prepare("SELECT id FROM atag_groups ORDER BY is_default DESC, sort_order")
            .unwrap()
            .query_map([], |r| r.get::<_, i64>(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(ids, vec![default, b, a]);
    }

    #[test]
    fn version_17_creates_groups_with_persistent_ordering() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE albums(id INTEGER PRIMARY KEY); CREATE TABLE atags(id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE); PRAGMA user_version = 16;").unwrap();
        crate::t_migration::check_and_migrate(&conn).unwrap();
        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 19);
        let a = save_on(&conn, None, "A").unwrap();
        let b = save_on(&conn, None, "B").unwrap();
        let default: i64 = conn
            .query_row("SELECT id FROM atag_groups WHERE is_default = 1", [], |r| {
                r.get(0)
            })
            .unwrap();
        reorder_on(&conn, &[default, b, a]).unwrap();
        crate::t_migration::check_and_migrate(&conn).unwrap();
        let ids = conn
            .prepare("SELECT id FROM atag_groups ORDER BY sort_order")
            .unwrap()
            .query_map([], |r| r.get::<_, i64>(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(ids, vec![default, b, a]);
    }
}
