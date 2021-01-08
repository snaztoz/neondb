use super::*;
use crate::{ErrorKind, Storage};

use serial_test::serial;

use std::fs::OpenOptions;
use std::io::{prelude::*, SeekFrom};

fn init_storage() -> Storage {
    util::fresh_volume(path_of!("tmp/storage/ops.neondb"));

    let mut s = Storage::new();
    s.mount(path_of!("tmp/storage/ops.neondb")).unwrap();

    s
}

fn write_ones(address: u64, size: usize) {
    let mut vol = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path_of!("tmp/storage/ops.neondb"))
        .unwrap();

    vol.seek(SeekFrom::Start(address))
        .and_then(|_| vol.write(&vec![1u8; size]))
        .unwrap();
}

#[test]
#[serial]
fn read_block_bytes() {
    let mut s = init_storage();
    let mut buff = [0u8; 16];
    let address = s.alloc(64).unwrap();

    assert!({
        let res = s.read(address, &mut buff);

        matches!(res, Ok(n) if n == 16)
    });

    write_ones(address, 16);

    assert!({
        let res = s.read(address, &mut buff);

        if res.is_err() {
            panic!("error at reading bytes");
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

    assert!({
        let mut s = init_storage();
        let mut buff = [0u8; 1];

        let address = s.alloc(64).unwrap();

        let res = s.read(address + 64, &mut buff);

        res.is_err()
    });
}

#[test]
#[serial]
fn read_truncated() {
    assert!({
        let mut s = init_storage();
        let mut buff = [0u8; 64];

        let address = s.alloc(64).unwrap();
        write_ones(address, 64);

        // membaca mulai dari posisi tengah blok
        let res = s.read(address + 32, &mut buff);

        if !matches!(res, Ok(n) if n == 32) {
            panic!("reading failed")
        }

        // 1 sebanyak 32 kali, lalu 0 sebanyak 32 kali
        buff[..32] == [1u8; 32] && buff[32..] == [0u8; 32]
    });

    assert!({
        let mut s = init_storage();
        let mut buff = [0u8; 64];

        // buat dua blok yang berjejeran, dan kedua blok tersebut
        // berisikan byte 1
        let address_one = s.alloc(64).unwrap();
        let address_two = s.alloc(64).unwrap();
        write_ones(address_one, 64);
        write_ones(address_two, 64);

        let res = s.read(address_one + 32, &mut buff);

        if !matches!(res, Ok(n) if n == 32) {
            panic!("reading failed")
        }

        // 1 sebanyak 32 kali, lalu 0 sebanyak 32 kali
        buff[..32] == [1u8; 32] && buff[32..] == [0u8; 32]
    })
}

/**
 * Karena operasi write kebanyakan mirip dengan yang ada pada
 * operasi read, maka test yang dibuat tidak terlalu mendetail.
 */

#[test]
#[serial]
fn write_block_bytes() {
    let mut s = init_storage();
    let text = "some text here";
    let address = s.alloc(64).unwrap();

    assert!({
        let res = s.write(address, &text.as_bytes());

        matches!(res, Ok(n) if n == text.len())
    });

    assert!({
        let mut buff = vec![0u8; text.len()];
        s.read(address, &mut buff).unwrap();

        &buff == &text.as_bytes()
    });
}

#[test]
#[serial]
fn write_at_illegal_address() {
    assert!({
        let mut s = init_storage();
        let text = "some random text";

        // alamat acak
        let res = s.write(234653, &text.as_bytes());

        matches!(res, Err(ErrorKind::BlockNotFound))
    });
}

#[test]
#[serial]
fn write_truncated() {
    let mut s = init_storage();
    let text = "truncated string";

    let si = text.find(' ').unwrap();
    let address = s.alloc(si).unwrap();

    assert!({
        let res = s.write(address, &text.as_bytes());

        matches!(res, Ok(n) if n == si)
    });

    assert!({
        let mut buff = vec![0u8; si];
        s.read(address, &mut buff).unwrap();

        buff == &text.as_bytes()[..si]
    });
}
