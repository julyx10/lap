//! Count distinct failed files only for the lifetime of an album scan.
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock};

type Key = (String, i64);
type Counts = HashMap<Key, Arc<Mutex<HashSet<i64>>>>;
fn counts() -> &'static Mutex<Counts> {
    static COUNTS: OnceLock<Mutex<Counts>> = OnceLock::new();
    COUNTS.get_or_init(Default::default)
}
fn database(conn: &Connection) -> String {
    conn.query_row("PRAGMA database_list", [], |r| r.get(2)).unwrap_or_default()
}
pub(crate) struct ScanFailures {
    key: Key,
    files: Arc<Mutex<HashSet<i64>>>,
}
impl ScanFailures {
    pub fn start(conn: &Connection, album: i64) -> Self {
        let key = (database(conn), album);
        let files = Arc::new(Mutex::new(HashSet::new()));
        if let Ok(mut counts) = counts().lock() { counts.insert(key.clone(), files.clone()); }
        Self { key, files }
    }
    pub fn count(&self) -> u64 {
        self.files.lock().map(|files| files.len() as u64).unwrap_or(0)
    }
}
impl Drop for ScanFailures {
    fn drop(&mut self) {
        if let Ok(mut counts) = counts().lock() {
            if counts.get(&self.key).is_some_and(|files| Arc::ptr_eq(files, &self.files)) { counts.remove(&self.key); }
        }
    }
}
pub(super) fn record(conn: &Connection, file: i64) {
    if counts().lock().map(|counts| counts.is_empty()).unwrap_or(true) { return; }
    let Ok(album) = conn.query_row("SELECT f.album_id FROM afiles a JOIN afolders f ON f.id=a.folder_id WHERE a.id=?1", [file], |r| r.get::<_, i64>(0)) else { return };
    let key = (database(conn), album);
    let files = counts().lock().ok().and_then(|counts| counts.get(&key).cloned());
    if let Some(files) = files { if let Ok(mut files) = files.lock() { files.insert(file); } }
}
