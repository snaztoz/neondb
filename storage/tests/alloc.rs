use storage::{ErrorKind, Storage};

use serial_test::serial;

mod util;

#[test]
#[serial]
fn alloc_block() {
    assert!({
        let mut s = Storage::new();
        s.mount(path_of!("tmp/test.neondb"))
            .expect("mounting test volume");

        let res = s.alloc(20); // alokasi 20 byte

        res.is_ok()
    });
}

#[test]
#[serial]
fn alloc_exceeding_volume_cap() {
    assert!({
        let mut s = Storage::new();
        s.mount(path_of!("tmp/test.neondb"))
            .expect("mounting test volume");

        let res = s.alloc(1 << 32);

        matches!(res, Err(ErrorKind::VolumeNotEnoughSpace))
    });
}
