use crate::{NEONDB_FILE_MARK, NEONDB_FILE_SIZE};

use std::fs::File;
use std::io::{prelude::*, SeekFrom};

#[macro_export]
macro_rules! path_of {
    ($p: expr) => {{
        let path = concat!("../", $p);
        std::path::Path::new(path)
    }};
}

pub fn create_volume() {
    let mut vol = File::create(path_of!("tmp/test.neondb")).unwrap();
    vol.set_len(NEONDB_FILE_SIZE).unwrap();

    // menandai volume
    vol.seek(SeekFrom::Start(0)).unwrap();
    vol.write(&NEONDB_FILE_MARK.as_bytes()).unwrap();
}
