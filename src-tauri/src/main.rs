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

mod t_cmds;
mod t_sqlite;
mod t_utils;


/// The main function is the entry point for the Tauri application.
fn main() {
    tauri::Builder::default()
        .setup(|_app| {
            // Create the database on startup
            t_sqlite::create_db().expect("error while creating the database");

            // let main_window = app.get_window("main").unwrap();
            // // Open devtools in development mode
            // #[cfg(debug_assertions)] // only include this block in debug builds
            // {
            //     main_window.open_devtools();
            // }

            Ok(())
        })
        .on_page_load(|window, _payload| {
            // avoid the blank window on startup
            // set window visible to false in tauri.conf.json at first
            window.show().unwrap();
        })
        .invoke_handler(tauri::generate_handler![
            t_cmds::get_albums,
            t_cmds::add_album, 
            t_cmds::delete_album,
            t_cmds::select_folder,
            t_cmds::expand_folder,
            t_cmds::get_files,
            t_cmds::get_file_thumb,
            t_cmds::get_file_info,
            t_cmds::get_file_image,
            t_cmds::get_camera_info,
            t_cmds::get_camera_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running application");
}
