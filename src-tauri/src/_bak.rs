/// It is kept here for reference and to show the evolution of the code.
/// 



/// 2024-09-08 moved from t_sqlite.rs

/// Define the exif struct
// pub struct ExifData {
//     pub id:             Option<i64>,        // unique id (autoincrement by db)
//     pub file_id:        i64,                // file id (from files table)
//     pub make:           Option<String>,     // camera make
//     pub model:          Option<String>,     // camera model
//     pub date_time:      Option<String>,  
//     pub exposure_time:  Option<String>,
//     pub f_number:       Option<String>,
//     pub iso_speed:      Option<String>,
//     pub focal_length:   Option<String>,
// }


// impl ExifData {

//     /// insert exif data into db
//     pub fn update_db(&self) -> Result<()> {
//         let conn = get_conn()?;
//         conn.execute(
//             "INSERT INTO exif_data (file_id, make, model, date_time, exposure_time, f_number, iso_speed, focal_length) 
//             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
//             params![
//                 self.file_id,
//                 self.make,
//                 self.model,
//                 self.date_time,
//                 self.exposure_time,
//                 self.f_number,
//                 self.iso_speed,
//                 self.focal_length,
//             ],
//         )?;
//         Ok(())
//     }
// }


// exif_data table
// conn.execute(
//     "CREATE TABLE IF NOT EXISTS exif_data (
//         id INTEGER PRIMARY KEY AUTOINCREMENT,
//         file_id INTEGER NOT NULL,
//         make TEXT,
//         model TEXT,
//         date_time TEXT,
//         exposure_time TEXT,
//         f_number TEXT,
//         iso_speed TEXT,
//         focal_length TEXT,
//         FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
//     )",
//     [],
// )?;


/// open a picture file
// #[tauri::command]
// pub fn open_file() -> Option<String> {
//     let result = FileDialog::new()
//         .set_title("open a picture file")
//         .add_filter("JPEG Image", &["jpg", "jpeg"])
//         .add_filter("PNG Image", &["png"])
//         .show_open_single_file();

//     match result {
//         Ok(Some(path)) => {
//             let file = fs::File::open(&path).ok()?;

//             // create an exif reader and read the exif data
//             let mut bufreader = BufReader::new(file);
//             let exif = Reader::new().read_from_container(&mut bufreader).ok()?;

//             let exit_data = t_sqlite::ExifData {
//                 id: None,
//                 file_id: 0,
//                 make: exif.get_field(Tag::Make, In::PRIMARY)
//                     .map(|field| field.display_value().with_unit(&exif).to_string().replace("\"", "")),
//                 model: exif.get_field(Tag::Model, In::PRIMARY)
//                     .map(|field| field.display_value().with_unit(&exif).to_string().replace("\"", "")),
//                 date_time: exif.get_field(Tag::DateTime, In::PRIMARY)
//                     .map(|field| field.display_value().with_unit(&exif).to_string()),
//                 exposure_time: exif.get_field(Tag::ExposureTime, In::PRIMARY)
//                     .map(|field| field.display_value().with_unit(&exif).to_string()),
//                 f_number: exif.get_field(Tag::FNumber, In::PRIMARY)
//                     .map(|field| field.display_value().with_unit(&exif).to_string()),
//                 iso_speed: exif.get_field(Tag::PhotographicSensitivity, In::PRIMARY)
//                     .map(|field| field.display_value().with_unit(&exif).to_string()),
//                 focal_length: exif.get_field(Tag::FocalLength, In::PRIMARY)
//                     .map(|field| field.display_value().with_unit(&exif).to_string()),
//             };

//             exit_data.update_db().expect("error while saving to db");
//             Some(path.to_str().unwrap().to_string())
//         },
//         Ok(None) => None,
//         Err(_) => None,
//     }
// }