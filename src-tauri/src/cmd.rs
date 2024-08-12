/**
 * This file is used to define the commands that can be called from the JS side of the application.
 * Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
 */

// use tauri::{
//     api::dialog::FileDialogBuilder,
//     api::dialog::MessageDialogBuilder
// };

// use std::{
//     sync::mpsc, 
//     thread
// };

use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;
use native_dialog::{FileDialog, MessageDialog, MessageType};
use dirs;
use chrono::{NaiveDateTime, Utc, Local};
use crate::db;
use exif::{Exif, In, Reader, Tag};


/**
 * The greet function is a simple example of a command that takes a string argument and returns a string.
 * The rename_all attribute is used to convert the function name to snake_case, which is the recommended 
 * naming convention for Tauri commands.
 */
#[tauri::command(rename_all = "snake_case")]
pub fn greet(name: &str) -> String {
    format!("Hi, {}! You've been greeted from Rust!", name)
}

/**
 * open a folder
 */
#[tauri::command]
pub fn open_folder() -> Option<String> {
    // get the desktop directory
    let desktop_dir = match dirs::desktop_dir() {
        Some(path) => path,
        None => PathBuf::from("~"),
    };
    let result = FileDialog::new()
        .set_location(&desktop_dir)
        .set_title("open a folder")
        .show_open_single_dir();

    match result {
        Ok(Some(path)) => {
            let album = db::Album {
                name: path.clone().file_name().unwrap().to_str().unwrap().to_string(),
                description: None,
                location: path.to_str().unwrap().to_string(),
                created_at: Utc::now().timestamp(),
                updated_at: Utc::now().timestamp()
            };
            album.save_to_db().expect("error while saving to db");
            Some(path.to_str().unwrap().to_string())
        },
        Ok(None) => None,
        Err(_) => None,
    }
}

/**
 * open a picture file
 */
#[tauri::command]
pub fn open_file() -> Option<String> {
    let result = FileDialog::new()
        .set_title("open a picture file")
        .add_filter("JPEG Image", &["jpg", "jpeg"])
        .add_filter("PNG Image", &["png"])
        .show_open_single_file();

    match result {
        Ok(Some(path)) => {
            let file = File::open(&path).ok()?;

            // create an exif reader and read the exif data
            let mut bufreader = BufReader::new(file);
            let exif = Reader::new().read_from_container(&mut bufreader).ok()?;

            let exit_data = db::ExifData {
                thumbnail_id: 0,
                make: exif.get_field(Tag::Make, In::PRIMARY)
                    .map(|field| field.display_value().with_unit(&exif).to_string().replace("\"", "")),
                model: exif.get_field(Tag::Model, In::PRIMARY)
                    .map(|field| field.display_value().with_unit(&exif).to_string().replace("\"", "")),
                date_time: exif.get_field(Tag::DateTime, In::PRIMARY)
                    .map(|field| field.display_value().with_unit(&exif).to_string()),
                exposure_time: exif.get_field(Tag::ExposureTime, In::PRIMARY)
                    .map(|field| field.display_value().with_unit(&exif).to_string()),
                f_number: exif.get_field(Tag::FNumber, In::PRIMARY)
                    .map(|field| field.display_value().with_unit(&exif).to_string()),
                iso_speed: exif.get_field(Tag::PhotographicSensitivity, In::PRIMARY)
                    .map(|field| field.display_value().with_unit(&exif).to_string()),
                focal_length: exif.get_field(Tag::FocalLength, In::PRIMARY)
                    .map(|field| field.display_value().with_unit(&exif).to_string()),
            };

            exit_data.save_to_db().expect("error while saving to db");
            Some(path.to_str().unwrap().to_string())
        },
        Ok(None) => None,
        Err(_) => None,
    }
}