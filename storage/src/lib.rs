pub use error::ErrorKind;
use mount::MountValidator;

use std::fs::{File, OpenOptions};
use std::path::Path;

pub const NEONDB_FILE_EXT: &str = "neondb";
pub const NEONDB_FILE_SIZE: u64 = 1 << 23;

mod error;
mod mount;

type Result<T> = std::result::Result<T, self::error::ErrorKind>;

pub struct Storage {
    volume: Option<File>,
}

impl Storage {
    pub fn new() -> Self {
        Storage { volume: None }
    }

    pub fn mount(&mut self, path: &Path) -> Result<()> {
        MountValidator::validate(path)?;

        let f = OpenOptions::new().read(true).write(true).open(path);

        self.volume = match f {
            Ok(f) => Some(f),
            Err(_) => panic!("internal error"),
        };

        Ok(())
    }
}
