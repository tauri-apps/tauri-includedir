use tauri_includedir_codegen::Compression;

fn main() {
    tauri_includedir_codegen::start("FILES")
        .dir("data", Compression::Gzip)
        .build(
            "data.rs",
            vec![
                "inner\\boom".to_string(),
                "empty".to_string(),
                "foo".to_string(),
            ],
        )
        .unwrap();
}
