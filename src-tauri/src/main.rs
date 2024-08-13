// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/**
 * project: jc-photo
 * author:  julyxx
 * email:   tiangle@gmail.com
 * GitHub:  /julyx10
 */


// declare modules in root file
mod cmd;
mod db;

/**
 * The main function is the entry point for the Tauri application.
 */
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Create the database on startup
            db::create_db().expect("error while creating the database");
            Ok(())
        })
        .on_page_load(|window, _payload| {
            // avoid the blank window on startup
            // set window visible to false in tauri.conf.json at first
            window.show().unwrap();
        })
        .invoke_handler(tauri::generate_handler![
            cmd::get_albums,
            cmd::add_folder, 
            cmd::open_file]
        )
        .run(tauri::generate_context!())
        .expect("error while running application");
}
