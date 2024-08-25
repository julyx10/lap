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
 */

mod cmd;
mod db;
mod utils;


/// The main function is the entry point for the Tauri application.
fn main() {
    tauri::Builder::default()
        .setup(|_app| {
            // Create the database on startup
            db::create_db().expect("error while creating the database");

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
            cmd::get_albums,
            cmd::add_album, 
            cmd::remove_album,
            cmd::read_folders,
            cmd::read_image_files,
            cmd::open_file]
        )
        .run(tauri::generate_context!())
        .expect("error while running application");
}
