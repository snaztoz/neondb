use storage::{ErrorKind, Storage};

use std::path::Path;

macro_rules! path_of {
    ($p: expr) => {{
        let full_path = concat!("../", $p);
        Path::new(full_path)
    }};
}

#[test]
fn mount_non_existing_volume() {
    assert!({
        let mut s = Storage::new();
        let res = s.mount(path_of!("tmp/test.neondb"));

        if let Err(ErrorKind::VolumeNotFound) = res {
            true
        } else {
            false
        }
    });
}

#[test]
fn mount_invalid_ext() {
    assert!({
        let mut s = Storage::new();
        let res = s.mount(path_of!("tmp/invalid.txt"));

        if let Err(ErrorKind::VolumeInvalidExt) = res {
            true
        } else {
            false
        }
    })
}

#[test]
fn mount_invalid_size() {
    assert!({
        let mut s = Storage::new();
        let res = s.mount(path_of!("tmp/invalid.neondb"));

        if let Err(ErrorKind::VolumeInvalidSize) = res {
            true
        } else {
            false
        }
    });
}