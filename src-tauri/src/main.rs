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

mod t_storage;
mod t_ai;
mod t_cluster;
mod t_cmds;
mod t_common;
mod t_config;
mod t_dedup;
mod t_face;
mod t_image;
#[cfg(all(not(target_os = "macos"), lap_has_libheif))]
mod t_heif;
mod t_jpeg;
mod t_jxl;
mod t_lens;
mod t_libraw;
mod t_menu;
mod t_migration;
mod t_protocol;
mod t_sqlite;
mod t_utils;
mod t_http;
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
                Err(e) => eprintln!("Failed to start AI Engine: {}", e),
            }

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
                // Prevent the window from closing immediately
                api.prevent_close();

                let app_handle = window.app_handle();

                // Get all open windows
                let windows = app_handle.webview_windows();
                for (_, other_window) in windows {
                    // Skip the main window (we'll close it last)
                    if other_window.label() != "main" {
                        // Close each window
                        if let Err(err) = other_window.close() {
                            eprintln!("Failed to close window: {}", err);
                        }
                    }
                }

                app_handle.exit(0);
            }
        })
        .on_menu_event(|app, event| {
            t_menu::handle_menu_event(app, event);
        })
        .invoke_handler(tauri::generate_handler![
            // library
            t_cmds::get_app_config,
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
            t_cmds::copy_folder,
            t_cmds::delete_folder,
            t_cmds::reveal_path,
            t_cmds::open_external_url,
            t_cmds::get_external_app_display_name,
            t_cmds::open_file_with_app,
            // file
            t_cmds::get_total_count_and_sum,
            t_cmds::get_query_count_and_sum,
            t_cmds::get_query_time_line,
            t_cmds::get_query_files,
            t_cmds::get_query_file_position,
            t_cmds::get_folder_files,
            t_cmds::get_folder_thumb_count,
            t_cmds::edit_image,
            t_cmds::copy_edited_image,
            t_cmds::copy_image,
            t_cmds::rename_file,
            t_cmds::move_file,
            t_cmds::copy_file,
            t_cmds::delete_file,
            t_cmds::delete_db_file,
            t_cmds::edit_file_comment,
            t_cmds::get_file_thumb,
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
            t_cmds::set_file_favorite,
            t_cmds::set_file_rating,
            // tag
            t_cmds::get_all_tags,
            t_cmds::get_tag_name,
            t_cmds::create_tag,
            t_cmds::rename_tag,
            t_cmds::delete_tag,
            t_cmds::get_tags_for_file,
            t_cmds::add_tag_to_file,
            t_cmds::remove_tag_from_file,
            // calendar
            t_cmds::get_taken_dates,
            // camera
            t_cmds::get_camera_info,
            t_cmds::get_lens_info,
            // location
            t_cmds::get_location_info,
            // settings
            t_cmds::get_package_info,
            t_cmds::get_build_time,
            t_cmds::get_storage_file_info,
            // ai
            t_cmds::check_ai_status,
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
            t_cmds::dedup_get_group,
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
                _ => {}
            });
        }
        Err(err) => {
            eprintln!("Error while running application: {}", err);
        }
    }
}
