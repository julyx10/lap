/**
 * project: Lap
 * author:  julyx10
 * date:    2024-08-08
 */
use std::env;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    write_build_info();
    build_libraw();

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
    write!(
        &mut formatted,
        "pub const BUILD_UNIX_TIME: u64 = {};",
        timestamp
    )
    .unwrap();

    fs::write(dest_path, formatted).unwrap();
}

fn build_libraw() {
    let is_windows = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows";
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    println!("cargo:rerun-if-changed=src/libraw_shim.cpp");
    println!("cargo:rerun-if-changed=src/jpeg_shim.cpp");
    println!("cargo:rerun-if-changed=third_party/LibRaw");
    println!("cargo:rerun-if-changed=third_party/libjpeg-turbo");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = out_dir_path();

    // Build libjpeg-turbo from submodule
    let jpeg_build = build_libjpeg(&manifest_dir, &out_dir, is_windows);

    // Build LibRaw from submodule source using cc crate
    let libraw_source = manifest_dir.join("third_party/LibRaw");
    if !libraw_source.exists() {
        panic!(
            "LibRaw source not found at {}. Run: git submodule update --init --recursive",
            libraw_source.display()
        );
    }

    let libraw_sources = collect_cpp_sources(&libraw_source.join("src"));

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .include(&libraw_source)
        .warnings(false)
        .define("LIBRAW_BUILDLIB", None);

    if is_windows {
        build
            .flag("/std:c++17")
            .flag("/EHsc")
            .define("WIN32", None)
            .define("LIBRAW_NODLL", None);
    } else {
        build
            .flag_if_supported("-std=c++17")
            .flag_if_supported("-w");
    }

    if let Some(jpeg) = &jpeg_build {
        for inc in &jpeg.include_dirs {
            build.include(inc);
        }
        build.define("USE_JPEG", None);
    }

    for source_file in &libraw_sources {
        build.file(source_file);
    }

    build.compile("raw");

    // Link dependencies
    if let Some(jpeg) = &jpeg_build {
        println!("cargo:rustc-link-search=native={}", jpeg.lib_dir.display());
        println!("cargo:rustc-link-lib=static={}", jpeg.lib_name);
    }
    if is_windows {
        println!("cargo:rustc-link-lib=ws2_32");
    } else {
        println!("cargo:rustc-link-lib=z");
        println!("cargo:rustc-link-lib=m");
        match target_os.as_str() {
            "macos" => println!("cargo:rustc-link-lib=c++"),
            "linux" => println!("cargo:rustc-link-lib=stdc++"),
            _ => {}
        }
    }

    // Build the shim
    let mut shim = cc::Build::new();
    shim.cpp(true)
        .include(&libraw_source)
        .include(libraw_source.join("libraw"))
        .warnings(false)
        .file("src/libraw_shim.cpp")
        .file("src/jpeg_shim.cpp");

    if let Some(jpeg) = &jpeg_build {
        for inc in &jpeg.include_dirs {
            shim.include(inc);
        }
    }

    if is_windows {
        shim.flag("/std:c++17")
            .flag("/EHsc")
            .define("WIN32", None)
            .define("LIBRAW_NODLL", None);
    } else {
        shim.flag_if_supported("-std=c++17");
    }

    shim.compile("lap_libraw_shim");
}

struct JpegBuild {
    include_dirs: Vec<PathBuf>,
    lib_dir: PathBuf,
    lib_name: String,
}

fn build_libjpeg(manifest_dir: &Path, out_dir: &Path, is_windows: bool) -> Option<JpegBuild> {
    let source_dir = manifest_dir.join("third_party/libjpeg-turbo");
    if !source_dir.exists() {
        println!(
            "cargo:warning=libjpeg-turbo submodule not found at {}. Run: git submodule update --init --recursive",
            source_dir.display()
        );
        return None;
    }

    let build_root = out_dir.join("libjpeg-build");
    let binary_dir = build_root.join("build");

    let (generator, static_lib, lib_name) = if is_windows {
        ("NMake Makefiles", "jpeg-static.lib", "jpeg-static")
    } else {
        ("Unix Makefiles", "libjpeg.a", "jpeg")
    };
    let static_lib_path = binary_dir.join(static_lib);

    fs::create_dir_all(&binary_dir).unwrap();

    if !static_lib_path.exists() {
        run_command(
            Command::new("cmake")
                .arg("-G")
                .arg(generator)
                .arg("-DCMAKE_BUILD_TYPE=Release")
                .arg("-DENABLE_SHARED=FALSE")
                .arg("-DENABLE_STATIC=TRUE")
                .arg(source_dir.as_os_str())
                .current_dir(&binary_dir),
            "configure libjpeg-turbo",
        );

        let jobs = env::var("NUM_JOBS").unwrap_or_else(|_| "1".to_string());
        run_command(
            Command::new("cmake")
                .arg("--build")
                .arg(".")
                .arg("--target")
                .arg("jpeg-static")
                .arg("--parallel")
                .arg(jobs)
                .current_dir(&binary_dir),
            "build libjpeg-turbo",
        );
    }

    Some(JpegBuild {
        include_dirs: vec![binary_dir.clone(), source_dir.join("src")],
        lib_dir: binary_dir,
        lib_name: lib_name.to_string(),
    })
}

/// Recursively collect all .cpp files under a directory
fn collect_cpp_sources(dir: &Path) -> Vec<PathBuf> {
    let mut sources = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                sources.extend(collect_cpp_sources(&path));
            } else if path.extension().is_some_and(|ext| ext == "cpp") {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if !name.ends_with("_ph.cpp") {
                    sources.push(path);
                }
            }
        }
    }
    sources
}

fn out_dir_path() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
}

fn run_command(command: &mut Command, description: &str) {
    let status = command
        .status()
        .unwrap_or_else(|e| panic!("Failed to {}: {}", description, e));
    if !status.success() {
        panic!("Failed to {}: exit status {}", description, status);
    }
}
