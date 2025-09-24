use std::{env, path::PathBuf};

fn main() {
    let native_enabled = std::env::var("CARGO_FEATURE_NATIVE").is_ok();
    if !native_enabled {
        return;
    }
    println!("cargo:warning=Compiling OSRM wrapper");
    cc::Build::new()
        .cpp(true)
        .file("src/wrapper.cpp")
        .flag("-std=c++17") // Wrapper code uses C++20
        // Expect the osrm headers to be placed in /usr/local/
        .include("/usr/local/include")
        .include("/usr/local/include/osrm") // Just in case includes are nested
        .define("ENABLE_LTO", "Off")
        .define("FMT_HEADER_ONLY", None)
        .compile("osrm_wrapper");

    // Cargo receives information about linking through print statements
    // with metadata as follows

    // Linking the actual OSRM library
    println!("cargo:rustc-link-search=native=/usr/local/lib");

    // Link the various osrm commands
    println!("cargo:rustc-link-lib=static=osrm");
    println!("cargo:rustc-link-lib=static=osrm_store");
    println!("cargo:rustc-link-lib=static=osrm_extract");
    println!("cargo:rustc-link-lib=static=osrm_partition");
    println!("cargo:rustc-link-lib=static=osrm_update");
    println!("cargo:rustc-link-lib=static=osrm_guidance");
    println!("cargo:rustc-link-lib=static=osrm_customize");
    println!("cargo:rustc-link-lib=static=osrm_contract");

    // // Link OSRM system deps
    println!("cargo:rustc-link-lib=dylib=boost_thread");
    println!("cargo:rustc-link-lib=dylib=boost_filesystem");
    println!("cargo:rustc-link-lib=dylib=boost_iostreams");
    println!("cargo:rustc-link-lib=dylib=tbb");
    println!("cargo:rustc-link-lib=dylib=stdc++");

    println!("cargo:rustc-link-lib=dylib=z");
    println!("cargo:rustc-link-lib=dylib=bz2");
    println!("cargo:rustc-link-lib=dylib=expat");

    // Link the OSRM wrapper
    let wrapper_dir = PathBuf::from(
        env::var("OUT_DIR").expect("Wrapper build failed to specify output directory"),
    );

    println!("cargo:rustc-link-search=native={}", wrapper_dir.display());
    println!("cargo:rustc-link-lib=static=osrm_wrapper");
}
