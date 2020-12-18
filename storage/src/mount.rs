use super::{ErrorKind, Result, NEONDB_FILE_EXT, NEONDB_FILE_SIZE};

use std::path::Path;

pub struct MountValidator;

impl MountValidator {
    pub fn validate(path: &Path) -> Result<()> {
        let validator = MountValidator;

        validator.is_file_exist(path)?;
        validator.is_valid_ext(path)?;
        validator.is_valid_size(path)?;

        Ok(())
    }

    fn is_file_exist(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(ErrorKind::VolumeNotFound);
        }
        Ok(())
    }

    fn is_valid_ext(&self, path: &Path) -> Result<()> {
        if let Some(ext) = path.extension() {
            if ext == NEONDB_FILE_EXT {
                return Ok(());
            }
        }
        return Err(ErrorKind::VolumeInvalidExt);
    }

    fn is_valid_size(&self, path: &Path) -> Result<()> {
        let metadata = match path.metadata() {
            Ok(metadata) => metadata,
            Err(_) => return Err(ErrorKind::VolumeInaccessible),
        };

        if metadata.len() != NEONDB_FILE_SIZE {
            return Err(ErrorKind::VolumeInvalidSize);
        }
        return Ok(());
    }
}
