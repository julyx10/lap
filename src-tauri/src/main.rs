// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

/**
 * Main entry point for the Tauri application.
 * project: Lap
 * author:  julyxx
 * GitHub:  /julyx10
 * date:    2024-08-08
 */
use tauri::Manager;

mod t_ai;
mod t_cluster;
mod t_cmds;
mod t_common;
mod t_config;
mod t_face;
mod t_image;
mod t_migration;
mod t_sqlite;
mod t_utils;
mod t_video;

/// The main function is the entry point for the Tauri application.
fn main() {
    let builder = tauri::Builder::default();

    // #[cfg(debug_assertions)]
    // let builder = builder.plugin(tauri_plugin_devtools::init());

    builder
        .plugin(tauri_plugin_window_state::Builder::default().build()) // macOS: ~/Library/Application Support/{APP_NAME}/window-state.json
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
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
        .setup(|_app| {
            // Create the database on startup
            t_sqlite::create_db().expect("error while creating the database");

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
        .invoke_handler(tauri::generate_handler![
            // library
            t_cmds::get_app_config,
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
            t_cmds::get_album,
            t_cmds::add_album,
            t_cmds::edit_album,
            t_cmds::remove_album,
            t_cmds::set_album_display_order,
            t_cmds::set_album_cover,
            t_cmds::index_album,
            t_cmds::cancel_indexing,
            // folder
            t_cmds::select_folder,
            t_cmds::fetch_folder,
            t_cmds::count_folder,
            t_cmds::create_folder,
            t_cmds::rename_folder,
            t_cmds::move_folder,
            t_cmds::copy_folder,
            t_cmds::delete_folder,
            t_cmds::reveal_folder,
            // file
            t_cmds::get_total_count_and_sum,
            t_cmds::get_query_count_and_sum,
            t_cmds::get_query_time_line,
            t_cmds::get_query_files,
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
            t_cmds::get_file_image,
            t_cmds::check_file_exists,
            t_cmds::set_file_rotate,
            t_cmds::get_file_has_tags,
            // favorite
            t_cmds::get_favorite_folders,
            t_cmds::get_folder_favorite,
            t_cmds::set_folder_favorite,
            t_cmds::set_file_favorite,
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
            // location
            t_cmds::get_location_info,
            // print
            t_cmds::print_image,
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
            t_cmds::get_persons,
            t_cmds::rename_person,
            t_cmds::delete_person,
            t_cmds::get_faces_for_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running application");
}
