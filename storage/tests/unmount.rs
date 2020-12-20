use storage::{ErrorKind, Storage};

use serial_test::serial;

mod util;

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

        if let Err(ErrorKind::VolumeNotFound) = res {
            true
        } else {
            false
        }
    });
}
