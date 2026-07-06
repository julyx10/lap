// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

/**
 * Main entry point for the Tauri application.
 * project: Lap
 * author:  julyx10
 * date:    2024-08-08
 */
use tauri::Manager;
use tauri_plugin_aptabase::EventTracker;

mod t_ai;
mod t_cluster;
mod t_cmds;
mod t_common;
mod t_config;
mod t_dedup;
mod t_face;
#[cfg(all(not(target_os = "macos"), lap_has_libheif))]
mod t_heif;
mod t_http;
mod t_image;
mod t_jpeg;
mod t_jxl;
mod t_lens;
mod t_libraw;
mod t_menu;
mod t_migration;
mod t_pasteboard;
mod t_protocol;
mod t_sqlite;
mod t_storage;
mod t_utils;
mod t_video;

/// The main function is the entry point for the Tauri application.
#[tokio::main]
async fn main() {
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("Unhandled panic: {}", panic_info);
    }));

    let builder = tauri::Builder::default();
    let builder = t_protocol::register_protocols(builder);

    let aptabase_enabled = option_env!("APTABASE_KEY")
        .filter(|k| !k.is_empty())
        .is_some();

    let builder = match option_env!("APTABASE_KEY").filter(|k| !k.is_empty()) {
        Some(key) => builder.plugin(tauri_plugin_aptabase::Builder::new(key).build()),
        None => builder,
    };

    let run_result = builder
        .plugin(tauri_plugin_window_state::Builder::default().build()) // macOS: ~/Library/Application Support/{APP_NAME}/window-state.json
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(t_video::VideoManager::default())
        .manage(t_ai::AiState(std::sync::Mutex::new(t_ai::AiEngine::new())))
        .manage(t_face::FaceState(std::sync::Arc::new(
            std::sync::Mutex::new(t_face::FaceEngine::new()),
        )))
        .manage(t_cmds::IndexCancellation(std::sync::Arc::new(
            std::sync::Mutex::new(std::collections::HashMap::new()),
        )))
        .manage(t_face::FaceIndexCancellation(std::sync::Arc::new(
            std::sync::Mutex::new(false),
        )))
        .manage(t_face::FaceIndexingStatus(std::sync::Arc::new(
            std::sync::Mutex::new(false),
        )))
        .manage(t_face::FaceIndexProgressState(std::sync::Arc::new(
            std::sync::Mutex::new(t_face::FaceIndexProgress {
                current: 0,
                total: 0,
                faces_found: 0,
                phase: "indexing".to_string(),
            }),
        )))
        .manage(t_dedup::DedupState::default())
        .setup(|_app| {
            t_video::init_ffmpeg_path(&_app.handle());
            t_config::set_app_identifier(&_app.config().identifier);
            t_menu::install_app_menu(&_app.handle())?;

            #[cfg(not(target_os = "macos"))]
            if let Some(window) = _app.get_webview_window("main") {
                if let Err(e) = window.set_decorations(false) {
                    eprintln!("Failed to disable main window decorations: {}", e);
                }
            }

            // tauri.windows.conf.json sets zoomHotkeysEnabled=true so wry sets
            // both IsZoomControlEnabled and IsPinchZoomEnabled to true at WebView
            // creation. That combination is what allows Chromium to synthesize
            // wheel+ctrlKey events for touchpad pinch. Touchscreen pinch is still
            // handled by our pointer-event logic (with `touch-action: none`).

            // Create the database on startup
            if let Err(e) = t_sqlite::create_db() {
                eprintln!("Failed to initialize database: {}", e);
            }

            // Initialize video HTTP server for Linux
            #[cfg(target_os = "linux")]
            t_http::init_video_http_server();

            // Cleanup video cache
            t_video::init_video_cache(&_app.handle());

            if let Err(e) = t_utils::restore_album_scopes(&_app.handle()) {
                eprintln!("Failed to restore asset scopes: {}", e);
            }

            // Initialize AI Engine
            let app_handle = _app.handle();
            let ai_state = _app.state::<t_ai::AiState>();
            let mut ai_engine = ai_state.0.lock().unwrap();
            match ai_engine.load_models(app_handle) {
                Ok(_) => println!("AI Engine started successfully"),
                Err(e) => {
                    eprintln!("Failed to start AI Engine: {}", e);
                    #[cfg(target_os = "windows")]
                    {
                        let arch_key = if cfg!(target_arch = "aarch64") {
                            r"HKLM\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\ARM64"
                        } else {
                            r"HKLM\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\X64"
                        };
                        let result = std::process::Command::new("reg")
                            .args(["query", arch_key, "/v", "Installed"])
                            .stdout(std::process::Stdio::null())
                            .status();
                        let installed = result.is_ok() && result.unwrap().success();
                        if !installed {
                            let arch = if cfg!(target_arch = "aarch64") { "arm64" } else { "x64" };
                            let url = format!("https://aka.ms/vs/17/release/vc_redist.{}.exe", arch);
                            let _ = std::process::Command::new("powershell")
                                .args(["-NoProfile", "-Command", &format!(
                                    r#"$wsh = New-Object -ComObject Wscript.Shell; $wsh.Popup('Lap requires the Microsoft Visual C++ Redistributable.`n`nA download page will open in your browser.`nPlease install it, then restart Lap.', 0, 'Lap - Missing Dependency', 0x30); Start-Process '{}'"#,
                                    url
                                )])
                                .stdout(std::process::Stdio::null())
                                .stderr(std::process::Stdio::null())
                                .status();
                            std::process::exit(1);
                        }
                    }
                }
            }

            t_utils::start_folder_mtime_sync(_app.handle().clone());

            // Open devtools in development mode
            // #[cfg(debug_assertions)] // only include this block in debug builds
            // {
            //     let window = app.get_webview_window("main").unwrap();
            //     window.open_devtools();
            // }

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() != "main" {
                return;
            }

            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Prevent the default close so we can decide what to do per platform.
                api.prevent_close();

                // macOS convention: closing the window hides it; the app stays
                // alive and can be reopened via the Dock icon (handled by
                // RunEvent::Reopen). Real quit keeps using the native Quit menu.
                #[cfg(target_os = "macos")]
                {
                    let _ = window.hide();
                }

                #[cfg(not(target_os = "macos"))]
                {
                    let app_handle = window.app_handle();

                    // Close every other window first (image viewer, settings, ...),
                    // then exit the process.
                    let windows = app_handle.webview_windows();
                    for (_, other_window) in windows {
                        if other_window.label() != "main" {
                            if let Err(err) = other_window.close() {
                                eprintln!("Failed to close window: {}", err);
                            }
                        }
                    }

                    app_handle.exit(0);
                }
            }
        })
        .on_menu_event(|app, event| {
            t_menu::handle_menu_event(app, event);
        })
        .invoke_handler(tauri::generate_handler![
            // library
            t_cmds::get_app_config,
            t_cmds::get_supported_format_extensions,
            t_cmds::set_last_selected_item_index,
            t_cmds::get_db_storage_dir,
            t_cmds::is_using_custom_db_storage,
            t_cmds::change_db_storage_dir,
            t_cmds::reset_db_storage_dir,
            t_cmds::add_library,
            t_cmds::edit_library,
            t_cmds::remove_library,
            t_cmds::hide_library,
            t_cmds::reorder_libraries,
            t_cmds::switch_library,
            t_cmds::get_library_info,
            t_cmds::save_library_state,
            t_cmds::get_library_state,
            t_cmds::get_current_library_state,
            // album
            t_cmds::get_all_albums,
            t_cmds::generate_directory_thumbnails,
            t_cmds::get_album,
            t_cmds::recount_album,
            t_cmds::add_album,
            t_cmds::edit_album,
            t_cmds::remove_album,
            t_cmds::set_album_display_order,
            t_cmds::set_album_cover,
            t_cmds::index_album,
            t_cmds::cancel_indexing,
            t_cmds::get_index_recovery_info,
            t_cmds::clear_index_recovery_info,
            // folder
            t_cmds::select_folder,
            t_cmds::fetch_folder,
            t_cmds::count_folder,
            t_cmds::create_folder,
            t_cmds::rename_folder,
            t_cmds::move_folder,
            t_cmds::move_folder_outside_library,
            t_cmds::copy_folder,
            t_cmds::delete_folder,
            t_cmds::delete_folder_permanently,
            t_cmds::reveal_path,
            t_cmds::open_external_url,
            t_cmds::get_external_app_display_name,
            t_cmds::open_file_with_app,
            t_cmds::open_files_with_app,
            // file query
            t_cmds::get_total_count_and_sum,
            t_cmds::get_query_count_and_sum,
            t_cmds::get_query_time_line,
            t_cmds::get_query_files,
            t_cmds::get_grouped_query_rows,
            t_cmds::get_group_file_ids,
            t_cmds::get_query_file_ids,
            t_cmds::get_query_file_position,
            // smart album
            t_cmds::get_smart_query_count_and_sum,
            t_cmds::get_smart_query_time_line,
            t_cmds::get_smart_query_files,
            t_cmds::get_smart_grouped_query_rows,
            t_cmds::get_smart_group_file_ids,
            t_cmds::get_smart_query_file_ids,
            t_cmds::get_smart_query_file_position,
            // collection
            t_cmds::list_collections,
            t_cmds::create_collection,
            t_cmds::rename_collection,
            t_cmds::delete_collection,
            t_cmds::reorder_collections,
            t_cmds::add_files_to_collection,
            t_cmds::remove_files_from_collection,
            t_cmds::clear_collection,
            t_cmds::get_collection_file_ids,
            t_cmds::get_collection_count_and_sum,
            t_cmds::get_collection_files,
            // folder file query
            t_cmds::get_folder_files,
            t_cmds::sync_album_folder_mtimes,
            t_cmds::is_directory_accessible,
            t_cmds::get_folder_thumb_count,
            // file operations
            t_cmds::edit_image,
            t_cmds::copy_edited_image,
            t_cmds::copy_images,
            t_cmds::rename_file,
            t_cmds::move_file,
            t_cmds::move_file_outside_library,
            t_cmds::copy_file,
            t_cmds::import_file,
            t_cmds::import_url,
            t_cmds::import_from_drag,
            t_cmds::get_drag_payload,
            t_cmds::import_file_bytes,
            t_cmds::has_importable_clipboard,
            t_cmds::import_clipboard,
            t_cmds::delete_file,
            t_cmds::delete_file_permanently,
            t_cmds::delete_db_file,
            t_cmds::batch_delete_files,
            // file metadata
            t_cmds::edit_file_comment,
            t_cmds::get_file_thumb,
            t_cmds::get_file_thumb_by_id,
            t_cmds::get_file_thumbs,
            t_cmds::get_file_info,
            t_cmds::update_file_info,
            t_cmds::add_file_to_db,
            t_cmds::check_file_exists,
            t_cmds::set_file_rotate,
            t_cmds::get_file_has_tags,
            // favorite
            t_cmds::get_favorite_folders,
            t_cmds::get_folder_favorite,
            t_cmds::set_folder_favorite,
            t_cmds::get_folder_search_excluded,
            t_cmds::set_folder_search_excluded,
            t_cmds::set_file_favorite,
            t_cmds::set_file_rating,
            t_cmds::batch_update_file_metadata,
            // tag
            t_cmds::get_all_tags,
            t_cmds::get_tag_name,
            t_cmds::create_tag,
            t_cmds::rename_tag,
            t_cmds::delete_tag,
            t_cmds::get_tags_for_file,
            t_cmds::add_tag_to_file,
            t_cmds::remove_tag_from_file,
            t_cmds::get_tag_selection_counts,
            t_cmds::apply_tags_to_files,
            // calendar
            t_cmds::get_taken_dates,
            // camera
            t_cmds::get_camera_info,
            t_cmds::get_lens_info,
            // location
            t_cmds::get_location_info,
            t_cmds::get_gps_heatmap_points,
            // settings
            t_cmds::get_package_info,
            t_cmds::get_build_time,
            t_cmds::get_storage_file_info,
            // ai
            t_cmds::check_ai_status,
            t_cmds::get_image_search_model_status,
            t_cmds::set_image_search_model,
            t_cmds::download_multilingual_image_search_model,
            t_cmds::cancel_multilingual_image_search_model_download,
            t_cmds::generate_embedding,
            t_cmds::search_similar_images,
            // person (face recognition)
            t_cmds::index_faces,
            t_cmds::cancel_face_index,
            t_cmds::reset_faces,
            t_cmds::is_face_indexing,
            t_cmds::get_face_stats,
            t_cmds::get_persons,
            t_cmds::rename_person,
            t_cmds::delete_person,
            t_cmds::get_faces_for_file,
            // dedup
            t_cmds::dedup_start_scan,
            t_cmds::dedup_get_scan_status,
            t_cmds::dedup_cancel_scan,
            t_cmds::dedup_list_groups,
            t_cmds::dedup_get_overview,
            t_cmds::dedup_set_keep,
            t_cmds::dedup_delete_selected,
            // video
            t_video::prepare_video,
            t_video::cancel_video_prepare,
            t_video::clear_video_cache,
            // backup / restore
            t_cmds::get_db_storage_info,
            t_cmds::backup_databases,
            t_cmds::parse_backup_file,
            t_cmds::restore_databases,
        ])
        .build(tauri::generate_context!());

    match run_result {
        Ok(app) => {
            app.run(move |app_handle, event| match event {
                tauri::RunEvent::Ready => {
                    if aptabase_enabled {
                        let _ = app_handle.track_event("app_started", None);
                    }
                }
                tauri::RunEvent::Exit { .. } => {
                    if aptabase_enabled {
                        let _ = app_handle.track_event("app_exited", None);
                    }
                    app_handle.flush_events_blocking();
                }

                // macOS: clicking the Dock icon of a running app reopens it.
                // When the main window is hidden (closed-to-hide), show it again.
                #[cfg(target_os = "macos")]
                tauri::RunEvent::Reopen { .. } => {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                _ => {}
            });
        }
        Err(err) => {
            eprintln!("Error while running application: {}", err);
        }
    }
}
