use std::env;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fmt::Write;

fn main() {
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
