use super::*;
use crate::{ErrorKind, Storage};

use serial_test::serial;

#[test]
#[serial]
fn mount_valid_volume() {
    assert!({
        let p = path_of!("tmp/storage/test.neondb");
        util::fresh_volume(p);

        let mut s = Storage::new();
        let res = s.mount(p);

        res.is_ok()
    });
}

#[test]
fn mount_non_existing_volume() {
    assert!({
        let mut s = Storage::new();
        let res = s.mount(path_of!("tmp/storage/non_existing.neondb"));

        matches!(res, Err(ErrorKind::VolumeNotFound))
    });
}

#[test]
fn mount_invalid_ext() {
    assert!({
        let mut s = Storage::new();
        let res = s.mount(path_of!("tmp/storage/invalid.txt"));

        matches!(res, Err(ErrorKind::VolumeInvalidExt))
    })
}

#[test]
fn mount_invalid_size() {
    assert!({
        let mut s = Storage::new();
        let res = s.mount(path_of!("tmp/storage/invalid.neondb"));

        matches!(res, Err(ErrorKind::VolumeInvalidSize))
    });
}

#[test]
#[serial]
fn unmount_volume() {
    assert!({
        let p = path_of!("tmp/storage/test.neondb");
        util::fresh_volume(p);

        let mut s = Storage::new();
        s.mount(p).unwrap();

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

#[test]
#[serial]
fn mount_new_volume() {
    let p = path_of!("tmp/storage/mount_new.neondb");

    assert!({
        let mut s = Storage::new();
        util::ensure_not_exists(p);

        let res = s.mount_new(p);

        res.is_ok()
    });

    assert!({
        let mut s = Storage::new();
        s.mount(p).unwrap();

        s.blocks().unwrap().is_empty()
    });
}
