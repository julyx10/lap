//! Tag-tree storage shared by catalog operations and XMP import/export.
use rusqlite::{params, Connection, OptionalExtension};

pub const DESCENDANTS: &str = "WITH RECURSIVE descendants(id) AS (
    SELECT id FROM atags WHERE id=?1 UNION ALL
    SELECT t.id FROM atags t JOIN descendants d ON t.parent_id=d.id)";
pub const PATHS: &str = "WITH RECURSIVE paths(id,path) AS (
    SELECT id,name FROM atags WHERE parent_id IS NULL UNION ALL
    SELECT t.id,p.path || '|' || t.name FROM atags t JOIN paths p ON t.parent_id=p.id)";

pub const FILE_PATHS: &str = "WITH RECURSIVE paths(id,name,parent_id,ancestor_id,path,valid) AS (
    SELECT t.id,t.name,t.parent_id,t.parent_id,t.name,instr(t.name,'|')=0
    FROM atags t JOIN afile_tags ft ON ft.tag_id=t.id WHERE ft.file_id=?1 UNION ALL
    SELECT p.id,p.name,p.parent_id,t.parent_id,t.name || '|' || p.path,p.valid AND instr(t.name,'|')=0
    FROM paths p JOIN atags t ON t.id=p.ancestor_id)";

