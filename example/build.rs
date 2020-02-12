use tauri_includedir_codegen::Compression;

fn main() {
    tauri_includedir_codegen::start("FILES")
        .dir("data", Compression::Gzip)
        .build(
            "data.rs",
            vec![
                "data/inner/boom".into(),
                "data/empty".to_string(),
                "data/foo".to_string(),
            ],
        )
        .unwrap();
}
