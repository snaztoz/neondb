use storage::NEONDB_FILE_SIZE;

use std::fs::File;

#[macro_export]
macro_rules! path_of {
    ($p: expr) => {{
        use std::path::Path;

        let path = concat!("../", $p);
        Path::new(path)
    }};
}

pub fn create_volume() {
    let vol = File::create(path_of!("tmp/test.neondb")).unwrap();
    vol.set_len(NEONDB_FILE_SIZE).unwrap();
}
