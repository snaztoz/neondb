use crate::{NEONDB_FILE_MARK, NEONDB_FILE_SIZE};

use std::fs::{self, OpenOptions};
use std::io::{prelude::*, SeekFrom};
use std::path::Path;

#[macro_export]
macro_rules! path_of {
    ($p: expr) => {{
        let path = concat!("../", $p);
        std::path::Path::new(path)
    }};
}

// ?todo
//
// Gantikan fungsi ini
pub fn fresh_volume(path: &Path) {
    let mut vol = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)
        .unwrap();

    if path.metadata().unwrap().len() != NEONDB_FILE_SIZE {
        vol.set_len(NEONDB_FILE_SIZE).unwrap();
    }

    // NEONDB_FILE_MARK + mark dari head-block milik RSSAlloc
    let first_bytes = NEONDB_FILE_MARK
        .as_bytes()
        .iter()
        .chain(&16u64.to_be_bytes())
        .chain(&0u64.to_be_bytes())
        .map(|b| *b)
        .collect::<Vec<u8>>();

    // menandai volume
    vol.seek(SeekFrom::Start(0)).unwrap();
    vol.write(&first_bytes).unwrap();
}

pub fn ensure_not_exists(path: &Path) {
    if path.exists() {
        fs::remove_file(path).expect("fail at removing test file");
    }
}
