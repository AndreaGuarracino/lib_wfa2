// extern crate bindgen;

use std::{env, path::PathBuf, process::Command};

struct BuildPaths {
    wfa_src: PathBuf,
}

impl BuildPaths {
    fn new() -> Self {
        Self {
            wfa_src: PathBuf::from("WFA2-lib"),
        }
    }

    fn wfa_lib_dir(&self) -> PathBuf {
        self.wfa_src.join("lib")
    }
}

fn build_wfa() -> Result<(), Box<dyn std::error::Error>> {
    let paths = BuildPaths::new();

    // Check if WFA2-lib exists and has Makefile
    if !paths.wfa_src.join("Makefile").exists() {
        return Err("WFA2-lib/Makefile not found. Make sure the submodule is initialized.".into());
    }

    // Detect platform and set appropriate compiler flags
    let target = env::var("TARGET").unwrap_or_default();
    let mut make_cmd = Command::new("make");
    
    // Handle platform-specific flags
    if target.contains("apple") || cfg!(target_os = "macos") {
        // For Apple Silicon/macOS, use mcpu=apple-m1 or generic flags
        if target.contains("aarch64") {
            // Apple Silicon M1/M2/M3
            make_cmd.env("CFLAGS", "-O3 -mcpu=apple-m1");
        } else {
            // Intel Mac
            make_cmd.env("CFLAGS", "-O3 -mtune=native");
        }
    } else if target.contains("x86_64") {
        // x86_64 Linux/Windows
        make_cmd.env("CFLAGS", "-O3 -march=native");
    } else if target.contains("aarch64") || target.contains("arm") {
        // ARM Linux
        make_cmd.env("CFLAGS", "-O3 -mcpu=native");
    } else {
        // Fallback to generic optimization
        make_cmd.env("CFLAGS", "-O3");
    }
    
    // Clean and build
    let output = make_cmd
        .args(["clean", "all"])
        .current_dir(&paths.wfa_src)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Make failed: {}", stderr).into());
    }

    Ok(())
}

fn setup_linking() {
    let paths = BuildPaths::new();

    // Link the WFA library
    println!("cargo:rustc-link-lib=static=wfa");
    
    // On macOS, link against libomp instead of libgomp
    let target = env::var("TARGET").unwrap_or_default();
    if target.contains("apple") || cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=omp");
    } else {
        println!("cargo:rustc-link-lib=gomp");
    }

    // Set library search path
    println!(
        "cargo:rustc-link-search=native={}",
        paths.wfa_lib_dir().display()
    );

    // Rerun if WFA library changes
    println!("cargo:rerun-if-changed=WFA2-lib");
    println!(
        "cargo:rerun-if-changed={}/libwfa.a",
        paths.wfa_lib_dir().display()
    );

    // Generate bindings
    // let bindings = bindgen::Builder::default()
    //     // Generate bindings for this header file.
    //     // .header("../wfa2/wavefront/wavefront_align.h")
    //     .header("WFA2-lib/wavefront/wavefront_align.h")
    //     // Add this directory to the include path to find included header files.
    //     // .clang_arg("-I../wfa2")
    //     .clang_arg(format!("-I{}", build_paths.wfa_src().display()))
    //     // Generate bindings for all functions starting with `wavefront_`.
    //     .allowlist_function("wavefront_.*")
    //     // Generate bindings for all variables starting with `wavefront_`.
    //     .allowlist_var("wavefront_.*")
    //     // Invalidate the built crate whenever any of the included header files
    //     // changed.
    //     .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
    //     // Finish the builder and generate the bindings.
    //     .generate()
    //     // Unwrap the Result and panic on failure.
    //     .expect("Unable to generate bindings");
    // // Write the bindings to the $OUT_DIR/bindings_wfa.rs file.
    // bindings
    //     .write_to_file(build_paths.out_dir().join("bindings_wfa.rs"))
    //     .expect("Couldn't write bindings!");
}

fn main() {
    if let Err(e) = build_wfa() {
        panic!("Failed to build WFA2-lib: {}", e);
    }
    setup_linking();
}