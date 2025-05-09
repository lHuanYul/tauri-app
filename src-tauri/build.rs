use std::path::Path;

const CLIB:&str = "clib.a";

fn main() {
    let c_path = Path::new("src").join("c");
    let c_files = vec![c_path.join("hytest.c")];
    cc::Build::new()
        .include(c_path)
        .files(c_files)
        .flag_if_supported("-std=c11")
        .compile(CLIB);

    println!("cargo:rustc-link-lib=libeng");   // engine API
    println!("cargo:rustc-link-lib=libmx");    // matrix API
    println!("cargo:rustc-link-search=C:/Program Files/MATLAB/R2024b/extern/lib/win64/microsoft");

    tauri_build::build()
}
