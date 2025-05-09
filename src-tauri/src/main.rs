// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

/**
 * project: jc-photo
 * author:  julyxx
 * email:   tiangle@gmail.com
 * GitHub:  /julyx10
 * date:    2024-08-08
 */

use tauri::Manager;

mod t_cmds;
mod t_sqlite;
mod t_utils;

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
        .setup(|_app| {
            // Create the database on startup
            t_sqlite::create_db().expect("error while creating the database");

            // // Open devtools in development mode
            // #[cfg(debug_assertions)] // only include this block in debug builds
            // {
            //     main_window.open_devtools();
            // }
    
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() != "main" {
                return;
            }

            if let tauri::WindowEvent::CloseRequested { api, .. }  = event {
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
            // album
            t_cmds::get_all_albums,
            t_cmds::get_album,
            t_cmds::add_album,
            t_cmds::edit_album,
            t_cmds::remove_album,
            t_cmds::set_album_display_order,

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
            // t_cmds::get_all_files,
            t_cmds::get_db_count,
            t_cmds::get_db_files,
            t_cmds::get_folder_files,
            t_cmds::copy_image_to_clipboard,
            t_cmds::rename_file,
            t_cmds::move_file,
            t_cmds::copy_file,
            t_cmds::delete_file,
            t_cmds::get_file_thumb,
            t_cmds::get_file_info,
            t_cmds::get_file_image,
            t_cmds::set_file_rotate,
            t_cmds::set_file_delete,

            // favorite
            t_cmds::get_favorite_folders,
            t_cmds::get_folder_favorite,
            t_cmds::set_folder_favorite,
            t_cmds::set_file_favorite,

            // calendar
            // t_cmds::get_files_by_date,
            // t_cmds::get_files_by_date_range,
            
            // camera
            // t_cmds::get_camera_files,
            t_cmds::get_camera_info,
            t_cmds::get_taken_dates,

            // print
            t_cmds::print_image,

            // settings
            t_cmds::get_db_file_size,
        ])
        .run(tauri::generate_context!())
        .expect("error while running application");
}