pub fn initialize(conn: &Connection) -> Result<(), String> {
    conn.execute_batch("DROP INDEX IF EXISTS idx_atags_name;
        CREATE INDEX IF NOT EXISTS idx_atags_parent ON atags(parent_id);
        CREATE UNIQUE INDEX IF NOT EXISTS uidx_atags_roots ON atags(name COLLATE NOCASE) WHERE parent_id IS NULL;
        CREATE UNIQUE INDEX IF NOT EXISTS uidx_atags_siblings ON atags(parent_id,name COLLATE NOCASE) WHERE parent_id IS NOT NULL;
        CREATE TABLE IF NOT EXISTS axmp_tag_paths (
            kind INTEGER NOT NULL, path TEXT NOT NULL COLLATE NOCASE,
            tag_id INTEGER REFERENCES atags(id) ON DELETE SET NULL,
            PRIMARY KEY(kind,path));
        CREATE INDEX IF NOT EXISTS idx_axmp_tag_paths_tag_id ON axmp_tag_paths(tag_id);")
        .map_err(|e| e.to_string())
}

pub fn validate_name(name: &str) -> Result<&str, String> {
    let name = name.trim();
    if name.is_empty()
        || name.chars().count() > 255
        || name.contains([',', ';', '|'])
        || name.contains('\0')
    {
        return Err("Invalid tag name: use 1–255 characters, without commas, semicolons or pipes".into());
    }
    Ok(name)
}

pub fn find(conn: &Connection, name: &str, parent: Option<i64>) -> Result<Option<i64>, String> {
    // Explicit predicates let SQLite use the partial root/sibling name indexes.
    let result = match parent {
        Some(parent) => conn.query_row(
            "SELECT id FROM atags WHERE parent_id=?1 AND name=?2 COLLATE NOCASE",
            params![parent, name],
            |r| r.get(0),
        ),
        None => conn.query_row(
            "SELECT id FROM atags WHERE parent_id IS NULL AND name=?1 COLLATE NOCASE",
            [name],
            |r| r.get(0),
        ),
    };
    result.optional().map_err(|e| e.to_string())
}

pub fn ensure_parent(conn: &Connection, parent: Option<i64>) -> Result<(), String> {
    if let Some(id) = parent {
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM atags WHERE id=?1)",
                [id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if !exists {
            return Err("Parent tag not found".into());
        }
    }
    Ok(())
}

pub fn path(conn: &Connection, id: i64) -> Result<String, String> {
    conn.query_row("WITH RECURSIVE ancestors(parent_id,path) AS (
        SELECT parent_id,name FROM atags WHERE id=?1 UNION ALL
        SELECT t.parent_id,t.name || '|' || a.path FROM atags t JOIN ancestors a ON a.parent_id=t.id)
        SELECT path FROM ancestors WHERE parent_id IS NULL", [id], |r| r.get(0))
        .map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct SubtreeInfo {
    pub tag_count: i64,
    pub file_count: i64,
}

pub fn subtree_info(conn: &Connection, id: i64) -> Result<SubtreeInfo, String> {
    conn.query_row(&format!("{DESCENDANTS} SELECT (SELECT COUNT(*) FROM descendants),
        (SELECT COUNT(DISTINCT ft.file_id) FROM afile_tags ft JOIN descendants d ON d.id=ft.tag_id)"),
        [id], |r|Ok(SubtreeInfo { tag_count:r.get(0)?,file_count:r.get(1)? })).map_err(|e|e.to_string())
}

pub fn protect_subtree(conn: &Connection, id: i64) -> Result<(), String> {
    // Renaming/moving an ancestor changes the exported paths of every descendant.
    conn.execute(
        &format!(
            "{DESCENDANTS}
        INSERT INTO axmp_imports(file_id,dirty)
        SELECT DISTINCT ft.file_id,8 FROM afile_tags ft JOIN descendants d ON d.id=ft.tag_id WHERE 1
        ON CONFLICT(file_id) DO UPDATE SET dirty=dirty|8"
        ),
        [id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn move_on(conn: &Connection, id: i64, parent: Option<i64>) -> Result<(), String> {
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    ensure_parent(&tx, parent)?;
    let (name, old_parent): (String, Option<i64>) = tx
        .query_row("SELECT name,parent_id FROM atags WHERE id=?1", [id], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .map_err(|e| e.to_string())?;
    if old_parent == parent {
        return Ok(());
    }
    if let Some(parent) = parent {
        let cycle: bool = tx
            .query_row(
                &format!("{DESCENDANTS} SELECT EXISTS(SELECT 1 FROM descendants WHERE id=?2)"),
                params![id, parent],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if cycle {
            return Err("Cannot move a tag into its own subtree".into());
        }
    }
    if find(&tx, &name, parent)?.is_some() {
        return Err("A tag with this name already exists at the destination".into());
    }
    protect_subtree(&tx, id)?;
    tx.execute(
        "UPDATE atags SET parent_id=?1 WHERE id=?2",
        params![parent, id],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

/// Source aliases survive local rename/move; deleted tags leave a tombstone.
pub fn import_path(
    conn: &Connection,
    value: &str,
    hierarchical: bool,
) -> Result<Option<i64>, String> {
    if value.len() > 16384 || (hierarchical && value.split('|').count() > 128) {
        return Err("Tag hierarchy exceeds the supported size".into());
    }
    let parts: Vec<&str> = if hierarchical {
        value.split('|').map(str::trim).collect()
    } else {
        vec![value.trim()]
    };
    if parts
        .iter()
        .any(|p| p.is_empty() || p.chars().count() > 255 || p.contains('\0'))
    {
        return Ok(None);
    }
    let source = parts.join("|");
    let kind = i32::from(hierarchical);
    let alias: Option<Option<i64>> = conn
        .query_row(
            "SELECT tag_id FROM axmp_tag_paths WHERE kind=?1 AND path=?2 COLLATE NOCASE",
            params![kind, source],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if let Some(id) = alias {
        return Ok(id);
    }
    let mut parent = None;
    let mut prefix = String::new();
    for part in parts {
        if !prefix.is_empty() {
            prefix.push('|');
        }
        prefix.push_str(part);
        let prior: Option<Option<i64>> = if hierarchical {
            conn.query_row(
                "SELECT tag_id FROM axmp_tag_paths WHERE kind=1 AND path=?1 COLLATE NOCASE",
                [&prefix],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
        } else {
            None
        };
        let id = if let Some(prior) = prior {
            match prior {
                Some(id) => id,
                None => return Ok(None),
            }
        } else {
            match find(conn, part, parent)? {
                Some(id) => id,
                None => {
                    conn.execute(
                        "INSERT INTO atags(name,parent_id) VALUES(?1,?2)",
                        params![part, parent],
                    )
                    .map_err(|e| e.to_string())?;
                    conn.last_insert_rowid()
                }
            }
        };
        parent = Some(id);
        if hierarchical {
            conn.execute(
                "INSERT OR IGNORE INTO axmp_tag_paths(kind,path,tag_id) VALUES(1,?1,?2)",
                params![prefix, id],
            )
            .map_err(|e| e.to_string())?;
        }
    }
    conn.execute(
        "INSERT OR IGNORE INTO axmp_tag_paths(kind,path,tag_id) VALUES(?1,?2,?3)",
        params![kind, source, parent],
    )
    .map_err(|e| e.to_string())?;
    Ok(parent)
}

pub fn keywords_for_file(conn: &Connection, id: i64) -> Result<(Vec<String>, Vec<String>), String> {
    let sql = format!("{FILE_PATHS} SELECT name,path,valid FROM paths WHERE ancestor_id IS NULL ORDER BY path COLLATE NOCASE,id");
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, bool>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    let mut flat = Vec::new();
    let mut paths = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for row in rows {
        let (name, path, valid) = row.map_err(|e| e.to_string())?;
        if seen.insert(name.to_lowercase()) {
            flat.push(name);
        }
        if valid {
            paths.push(path);
        }
    }
    Ok((flat, paths))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_reject_delimiters_and_allow_asterisks() {
        for name in ["", " ", "\t ", "a,b", "a;b", "a|b", "bad\0name"] {
            assert!(validate_name(name).is_err(), "{name:?}");
        }
        for name in ["a/b", r"a\b", "two words", "a*b", "*tag", "tag*", "*", "旅行"] {
            assert_eq!(validate_name(name).unwrap(), name);
        }
        assert_eq!(validate_name(" \ttwo words\t ").unwrap(), "two words");
        assert!(validate_name(&"a".repeat(255)).is_ok());
        assert!(validate_name(&"a".repeat(256)).is_err());
    }

    fn fixture() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;
            CREATE TABLE atags(id INTEGER PRIMARY KEY,name TEXT NOT NULL,parent_id INTEGER REFERENCES atags(id) ON DELETE CASCADE);
            CREATE TABLE afiles(id INTEGER PRIMARY KEY,name TEXT,rating INTEGER,culling_flag INTEGER,comments TEXT,is_favorite INTEGER,has_tags INTEGER);
            INSERT INTO afiles VALUES(1,'test.jpg',0,0,NULL,0,0),(2,'other.jpg',0,0,NULL,0,0);
            CREATE TABLE afile_tags(file_id INTEGER REFERENCES afiles(id) ON DELETE CASCADE,tag_id INTEGER REFERENCES atags(id) ON DELETE CASCADE,PRIMARY KEY(file_id,tag_id));").unwrap();
        crate::t_xmp::initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn initialization_removes_legacy_index_and_import_rejects_nul_paths() {
        let conn = fixture();
        conn.execute("CREATE INDEX idx_atags_name ON atags(name)", []).unwrap();
        initialize(&conn).unwrap();
        initialize(&conn).unwrap();
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM sqlite_master WHERE name='idx_atags_name'", [], |r| r.get::<_, i64>(0)).unwrap(), 0);
        assert_eq!(import_path(&conn, "bad\0name", false).unwrap(), None);
        assert_eq!(import_path(&conn, "Places|bad\0child", true).unwrap(), None);
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM atags", [], |r| r.get::<_, i64>(0)).unwrap(), 0);
    }

    #[test]
    fn hierarchy_identity_siblings_cycles_and_collisions() {
        let conn = fixture();
        let paris = import_path(&conn, "Places|France|Paris", true)
            .unwrap()
            .unwrap();
        let other = import_path(&conn, "Clients|Paris", true).unwrap().unwrap();
        assert_ne!(paris, other);
        assert_eq!(
            import_path(&conn, "places|france|PARIS", true).unwrap(),
            Some(paris)
        );
        assert!(conn
            .execute("INSERT INTO atags(name) VALUES('places')", [])
            .is_err());
        let root = find(&conn, "Places", None).unwrap().unwrap();
        assert!(move_on(&conn, root, Some(paris)).is_err());
        let clients = find(&conn, "Clients", None).unwrap().unwrap();
        assert!(move_on(&conn, paris, Some(clients)).is_err());
        assert_eq!(path(&conn, paris).unwrap(), "Places|France|Paris");
        move_on(&conn, paris, None).unwrap();
        assert_eq!(path(&conn, paris).unwrap(), "Paris");
    }

    #[test]
    fn aliases_follow_ancestor_edits_new_children_and_deletions() {
        let conn = fixture();
        let city = import_path(&conn, "Places|France|Paris", true)
            .unwrap()
            .unwrap();
        conn.execute("INSERT INTO afile_tags VALUES(1,?1)", [city])
            .unwrap();
        let root = find(&conn, "Places", None).unwrap().unwrap();
        conn.execute(
            "INSERT INTO afile_tags VALUES(1,?1),(2,?2)",
            params![root, city],
        )
        .unwrap();
        let info = subtree_info(&conn, root).unwrap();
        assert_eq!((info.tag_count, info.file_count), (3, 2));
        conn.execute(
            "DELETE FROM afile_tags WHERE file_id=2 OR tag_id=?1",
            [root],
        )
        .unwrap();
        conn.execute("UPDATE axmp_imports SET dirty=0", []).unwrap();
        let country = find(&conn, "France", find(&conn, "Places", None).unwrap())
            .unwrap()
            .unwrap();
        conn.execute(
            "UPDATE atags SET name='French places' WHERE id=?1",
            [country],
        )
        .unwrap();
        assert_eq!(
            conn.query_row("SELECT dirty FROM axmp_imports WHERE file_id=1", [], |r| {
                r.get::<_, i64>(0)
            })
            .unwrap()
                & 8,
            8
        );
        assert_eq!(
            import_path(&conn, "Places|France|Paris", true).unwrap(),
            Some(city)
        );
        let lyon = import_path(&conn, "Places|France|Lyon", true)
            .unwrap()
            .unwrap();
        assert_eq!(path(&conn, lyon).unwrap(), "Places|French places|Lyon");
        let (flat, paths) = keywords_for_file(&conn, 1).unwrap();
        assert_eq!(flat, vec!["Paris"]);
        assert_eq!(paths, vec!["Places|French places|Paris"]);
        conn.execute("DELETE FROM atags WHERE id=?1", [country])
            .unwrap();
        assert_eq!(
            import_path(&conn, "Places|France|Paris", true).unwrap(),
            None
        );
        assert_eq!(
            import_path(&conn, "Places|France|New city", true).unwrap(),
            None
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM afile_tags", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn flat_pipe_text_remains_flat_and_export_does_not_invent_its_hierarchy() {
        let conn = fixture();
        let flat = import_path(&conn, "literal|word", false).unwrap().unwrap();
        conn.execute("INSERT INTO afile_tags VALUES(1,?1)", [flat])
            .unwrap();
        assert_eq!(find(&conn, "literal", None).unwrap(), None);
        assert_eq!(
            keywords_for_file(&conn, 1).unwrap(),
            (vec!["literal|word".into()], vec![])
        );
        assert!(validate_name("bad|name").is_err());
        assert!(import_path(&conn, &"a|".repeat(129), true).is_err());
    }
}
