use super::{ErrorKind, Result, NEONDB_FILE_EXT, NEONDB_FILE_MARK, NEONDB_FILE_SIZE};

use std::fs::{File, OpenOptions};
use std::io::{prelude::*, SeekFrom};
use std::path::Path;

pub struct MountValidator;

impl MountValidator {
    pub fn validate(path: &Path) -> Result<()> {
        let validator = MountValidator;

        if !path.exists() {
            return Err(ErrorKind::VolumeNotFound);
        }

        validator.validate_ext(path)?;
        validator.validate_size(path)?;
        validator.validate_vol_mark(path)?;

        Ok(())
    }

    pub fn validate_new(path: &Path) -> Result<()> {
        let validator = MountValidator;

        if path.exists() {
            return Err(ErrorKind::VolumeAlreadyExists);
        }

        validator.validate_ext(path)?;

        Ok(())
    }

    fn validate_ext(&self, path: &Path) -> Result<()> {
        if let Some(ext) = path.extension() {
            if ext == NEONDB_FILE_EXT {
                return Ok(());
            }
        }
        return Err(ErrorKind::VolumeInvalidExt);
    }

    fn validate_size(&self, path: &Path) -> Result<()> {
        let metadata = match path.metadata() {
            Ok(metadata) => metadata,
            Err(_) => return Err(ErrorKind::VolumeInaccessible),
        };

        if metadata.len() != NEONDB_FILE_SIZE {
            return Err(ErrorKind::VolumeInvalidSize);
        }
        return Ok(());
    }

    fn validate_vol_mark(&self, path: &Path) -> Result<()> {
        let mut buff = [0u8; NEONDB_FILE_MARK.len()];

        let mut vol = File::open(path).unwrap();
        vol.seek(SeekFrom::Start(0)).unwrap();
        vol.read(&mut buff).unwrap();

        if buff != NEONDB_FILE_MARK.as_bytes() {
            return Err(ErrorKind::VolumeCorrupted);
        }
        Ok(())
    }
}

pub fn new_volume(path: &Path) -> Result<File> {
    let mut vol = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .expect("creating new volume");

    vol.set_len(NEONDB_FILE_SIZE)
        .and_then(|_| vol.seek(SeekFrom::Start(0)))
        .and_then(|_| vol.write(&NEONDB_FILE_MARK.as_bytes()))
        .map_err(|_| ErrorKind::VolumeInitFailed)?;

    Ok(vol)
}
