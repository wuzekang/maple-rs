use std::{env, path::PathBuf};

fn main() {
    let dst = cmake::Config::new("freetype")
        .define("DISABLE_FORCE_DEBUG_POSTFIX", "ON")
        .build();

    env::set_var(
        "PKG_CONFIG_PATH",
        format!("{}/lib/pkgconfig", dst.display()),
    );

    let pkg = pkg_config::probe_library("freetype2").unwrap();

    let include_paths = pkg
        .include_paths
        .iter()
        .map(|path| format!("-I{}", path.to_str().unwrap()));

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .use_core()
        // .derive_debug(false)
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .clang_args(include_paths)
        .header("wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
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
