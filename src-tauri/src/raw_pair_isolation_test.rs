// This ignored integration test is run only by scripts/test_raw_pair_isolated.py.
// It exercises production command functions and delete logic, not the WebView IPC transport.
#[tokio::test]
#[ignore = "requires the isolated temporary HOME created by scripts/test_raw_pair_isolated.py"]
async fn raw_pair_isolated_command_workflow() {
    use rusqlite::params;
    use std::path::Path;

    let home = std::fs::canonicalize(std::env::var_os("HOME").expect("isolated HOME required")).unwrap();
    let root = std::fs::canonicalize(std::env::var_os("LAP_RAW_TEST_ROOT").expect("test root required")).unwrap();
    assert_eq!(home, root, "HOME must be exactly the disposable test root");
    assert!(root.starts_with(std::fs::canonicalize(std::env::temp_dir()).unwrap()), "test root must be inside the system temporary directory");
    assert!(root.join(".lap-raw-test-root").is_file(), "test-root marker required");
    let app_dir = t_config::get_app_data_dir().unwrap();
    let cache_dir = t_config::get_app_cache_dir().unwrap();
    assert!(app_dir.starts_with(&root) && cache_dir.starts_with(&root), "application paths must remain inside test root");
    let photos = root.join("fixture-photos");
    std::fs::create_dir(&photos).unwrap();
    for name in ["a.NEF", "a.JPG", "b.NEF", "b.JPG"] {
        std::fs::write(photos.join(name), b"isolated synthetic file, not a real photo").unwrap();
    }

    let db_path = t_storage::get_current_db_path().unwrap();
    assert!(Path::new(&db_path).starts_with(&root), "database must remain in test root");
    t_sqlite::create_db().unwrap();
    {
        let conn = t_sqlite::open_conn().unwrap();
        let dir = photos.to_str().unwrap();
        conn.execute("INSERT INTO albums(id,name,path,indexed,total,file_types,small_image_filter,excluded_folders) VALUES(1,'fixture',?1,4,4,7,0,'[]')", params![dir]).unwrap();
        conn.execute("INSERT INTO afolders(id,album_id,name,path,is_excluded_from_search) VALUES(1,1,'fixture',?1,0)", params![dir]).unwrap();
        for (id, name, kind) in [(1, "a.NEF", 3), (2, "a.JPG", 1), (3, "b.NEF", 3), (4, "b.JPG", 1)] {
            conn.execute("INSERT INTO afiles(id,folder_id,name,size,file_type,width,height,last_scan_time,geo_cc,geo_admin1,geo_name) VALUES(?1,1,?2,32,?3,1000,1000,1,?4,'Shared region','Shared city')", params![id, name, kind, if id == 2 { "US" } else { "CN" }]).unwrap();
        }
        conn.execute("INSERT INTO atags(id,name,group_id) VALUES(11,'shared',(SELECT id FROM atag_groups WHERE is_default=1))", []).unwrap();
        conn.execute("INSERT INTO afile_tags(file_id,tag_id) VALUES (1,11),(2,11)", []).unwrap();
        conn.execute("INSERT INTO acollections(id,name,created_at,updated_at) VALUES(1,'both',1,1),(2,'jpeg only',1,1)", []).unwrap();
        conn.execute("INSERT INTO acollections_files(collection_id,file_id,added_at) VALUES(1,1,1),(1,2,1),(2,2,1)", []).unwrap();
    }
    // Exercise the actual library lock and scan permit concurrently against
    // this isolated SQLite database and its real fixture files. This does not
    // instantiate a Tauri/Wry AppHandle or execute the full index worker.
    let library_id = t_config::current_library_id().unwrap();
    let scan_library = library_id.clone();
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let scan = std::thread::spawn(move || {
        with_current_library(&scan_library, || {
            let _permit = t_utils::test_scan_permit();
            let conn = t_sqlite::open_conn()?;
            let count: i64 = conn.query_row("SELECT COUNT(*) FROM afiles", [], |r| r.get(0)).map_err(|e| e.to_string())?;
            entered_tx.send(count).map_err(|e| e.to_string())?;
            release_rx.recv().map_err(|e| e.to_string())?;
            Ok(())
        })
    });
    assert_eq!(entered_rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap(), 4);
    let result = with_current_library(&library_id, || t_utils::RawFolderMutation::begin(&[photos.to_str().unwrap()]).map(|_| ()));
    assert!(result.is_err(), "mutations must return busy while a scan owns the gate");
    assert!(matches!(FILE_REFRESH_LIBRARY_LOCK.try_write(), Err(std::sync::TryLockError::WouldBlock)), "library switching must wait for the scan");
    assert!(photos.join("a.NEF").exists() && photos.join("a.JPG").exists());
    release_tx.send(()).unwrap();
    scan.join().unwrap().unwrap();
    assert!(FILE_REFRESH_LIBRARY_LOCK.try_write().is_ok());
    let conn = t_sqlite::open_conn().unwrap();
    assert_eq!(conn.query_row("SELECT COUNT(*) FROM afiles", [], |r| r.get::<_, i64>(0)).unwrap(), 4);
    assert_eq!(conn.query_row("SELECT COUNT(*) FROM afile_tags", [], |r| r.get::<_, i64>(0)).unwrap(), 2);
    drop(conn);
    // Switch only disposable libraries under the same write lock as the real
    // switch command. Queued work for the old library must be rejected.
    let second = t_config::add_library("second isolated library").unwrap();
    assert!(AThumb::try_begin_background_task(&library_id, 987654, 512));
    assert!(!AThumb::try_begin_background_task(&library_id, 987654, 512));
    assert!(AThumb::try_begin_background_task(&second.id, 987654, 512), "thumbnail dedup must be scoped by library");
    AThumb::finish_background_task(&library_id, 987654, 512);
    AThumb::finish_background_task(&second.id, 987654, 512);
    {
        let _switch = FILE_REFRESH_LIBRARY_LOCK.write().unwrap();
        t_config::switch_library(&second.id).unwrap();
        t_sqlite::clear_conn_pool();
        t_sqlite::create_db().unwrap();
    }
    // The same helper gates detached thumbnail/embedding stages: queued work
    // from the old library cannot write into the newly selected database.
    let stale_work = with_current_library(&library_id, || {
        t_sqlite::open_conn()?.execute("INSERT INTO albums(id,name,path) VALUES(999,'stale','/wrong')", []).map_err(|e| e.to_string())?;
        Ok(())
    });
    assert!(stale_work.is_err());
    let new_conn = t_sqlite::open_conn().unwrap();
    assert_eq!(new_conn.query_row("SELECT COUNT(*) FROM afiles", [], |r| r.get::<_, i64>(0)).unwrap(), 0);
    assert_eq!(new_conn.query_row("SELECT COUNT(*) FROM albums WHERE id=999", [], |r| r.get::<_, i64>(0)).unwrap(), 0);
    drop(new_conn);
    {
        let _switch = FILE_REFRESH_LIBRARY_LOCK.write().unwrap();
        t_config::switch_library(&library_id).unwrap();
        t_sqlite::clear_conn_pool();
        t_sqlite::create_db().unwrap();
    }
    assert_eq!(t_sqlite::open_conn().unwrap().query_row("SELECT COUNT(*) FROM afile_tags", [], |r| r.get::<_, i64>(0)).unwrap(), 2);

    let pair_snapshot = || -> Vec<(i64, Option<String>, Option<i64>)> {
        let conn = t_sqlite::open_conn().unwrap();
        let mut stmt = conn.prepare("SELECT id, media_subtype, live_photo_video_id FROM afiles ORDER BY id").unwrap();
        stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))).unwrap().collect::<Result<_, _>>().unwrap()
    };
    assert!(backfill_raw_pair_index().await.unwrap());
    let initial_pairs = pair_snapshot();
    assert!(backfill_raw_pair_index().await.unwrap(), "backfill must be idempotent");
    assert_eq!(initial_pairs, pair_snapshot(), "second backfill must preserve links");
    let mut query: QueryParams = serde_json::from_value(serde_json::json!({
        "searchFileName":"", "searchFileType":0, "sortType":0, "sortOrder":0,
        "searchAllSubfolders":"", "searchFolder":"", "startDate":0, "endDate":0,
        "calendarSort":0, "make":"", "model":"", "lensMake":"", "lensModel":"",
        "locationAdmin1":"", "locationName":"", "isFavorite":false, "rating":-1,
        "tagId":0, "personId":0, "groupRawJpegPairs":false
    })).unwrap();
    let mut separate = get_query_file_ids(query.clone()).await.unwrap();
    separate.sort();
    assert_eq!(separate, vec![1, 2, 3, 4]);
    assert_eq!(get_query_count_and_sum(query.clone()).await.unwrap().0, 4);
    query.group_raw_jpeg_pairs = true;
    let mut grouped = get_query_file_ids(query.clone()).await.unwrap();
    grouped.sort();
    assert_eq!(grouped, vec![1, 3]);
    assert_eq!(get_query_count_and_sum(query.clone()).await.unwrap().0, 2);
    assert_eq!(initial_pairs, pair_snapshot(), "query display setting must not mutate pair links");
    assert_eq!(get_collection_query_file_ids(1, query.clone()).unwrap(), vec![1]);
    assert_eq!(get_collection_query_file_ids(2, query.clone()).unwrap(), vec![2]);
    let mut smart: SmartQueryParams = serde_json::from_value(serde_json::json!({"sortType":0,"sortOrder":0,"rules":[{"id":"rating","field":"rating","operator":"eq","value":0}],"groupRawJpegPairs":false})).unwrap();
    assert_eq!(get_smart_query_count_and_sum(smart.clone()).await.unwrap().0, 4);
    smart.group_raw_jpeg_pairs = true;
    assert_eq!(get_smart_query_count_and_sum(smart).await.unwrap().0, 2);
    assert_eq!(resolve_raw_pair_primary_ids(vec![2, 4]).await.unwrap(), HashMap::from([(2, 1), (4, 3)]));

    // Exercise actual statistics command functions, not just SQL fragment builders.
    assert_eq!(get_album_visible_counts(Some(false)).unwrap()[&1], 4);
    assert_eq!(get_album_visible_counts(Some(true)).unwrap()[&1], 2);
    assert_eq!(get_library_visible_counts(None, Some(false)).await.unwrap().all, 4);
    assert_eq!(get_library_visible_counts(None, Some(true)).await.unwrap().all, 2);
    assert_eq!(get_tag_counts(Some(false)).unwrap()[&11], 2);
    assert_eq!(get_tag_counts(Some(true)).unwrap()[&11], 1);
    assert_eq!(get_tag_groups(Some(false)).unwrap().iter().find(|g| g.is_default).unwrap().count, 2);
    assert_eq!(get_tag_groups(Some(true)).unwrap().iter().find(|g| g.is_default).unwrap().count, 1);
    assert_eq!(get_all_tags(0, Some(true)).unwrap().iter().find(|t| t.name == "shared").unwrap().id, 11);
    let separate_collections = get_collection_counts(Some(false)).unwrap();
    let grouped_collections = get_collection_counts(Some(true)).unwrap();
    assert_eq!((separate_collections[&1], grouped_collections[&1]), (2, 1));
    assert_eq!((separate_collections[&2], grouped_collections[&2]), (1, 1));
    let locations = get_location_info(0, Some(true)).unwrap();
    let location_count = |cc: &str| locations.iter().find(|entry| entry.cc == cc).unwrap().counts.iter().sum::<i64>();
    assert_eq!(location_count("CN"), 2);
    assert_eq!(location_count("US"), 1, "same city in another country must remain visible");

    // All deletion paths are derived from fixed filenames inside our verified fixture.
    // Use permanent delete; never touch the OS Trash or any user's library.
    let a_raw = photos.join("a.NEF");
    let a = delete_file_group(1, a_raw.to_str().unwrap(), true, false).unwrap();
    assert_eq!(a.deleted_file_ids, vec![1]);
    assert!(!a_raw.exists());
    assert!(photos.join("a.JPG").exists());
    let b_raw = photos.join("b.NEF");
    let b = delete_file_group(3, b_raw.to_str().unwrap(), true, true).unwrap();
    assert_eq!(b.deleted_file_ids, vec![3, 4]);
    assert!(!b_raw.exists());
    assert!(!photos.join("b.JPG").exists());
    let remaining = get_query_file_ids(serde_json::from_value(serde_json::json!({
        "searchFileName":"", "searchFileType":0, "sortType":0, "sortOrder":0,
        "searchAllSubfolders":"", "searchFolder":"", "startDate":0, "endDate":0,
        "calendarSort":0, "make":"", "model":"", "lensMake":"", "lensModel":"",
        "locationAdmin1":"", "locationName":"", "isFavorite":false, "rating":-1,
        "tagId":0, "personId":0, "groupRawJpegPairs":false
    })).unwrap()).await.unwrap();
    assert_eq!(remaining, vec![2]);

    if let Some(sample_dir) = std::env::var_os("LAP_RAW_SAMPLE_DIR") {
        let sample_dir = std::fs::canonicalize(sample_dir).unwrap();
        assert!(sample_dir.starts_with(&root), "real samples must be copies inside test root");
        let files: Vec<_> = std::fs::read_dir(&sample_dir).unwrap().map(|entry| entry.unwrap().path()).collect();
        assert_eq!(files.len(), 2);
        let raw = files.iter().find(|p| p.extension().is_some_and(|s| s.eq_ignore_ascii_case("nef"))).unwrap();
        let jpeg = files.iter().find(|p| p.extension().is_some_and(|s| s.eq_ignore_ascii_case("jpg") || s.eq_ignore_ascii_case("jpeg"))).unwrap();
        for path in [raw, jpeg] {
            assert!(std::fs::canonicalize(path).unwrap().starts_with(&sample_dir));
        }
        {
            let conn = t_sqlite::open_conn().unwrap();
            let dir = sample_dir.to_str().unwrap();
            conn.execute("INSERT INTO albums(id,name,path,indexed,total,file_types,small_image_filter,excluded_folders) VALUES(2,'real copies',?1,2,2,7,0,'[]')", params![dir]).unwrap();
            conn.execute("INSERT INTO afolders(id,album_id,name,path,is_excluded_from_search) VALUES(2,2,'real copies',?1,0)", params![dir]).unwrap();
        }
        // Production indexing reads metadata from copies only; no AppHandle needed.
        let raw_file = add_file_to_db(2, raw.to_str().unwrap()).unwrap().unwrap();
        let jpeg_file = add_file_to_db(2, jpeg.to_str().unwrap()).unwrap().unwrap();
        let raw_id = raw_file.id.unwrap();
        let jpeg_id = jpeg_file.id.unwrap();
        assert_eq!(raw_file.file_type, Some(3));
        assert_eq!(jpeg_file.file_type, Some(1));
        assert!(backfill_raw_pair_index().await.unwrap());
        assert_eq!(resolve_raw_pair_primary_ids(vec![jpeg_id]).await.unwrap().get(&jpeg_id), Some(&raw_id));
        let actual = AFile::get_file_info(raw_id).unwrap().unwrap();
        assert_eq!(actual.media_subtype.as_deref(), Some("raw_jpeg_pair"));
        assert_eq!(actual.live_photo_video_id, Some(jpeg_id));
        assert!(raw.exists() && jpeg.exists(), "sample copies must remain intact");
    }
}
