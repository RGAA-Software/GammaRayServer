use std::fs;
use std::path::Path;

fn main() {
    // save folder with exe
    let src_path = "src/dash_settings.toml";
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).parent().unwrap()
        .parent().unwrap()
        .parent().unwrap()
        .join("dash_settings.toml");

    if let Err(e) = fs::copy(src_path, dest_path) {
        eprintln!("copy settings failed: {}", e);
    }

    // root folder in RustRover IDE
    let src_path = "src/dash_settings.toml";
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).parent().unwrap()
        .parent().unwrap()
        .parent().unwrap()
        .parent().unwrap()
        .parent().unwrap()
        .join("dash_settings.toml");

    if let Err(e) = fs::copy(src_path, dest_path) {
        eprintln!("copy settings failed: {}", e);
    }
}