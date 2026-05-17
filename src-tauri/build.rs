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
    // Allow conditional compilation based on whether libheif is available.
    println!("cargo::rustc-check-cfg=cfg(lap_has_libheif)");

    write_build_info();
    build_libraw();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "macos" {
        build_libheif();
    }

    // build tauri
    tauri_build::build();
}

fn build_libheif() {
    println!("cargo:rerun-if-changed=third_party/libheif");
    println!("cargo:rerun-if-changed=third_party/libde265");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let source_dir = manifest_dir.join("third_party").join("libheif");
    if !source_dir.exists() {
        println!(
            "cargo:warning=libheif submodule not found at {}. Add it under src-tauri/third_party/libheif to enable HEIC/HEIF decoding on Windows/Linux.",
            source_dir.display()
        );
        return;
    }

    // NOTE: We intentionally keep this build minimal and static, mirroring the libjpeg-turbo approach.
    // The exact codec backends (libde265/dav1d/aom) depend on how libheif is vendored/configured.
    let out_dir = out_dir_path();
    let build_root = out_dir.join("libheif-build");
    let binary_dir = build_root.join("build");
    fs::create_dir_all(&binary_dir).unwrap();

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let is_windows = target_os == "windows";
    let use_cmake_default_generator = is_windows;
    let libde265 = match build_libde265(&manifest_dir, &out_dir, use_cmake_default_generator) {
        Some(build) => build,
        None => {
            println!(
                "cargo:warning=libde265 is unavailable, so libheif will be built without a working HEIC decoder backend."
            );
            return;
        }
    };

    // Configure
    let mut configure = Command::new("cmake");
    if !is_windows {
        configure.arg("-G").arg("Unix Makefiles");
    }
    configure
        .arg("-DCMAKE_BUILD_TYPE=Release")
        .arg("-DBUILD_SHARED_LIBS=OFF")
        .arg("-DBUILD_DOCUMENTATION=OFF")
        .arg("-DBUILD_TESTING=OFF")
        .arg("-DENABLE_PLUGIN_LOADING=OFF")
        .arg("-DENABLE_EXPERIMENTAL_FEATURES=OFF")
        .arg("-DENABLE_EXPERIMENTAL_MINI_FORMAT=OFF")
        .arg("-DWITH_EXAMPLES=OFF")
        .arg("-DWITH_GDK_PIXBUF=OFF")
        .arg("-DWITH_UNCOMPRESSED_CODEC=OFF")
        .arg("-DWITH_WEBCODECS=OFF")
        .arg("-DWITH_KVAZAAR=OFF")
        .arg("-DWITH_OpenJPEG_DECODER=OFF")
        .arg("-DWITH_OpenJPEG_ENCODER=OFF")
        .arg("-DWITH_OPENJPH_ENCODER=OFF")
        .arg("-DWITH_FFMPEG_DECODER=OFF")
        .arg("-DWITH_UVG266=OFF")
        .arg("-DWITH_VVDEC=OFF")
        .arg("-DWITH_VVENC=OFF")
        .arg("-DWITH_RAV1E=OFF")
        .arg("-DWITH_X265=OFF")
        .arg("-DWITH_X264=OFF")
        .arg("-DWITH_OpenH264_DECODER=OFF")
        .arg("-DWITH_AOM_DECODER=OFF")
        .arg("-DWITH_AOM_ENCODER=OFF")
        .arg("-DWITH_DAV1D=OFF")
        .arg("-DWITH_LIBDE265=ON")
        .arg("-DWITH_JPEG_DECODER=OFF")
        .arg("-DWITH_JPEG_ENCODER=OFF")
        .arg("-DWITH_LIBSHARPYUV=OFF")
        .arg(format!(
            "-DLIBDE265_INCLUDE_DIR={}",
            libde265.include_dir.display()
        ))
        .arg(format!(
            "-DLIBDE265_LIBRARY={}",
            libde265.lib_path.display()
        ))
        .arg(source_dir.as_os_str())
        .current_dir(&binary_dir);
    if is_windows {
        configure.arg("-DCMAKE_C_FLAGS=/DLIBDE265_STATIC_BUILD");
        configure.arg("-DCMAKE_CXX_FLAGS=/DLIBDE265_STATIC_BUILD");
    }

    run_command(&mut configure, "configure libheif");

    let jobs = env::var("NUM_JOBS").unwrap_or_else(|_| "1".to_string());
    run_command(
        Command::new("cmake")
            .arg("--build")
            .arg(".")
            .arg("--target")
            .arg("heif")
            .arg("--config")
            .arg("Release")
            .arg("--parallel")
            .arg(jobs)
            .current_dir(&binary_dir),
        "build libheif",
    );

    // Link - locate the static library output.
    // libheif's output name differs across platforms/build systems; keep it permissive.
    let candidates: [(&str, PathBuf); 14] = [
        ("heif", binary_dir.join("libheif.a")),
        ("heif", binary_dir.join("Release").join("libheif.a")),
        ("heif", binary_dir.join("libheif").join("libheif.a")),
        ("heif", binary_dir.join("libheif").join("Release").join("libheif.a")),
        ("heif", binary_dir.join("heif.lib")),
        ("heif", binary_dir.join("Release").join("heif.lib")),
        ("heif", binary_dir.join("Debug").join("heif.lib")),
        ("heif", binary_dir.join("libheif").join("Release").join("heif.lib")),
        ("heif", binary_dir.join("libheif").join("Debug").join("heif.lib")),
        ("libheif", binary_dir.join("libheif.lib")),
        ("libheif", binary_dir.join("Release").join("libheif.lib")),
        ("libheif", binary_dir.join("Debug").join("libheif.lib")),
        ("libheif", binary_dir.join("libheif").join("Release").join("libheif.lib")),
        ("libheif", binary_dir.join("libheif").join("Debug").join("libheif.lib")),
    ];

    let (lib_name, lib_path) = match candidates.iter().find(|(_, p)| p.exists()) {
        Some((name, path)) => (name.to_string(), path.clone()),
        None => {
            log_library_search_failure(&binary_dir, "libheif");
            println!(
                "cargo:warning=libheif build completed but static library was not found under {}",
                binary_dir.display()
            );
            return;
        }
    };

    let lib_dir = lib_path.parent().unwrap_or(&binary_dir);
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static={}", lib_name);
    println!("cargo:rustc-link-search=native={}", libde265.lib_dir.display());
    println!("cargo:rustc-link-lib=static={}", libde265.lib_name);
    match target_os.as_str() {
        "macos" => println!("cargo:rustc-link-lib=c++"),
        "windows" => {}
        // Some libheif builds depend on the C++ runtime.
        _ => println!("cargo:rustc-link-lib=stdc++"),
    }

    // Enable the Rust-side libheif bindings only when the native library is available.
    println!("cargo:rustc-cfg=lap_has_libheif");
}

