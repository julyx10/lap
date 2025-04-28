use reqwest::blocking::get;
use reqwest::Url;
use select::document::Document;
use select::node::Node;
use select::predicate::{Name, Attr};
use std::fs::{File, create_dir_all};
use std::io::{Write, BufReader};
use std::path::{Path, PathBuf};
use std::error::Error;
use std::fs::File as StdFile;

fn fetch_images_from_webpage(url: &str, save_dir: &str) -> Result<(), Box<dyn Error>> {
    // Create the save directory if it doesn't exist
    create_dir_all(save_dir)?;

    // Fetch the HTML content from the webpage
    let body = get(url)?.text()?;

    // Parse the HTML
    let document = Document::from(body.as_str());

    // Loop through all <img> tags to find image URLs
    for node in document.find(Name("img")) {
        if let Some(src) = node.attr("src") {
            let img_url = Url::parse(src)?;
            let img_url = img_url.join(url)?; // Resolve relative URLs

            // Get the image filename from the URL
            let file_name = Path::new(src)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown.jpg");

            let file_path = Path::new(save_dir).join(file_name);

            // Download and save the image
            if let Err(e) = download_and_save_image(&img_url, &file_path) {
                eprintln!("Failed to download image: {}. Error: {}", src, e);
            }
        }
    }

    Ok(())
}

fn download_and_save_image(url: &Url, path: &Path) -> Result<(), Box<dyn Error>> {
    // Fetch the image bytes
    let response = get(url.as_str())?;
    let content = response.bytes()?;

    // Save the image to the local file system
    let mut file = File::create(path)?;
    file.write_all(&content)?;

    Ok(())
}

// fn main() {
//     let url = "https://example.com";  // Replace with your URL
//     let save_dir = "images";  // Directory to save images

//     if let Err(e) = fetch_images_from_webpage(url, save_dir) {
//         eprintln!("Error: {}", e);
//     }
// }
