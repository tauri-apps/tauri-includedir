use tauri_includedir_codegen::Compression;

fn main() {
    tauri_includedir_codegen::start("FILES")
        .dir("data", Compression::Gzip)
        .build("data.rs", vec!["notinclude".to_string()])
        .unwrap();
}