/// writes the build information to a file
fn write_build_info() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("build_info.rs");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    // This intentionally records the real build time for About/version
    // metadata, so Cargo will rewrite build_info.rs on each build.
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
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let is_windows = target_os == "windows";
    let is_linux_aarch64 = target_os == "linux" && target_arch == "aarch64";
    const LIBRAW_LINK_NAME: &str = "raw";
    const SHIM_LINK_NAME: &str = "lap_libraw_shim";

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
        .cargo_metadata(false)
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
    if is_linux_aarch64 {
        // Avoid GCC's AArch64 outline atomics helpers
        // (__aarch64_swp8_sync / __aarch64_ldadd8_sync), which are not
        // resolved by libatomic on the GitHub Actions ARM runner.
        build.flag_if_supported("-mno-outline-atomics");
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

    build.compile(LIBRAW_LINK_NAME);

    // Build the shim (must compile before emitting -ljpeg so the linker
    // processes libjpeg after liblap_libraw_shim, which references jpeg).
    let mut shim = cc::Build::new();
    shim.cpp(true)
        .cargo_metadata(false)
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
    if is_linux_aarch64 {
        shim.flag_if_supported("-mno-outline-atomics");
    }

    shim.compile(SHIM_LINK_NAME);

    // cc::Build is compiled with cargo_metadata(false), so emit the search
    // path and link directives explicitly to preserve linker ordering.
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    if let Some(jpeg) = &jpeg_build {
        println!("cargo:rustc-link-search=native={}", jpeg.lib_dir.display());
    }

    // On Linux with static linking, libraw, libjpeg, and the shim are
    // interdependent. Use --start-group / --end-group via raw linker args to
    // let the linker resolve symbols across the archives iteratively, and to
    // bypass Cargo's deduplication of rustc-link-lib directives.
    if target_os == "linux" {
        println!("cargo:rustc-link-arg=-Wl,--start-group");
        println!("cargo:rustc-link-arg=-l{}", LIBRAW_LINK_NAME);
        if let Some(jpeg) = &jpeg_build {
            println!("cargo:rustc-link-arg=-l{}", jpeg.lib_name);
        }
        println!("cargo:rustc-link-arg=-l{}", SHIM_LINK_NAME);
        println!("cargo:rustc-link-arg=-Wl,--end-group");
    } else {
        println!("cargo:rustc-link-lib=static={}", LIBRAW_LINK_NAME);
        println!("cargo:rustc-link-lib=static={}", SHIM_LINK_NAME);
        if let Some(jpeg) = &jpeg_build {
            println!("cargo:rustc-link-lib=static={}", jpeg.lib_name);
        }
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

    // macOS pasteboard shim for drag-drop URL extraction
    if target_os == "macos" {
        cc::Build::new()
            .file("src/pasteboard.mm")
            .compile("lap_pasteboard");
        println!("cargo:rustc-link-lib=framework=AppKit");
    }
}

struct JpegBuild {
    include_dirs: Vec<PathBuf>,
    lib_dir: PathBuf,
    lib_name: String,
}

struct LibDe265Build {
    include_dir: PathBuf,
    lib_dir: PathBuf,
    lib_name: String,
    lib_path: PathBuf,
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

    let (static_lib, lib_name) = if is_windows {
        ("jpeg-static.lib", "jpeg-static")
    } else {
        ("libjpeg.a", "jpeg")
    };
    let static_lib_path = binary_dir.join(static_lib);
    let static_lib_path_release = binary_dir.join("Release").join(static_lib);

    fs::create_dir_all(&binary_dir).unwrap();

    if !static_lib_path.exists() && !static_lib_path_release.exists() {
        let mut configure = Command::new("cmake");
        if !is_windows {
            configure.arg("-G").arg("Unix Makefiles");
        }
        configure
            .arg("-DCMAKE_BUILD_TYPE=Release")
            .arg("-DENABLE_SHARED=FALSE")
            .arg("-DENABLE_STATIC=TRUE")
            .arg(source_dir.as_os_str())
            .current_dir(&binary_dir);

        run_command(&mut configure, "configure libjpeg-turbo");

        let jobs = env::var("NUM_JOBS").unwrap_or_else(|_| "1".to_string());
        run_command(
            Command::new("cmake")
                .arg("--build")
                .arg(".")
                .arg("--target")
                .arg("jpeg-static")
                .arg("--config")
                .arg("Release")
                .arg("--parallel")
                .arg(jobs)
                .current_dir(&binary_dir),
            "build libjpeg-turbo",
        );
    }

    let final_lib_dir = if static_lib_path_release.exists() {
        binary_dir.join("Release")
    } else {
        binary_dir.clone()
    };

    Some(JpegBuild {
        include_dirs: vec![binary_dir, source_dir.join("src")],
        lib_dir: final_lib_dir,
        lib_name: lib_name.to_string(),
    })
}

fn build_libde265(
    manifest_dir: &Path,
    out_dir: &Path,
    use_cmake_default_generator: bool,
) -> Option<LibDe265Build> {
    let source_dir = manifest_dir.join("third_party/libde265");
    if !source_dir.exists() {
        println!(
            "cargo:warning=libde265 submodule not found at {}. Run: git submodule update --init --recursive",
            source_dir.display()
        );
        return None;
    }

    let build_root = out_dir.join("libde265-build");
    let binary_dir = build_root.join("build");
    fs::create_dir_all(&binary_dir).unwrap();

    let candidates: [(&str, PathBuf); 13] = [
        ("de265", binary_dir.join("libde265.a")),
        ("de265", binary_dir.join("Release").join("libde265.a")),
        ("de265", binary_dir.join("libde265").join("libde265.a")),
        ("de265", binary_dir.join("de265.lib")),
        ("de265", binary_dir.join("Release").join("de265.lib")),
        ("de265", binary_dir.join("Debug").join("de265.lib")),
        ("de265", binary_dir.join("libde265").join("Release").join("de265.lib")),
        ("de265", binary_dir.join("libde265").join("Debug").join("de265.lib")),
        ("libde265", binary_dir.join("libde265.lib")),
        ("libde265", binary_dir.join("Release").join("libde265.lib")),
        ("libde265", binary_dir.join("Debug").join("libde265.lib")),
        ("libde265", binary_dir.join("libde265").join("Release").join("libde265.lib")),
        ("libde265", binary_dir.join("libde265").join("Debug").join("libde265.lib")),
    ];

    let have_existing = candidates.iter().any(|(_, path)| path.exists());
    if !have_existing {
        let mut configure = Command::new("cmake");
        // Windows keeps CMake's default generator; other targets use Makefiles
        // to match the GitHub Actions build environment.
        if !use_cmake_default_generator {
            configure.arg("-G").arg("Unix Makefiles");
        }
        configure
            .arg("-DBUILD_SHARED_LIBS=OFF")
            .arg("-DENABLE_SDL=OFF")
            .arg("-DENABLE_DECODER=ON")
            .arg("-DENABLE_ENCODER=OFF")
            .arg("-DENABLE_SHERLOCK265=OFF")
            .arg("-DENABLE_INTERNAL_DEVELOPMENT_TOOLS=OFF")
            .arg("-DWITH_FUZZERS=OFF")
            .arg(source_dir.as_os_str())
            .current_dir(&binary_dir);

        run_command(&mut configure, "configure libde265");

        let jobs = env::var("NUM_JOBS").unwrap_or_else(|_| "1".to_string());
        run_command(
            Command::new("cmake")
                .arg("--build")
                .arg(".")
                .arg("--target")
                .arg("de265")
                .arg("--config")
                .arg("Release")
                .arg("--parallel")
                .arg(jobs)
                .current_dir(&binary_dir),
            "build libde265",
        );
    }

    let (lib_name, lib_path) = match candidates.iter().find(|(_, path)| path.exists()) {
        Some((name, path)) => (name.to_string(), path.clone()),
        None => {
            log_library_search_failure(&binary_dir, "libde265");
            println!(
                "cargo:warning=libde265 build completed but static library was not found under {}",
                binary_dir.display()
            );
            return None;
        }
    };

    let include_dir = out_dir.join("libde265-include");
    let include_libde265_dir = include_dir.join("libde265");
    fs::create_dir_all(&include_libde265_dir).unwrap();
    fs::copy(
        source_dir.join("libde265").join("de265.h"),
        include_libde265_dir.join("de265.h"),
    )
    .unwrap();
    let version_header_candidates = [
        binary_dir.join("libde265").join("de265-version.h"),
        binary_dir.join("de265-version.h"),
        binary_dir.join("Release").join("de265-version.h"),
        binary_dir.join("Debug").join("de265-version.h"),
    ];
    let Some(version_header) = version_header_candidates.iter().find(|path| path.exists()) else {
        println!(
            "cargo:warning=libde265 build completed but generated de265-version.h was not found under {}",
            binary_dir.display()
        );
        return None;
    };
    fs::copy(version_header, include_libde265_dir.join("de265-version.h")).unwrap();
    let lib_dir = lib_path.parent().unwrap_or(&binary_dir).to_path_buf();

    Some(LibDe265Build {
        include_dir,
        lib_dir,
        lib_name,
        lib_path,
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
    sources.sort();
    sources
}

fn out_dir_path() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
}

fn log_library_search_failure(binary_dir: &Path, library_name: &str) {
    fn walk(dir: &Path, depth: usize, max_depth: usize) {
        if depth > max_depth {
            return;
        }
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, depth + 1, max_depth);
                continue;
            }
            if path.extension().is_some_and(|ext| {
                matches!(
                    ext.to_string_lossy().to_ascii_lowercase().as_str(),
                    "a" | "lib" | "dll" | "pdb"
                )
            }) {
                println!("cargo:warning=  found: {}", path.display());
            }
        }
    }

    println!(
        "cargo:warning=--- searching {} artifacts under {}",
        library_name,
        binary_dir.display()
    );
    walk(binary_dir, 0, 3);
}

fn run_command(command: &mut Command, description: &str) {
    let status = command
        .status()
        .unwrap_or_else(|e| panic!("Failed to {description}: {e}"));
    if !status.success() {
        panic!("Failed to {description}: exit status {status}");
    }
}
