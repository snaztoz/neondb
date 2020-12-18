use super::{Result, NEONDB_FILE_EXT, NEONDB_FILE_SIZE};

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
            return Err("file not exist".to_string());
        }
        Ok(())
    }

    fn is_valid_ext(&self, path: &Path) -> Result<()> {
        let err_msg = format!("the expected extension should be .{}", NEONDB_FILE_EXT);

        if let Some(ext) = path.extension() {
            if ext == NEONDB_FILE_EXT {
                return Ok(());
            }
        }
        return Err(err_msg);
    }

    fn is_valid_size(&self, path: &Path) -> Result<()> {
        let metadata = match path.metadata() {
            Ok(metadata) => metadata,
            Err(_) => return Err("failed to retrieve file's metadata".to_string()),
        };

        if metadata.len() != NEONDB_FILE_SIZE {
            return Err("the file doesn't have the expected size".to_string());
        }
        return Ok(());
    }
}
