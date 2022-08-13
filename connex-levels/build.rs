use std::{fs, path::PathBuf, str::FromStr};

use connex::World;

fn main() {
    let mut level_files: Vec<_> = fs::read_dir("levels").unwrap()
        .filter_map(|x| x.ok())
        .filter(|f| f.file_type().map(|t| t.is_file()).unwrap_or_default()) // is file
        .map(|f| PathBuf::from(f.file_name())) // file is *.txt
        .collect();

    level_files.sort_unstable();

    let mut src = String::new();

    src.push_str("&[");
    for path in level_files {
        let mut abs_path = PathBuf::new();
        abs_path.push(env!("CARGO_MANIFEST_DIR"));
        abs_path.push("levels");
        abs_path.push(path.as_path());

        println!("cargo:rerun-if-changed={}", abs_path.to_str().unwrap());

        let content = String::from_utf8(fs::read(&abs_path).unwrap()).unwrap();
        World::from_str(&content)
            .map_err(|e| format!("{} compile failed: {e}", path.to_str().unwrap()))
            .unwrap();

        src.push_str("include_str!(r#\"");
        src.push_str(abs_path.to_str().unwrap());
        src.push_str("\"#),");
    }
    src.push(']');

    println!("cargo:rerun-if-changed=levels");

    let mut out_file_path = PathBuf::new();
    out_file_path.push(std::env::var("OUT_DIR").unwrap());
    out_file_path.push("levels.rs");

    fs::write(out_file_path.as_path(), src.as_bytes()).unwrap()
}
