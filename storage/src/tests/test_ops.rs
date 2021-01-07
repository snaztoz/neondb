use super::*;
use crate::{ErrorKind, Storage};

use serial_test::serial;

use std::fs::File;
use std::io::{prelude::*, SeekFrom};

fn init_storage() -> Storage {
    util::fresh_volume(path_of!("tmp/storage/ops.neondb"));

    let mut s = Storage::new();
    s.mount(path_of!("tmp/storage/ops.neondb")).unwrap();

    s
}

#[test]
#[serial]
fn read_block_bytes() {
    let mut s = init_storage();
    let mut buff = [0u8; 16];
    let address = s.alloc(64).unwrap();

    assert!({
        let res = s.read(address, &mut buff);

        if res.is_err() {
            panic!("error at reading volume head's bytes");
        }

        let n = res.unwrap();
        if n != 16 {
            panic!("bytes read length mismatch");
        }

        true
    });

    // coba tulis beberapa byte
    let mut vol = File::create(path_of!("tmp/storage/ops.neondb")).unwrap();
    vol.seek(SeekFrom::Start(address))
        .and_then(|_| vol.write(&[1u8; 16]))
        .unwrap();

    assert!({
        let res = s.read(address, &mut buff);

        if res.is_err() {
            panic!("error at reading volume head's bytes");
        }

        buff == [1u8; 16]
    });
}

#[test]
#[serial]
fn read_at_illegal_address() {
    assert!({
        let mut s = init_storage();
        let mut buff = [0u8; 16];

        // alamat acak
        let res = s.read(234653, &mut buff);

        matches!(res, Err(ErrorKind::BlockNotFound))
    });
}
