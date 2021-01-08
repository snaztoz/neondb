use std::fs::{self, OpenOptions};
use std::path::Path;

const TMP_PATH: &str = "../tmp/storage/";

fn main() {
    println!("cargo:rerun-if-changed={}", TMP_PATH);

    let storage_path = Path::new(TMP_PATH);
    if !storage_path.exists() {
        fs::create_dir_all(storage_path).unwrap();
    }

    let files = vec!["invalid.txt", "invalid.neondb"];
    for file in files.iter() {
        let fp = storage_path.join(file);
        if !fp.exists() {
            OpenOptions::new()
                .create(true)
                .write(true)
                .open(storage_path.join(file))
                .unwrap();
        }
    }
}
