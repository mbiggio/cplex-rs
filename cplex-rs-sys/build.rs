use glob::glob;
use std::env;
use std::path::PathBuf;

fn main() {
    let building_docs = std::env::var("DOCS_RS").is_ok();
    let cplex_include_path = if building_docs {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("include")
            .join("22010000")
    } else {
        let cplex_installation_path = env::var("CPLEX_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            glob("/opt/ibm/ILOG/*/cplex")
                .expect("Invalid glob pattern")
                .filter_map(|path| path.ok())
                .next()
                .expect("No valid CPLEX installation path found. Please set the env variable 'CPLEX_PATH' with the CPLEX installation directory or install CPLEX in the default location.")
        });

        let os = env::consts::OS;
        let arch = std::env::consts::ARCH;
        println!("cargo:warning=Detected OS: {}", os);
        println!("cargo:warning=Detected arch: {}", arch);

        let os_string = if os == "linux" && arch == "x86_64" {
            "x86-64_linux"
        } else if os == "macos" && arch == "aarch64" {
            "arm64_osx"
        } else {
            panic!("Unsupported OS-arch combination: {}-{}", os, arch);
        };

        let cplex_lib_path = cplex_installation_path.join(format!("lib/{os_string}/static_pic"));

        // Tell cargo to look for shared libraries in the specified directory
        println!(
            "cargo:rustc-link-search={}",
            cplex_lib_path.as_os_str().to_string_lossy()
        );

        // Tell cargo to tell rustc to link the system cplex
        // static library.
        println!("cargo:rustc-link-lib=cplex");

        cplex_installation_path.join("include")
    };

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(
            cplex_include_path
                .join("ilcplex")
                .join("cplex.h")
                .to_string_lossy(),
        )
        .clang_arg(format!(
            "-F{}",
            cplex_include_path.as_os_str().to_string_lossy()
        ))
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_item("CPX.*")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
