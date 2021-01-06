use super::*;

use serial_test::serial;

use crate::{ErrorKind, Storage};

#[test]
#[serial]
fn mount_valid_volume() {
    assert!({
        util::fresh_volume();

        let mut s = Storage::new();
        let res = s.mount(path_of!("tmp/test.neondb"));

        res.is_ok()
    });
}

#[test]
fn mount_non_existing_volume() {
    assert!({
        let mut s = Storage::new();
        let res = s.mount(path_of!("tmp/non_existing.neondb"));

        matches!(res, Err(ErrorKind::VolumeNotFound))
    });
}

#[test]
fn mount_invalid_ext() {
    assert!({
        let mut s = Storage::new();
        let res = s.mount(path_of!("tmp/invalid.txt"));

        matches!(res, Err(ErrorKind::VolumeInvalidExt))
    })
}

#[test]
fn mount_invalid_size() {
    assert!({
        let mut s = Storage::new();
        let res = s.mount(path_of!("tmp/invalid.neondb"));

        matches!(res, Err(ErrorKind::VolumeInvalidSize))
    });
}

#[test]
#[serial]
fn unmount_volume() {
    assert!({
        let mut s = Storage::new();
        s.mount(path_of!("tmp/test.neondb")).unwrap();

        let res = s.unmount();

        res.is_ok()
    });
}

#[test]
fn unmount_non_existing_volume() {
    assert!({
        let mut s = Storage::new();

        let res = s.unmount();

        matches!(res, Err(ErrorKind::VolumeNotFound))
    });
}
