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
        // Base CFLAGS for the target architecture
        let mut cflags = if target.contains("aarch64") {
            "-O3 -mcpu=apple-m1".to_string()
        } else {
            "-O3 -mtune=native".to_string()
        };

        // On macOS, find libomp installed by Homebrew to get correct paths
        let libomp_prefix = String::from_utf8(
            Command::new("brew")
                .arg("--prefix")
                .arg("libomp")
                .output()?
                .stdout,
        )
        .unwrap()
        .trim()
        .to_string();

        // Add the include path for omp.h to CFLAGS
        cflags.push_str(&format!(" -I{}/include", libomp_prefix));
        make_cmd.env("CFLAGS", cflags);

        // Add the library path for the linker
        make_cmd.env("LDFLAGS", format!("-L{}/lib", libomp_prefix));

        // Explicitly set the correct OpenMP flags for macOS to override Makefile logic.
        make_cmd.env("OMP_FLAG", "-Xpreprocessor -fopenmp -lomp");
    } else if target.contains("x86_64") {
        make_cmd.env("CFLAGS", "-O3 -march=native");
    } else if target.contains("aarch64") || target.contains("arm") {
        make_cmd.env("CFLAGS", "-O3 -mcpu=native");
    } else {
        make_cmd.env("CFLAGS", "-O3");
    }

    // Disable building examples and tools
    make_cmd.env("BUILD_EXAMPLES", "0");
    make_cmd.env("BUILD_TOOLS", "0");
    make_cmd.env("BUILD_WFA_PARALLEL", "0");

    // Clean and build only the static library, not the tools.
    let output = make_cmd
        .args(["clean", "all"])
        .current_dir(&paths.wfa_src)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!("Make failed:\nSTDOUT:\n{}\nSTDERR:\n{}", stdout, stderr).into());
    }

    Ok(())
}

fn setup_linking() {
    let paths = BuildPaths::new();

    // Link the WFA library
    println!("cargo:rustc-link-lib=static=wfa");

    // On macOS, link against libomp instead of libgomp for the final Rust binary
    let target = env::var("TARGET").unwrap_or_default();
    if target.contains("apple") || cfg!(target_os = "macos") {
        // Find libomp from Homebrew and add its lib path for rustc to find.
        let libomp_prefix = String::from_utf8(
            Command::new("brew")
                .arg("--prefix")
                .arg("libomp")
                .output()
                .expect("Failed to execute brew command to find libomp")
                .stdout,
        )
        .unwrap()
        .trim()
        .to_string();
        println!("cargo:rustc-link-search=native={}/lib", libomp_prefix);

        println!("cargo:rustc-link-lib=omp");
    } else {
        println!("cargo:rustc-link-lib=gomp");
    }

    // Set library search path for WFA
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