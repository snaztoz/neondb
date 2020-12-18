use storage::Storage;

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

        res.is_err()
    });
}

#[test]
fn mount_invalid_ext() {
    assert!({
        let mut s = Storage::new();
        let res = s.mount(path_of!("tmp/invalid.txt"));

        res.is_err()
    })
}

#[test]
fn mount_invalid_size() {
    assert!({
        let mut s = Storage::new();
        let res = s.mount(path_of!("tmp/invalid.neondb"));

        res.is_err()
    });
}
