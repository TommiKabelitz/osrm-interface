use std::{env, path::PathBuf};

fn main() {
    let native_enabled = env::var("CARGO_FEATURE_NATIVE").is_ok();
    if !native_enabled {
        return;
    }

    println!("cargo:rerun-if-changed=src/wrapper.cpp");
    println!("cargo:warning=Compiling OSRM wrapper");

    let is_debug = matches!(env::var("PROFILE").as_deref(), Ok("debug"));

    // Determine backend path
    let osrm_path = if is_debug {
        env::var("OSRM_DEBUG_PATH")
            .or_else(|_| env::var("OSRM_BACKEND_PATH"))
            .unwrap_or_else(|_| {
                println!("cargo:warning=No OSRM_DEBUG_PATH or OSRM_BACKEND_PATH set, falling back to /usr/local");
                "/usr/local".into()
            })
    } else {
        env::var("OSRM_BACKEND_PATH").unwrap_or_else(|_| "/usr/local".into())
    };

    let include_dir = PathBuf::from(&osrm_path).join("include");
    let osrm_include_dir = include_dir.join("osrm");
    let lib_dir = PathBuf::from(&osrm_path).join("lib");

    // Check if essential headers exist
    if !include_dir.join("osrm.hpp").exists() && !osrm_include_dir.join("osrm.hpp").exists() {
        panic!(
            "Could not find OSRM header: osrm.hpp in {} or {}. Make sure OSRM is built or OSRM_BACKEND_PATH is set correctly.",
            include_dir.display(),
            osrm_include_dir.display(),
        );
    }

    // Start building
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .file("src/wrapper.cpp")
        .flag("-std=c++17")
        .include(&include_dir)
        .include(&osrm_include_dir)
        .define("ENABLE_LTO", "Off")
        .define("FMT_HEADER_ONLY", None);

    if is_debug {
        build
            .flag("-g")
            .flag("-O0")
            .flag("-fno-omit-frame-pointer")
            .flag("-fno-inline");
    } else {
        build.flag("-O3").flag("-DNDEBUG");
    }

    build.compile("osrm_wrapper");

    // Link directories
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    println!("cargo:rustc-link-lib=static=osrm");

    // Link system deps
    for lib in &[
        "boost_thread",
        "boost_filesystem",
        "boost_iostreams",
        "tbb",
        "stdc++",
        "z",
        "bz2",
        "expat",
    ] {
        println!("cargo:rustc-link-lib=dylib={}", lib);
    }

    // Link wrapper
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=osrm_wrapper");
}
