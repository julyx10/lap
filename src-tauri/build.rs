use std::env;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fmt::Write;
// use std::path::PathBuf;

fn main() {

    write_build_info();
    
    // cp_ffmpeg_to_target();

    // build tauri
    tauri_build::build();
}

/// writes the build information to a file
fn write_build_info() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("build_info.rs");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let timestamp = now.as_secs();

    let mut formatted = String::new();
    write!(&mut formatted, "pub const BUILD_UNIX_TIME: u64 = {};", timestamp).unwrap();

    fs::write(dest_path, formatted).unwrap();
}

// copy ffmpeg binaries to the target directory
// fn cp_ffmpeg_to_target() {
//     // OUT_DIR = target/debug/build/rusty_ffmpeg-abc123/out
//     let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
//     let target_dir = out_dir.join("../../../.."); // go to `target/debug` or `target/release`

//     let bin_dir = target_dir.join("ffmpeg-bin");
//     fs::create_dir_all(&bin_dir).unwrap();

//     // Copy ffmpeg and ffprobe
//     fs::copy("bin/ffmpeg/ffmpeg", bin_dir.join("ffmpeg")).unwrap();
//     fs::copy("bin/ffmpeg/ffprobe", bin_dir.join("ffprobe")).unwrap();
// }