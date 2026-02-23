use std::env;
use std::path::PathBuf;

fn main() {
    let otio_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("vendor")
        .join("OpenTimelineIO-C-Bindings");

    if !otio_dir.exists() {
        println!(
            "cargo:warning=OpenTimelineIO-C-Bindings not found at {}. \
             Run: git submodule update --init --recursive",
            otio_dir.display()
        );
        println!("cargo:warning=Building without OTIO C bindings. Using stub implementation.");
        return;
    }

    let dst = cmake::Config::new(&otio_dir)
        .define("COTIO_SHARED_LIBS", "OFF")
        .build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=copentime");
    println!("cargo:rustc-link-lib=static=copentimelineio");
    println!("cargo:rustc-link-lib=static=opentime");
    println!("cargo:rustc-link-lib=static=opentimelineio");
    println!("cargo:rustc-link-lib=dylib=c++");

    let include_path = dst.join("include");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", include_path.display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings");
}
