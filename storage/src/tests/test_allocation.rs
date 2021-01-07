use super::*;
use crate::Storage;

use serial_test::serial;

fn init_storage() -> Storage {
    util::fresh_volume(path_of!("tmp/storage/allocation.neondb"));

    let mut s = Storage::new();
    s.mount(path_of!("tmp/storage/allocation.neondb")).unwrap();

    s
}

#[test]
#[serial]
fn alloc_one_block() {
    assert!({
        let mut s = init_storage();

        let address = s.alloc(64).unwrap();
        let blocks = s.blocks().unwrap();

        blocks[0].address == address && blocks[0].size == 64
    });
}

#[test]
#[serial]
fn alloc_blocks() {
    assert!({
        let mut s = init_storage();

        for i in 0..3 {
            let address = s.alloc(64).unwrap();
            let block = &s.blocks().unwrap()[i];

            if block.address != address || block.size != 64 {
                panic!("wrong address/size on allocation");
            }
        }

        true
    });
}

#[test]
#[serial]
fn dealloc_one_block() {
    assert!({
        let mut s = init_storage();
        let mut addresses = vec![];
        let mut dealloc_address = 0;

        for i in 0..3 {
            let address = s.alloc(64).unwrap();

            if i == 1 {
                dealloc_address = address;
            } else {
                addresses.push(address);
            }
        }

        s.dealloc(dealloc_address).unwrap();

        for (i, b) in s.blocks().unwrap().iter().enumerate() {
            if b.address != addresses[i] {
                panic!("wrong address on deallocation");
            }
        }

        true
    });
}

#[test]
#[serial]
fn dealloc_last_blocks() {
    assert!({
        let mut s = init_storage();
        let mut dealloc_addresses = vec![];

        for i in 0..5 {
            let address = s.alloc(64).unwrap();
            if i == 0 {
                continue;
            }
            dealloc_addresses.push(address);
        }

        for address in dealloc_addresses {
            s.dealloc(address).unwrap();
        }

        if s.blocks().unwrap().len() != 1 {
            panic!("wrong in deallocation");
        }

        // alokasi kembali dengan ukuran yang cukup besar
        let address = s.alloc(64 * 5).unwrap();

        let rssblock_meta_size = 16;
        let blocks = s.blocks().unwrap();

        // pastikan tidak ada gap
        blocks[0].address + blocks[0].size + rssblock_meta_size == address
    });
}

#[test]
#[serial]
fn dealloc_blocks() {
    // Ketika terdapat 2 atau lebih blok kosong yang berjejeran,
    // maka blok-blok tersebut akan digabungkan menjadi satu.
    //
    // Hal ini akan dibuktikan dengan mencoba mengalokasikan
    // blok yang berukutan (atau hampir) sama dengan jumlah total
    // dari semua blok kosong tersebut.
    assert!({
        let mut s = init_storage();
        let mut addresses = vec![];
        let mut dealloc_addresses = vec![];

        for i in 0..5 {
            let address = s.alloc(64).unwrap();

            if i == 0 || i == 4 {
                addresses.push(address);
            } else {
                dealloc_addresses.push(address);
            }
        }

        for address in dealloc_addresses {
            s.dealloc(address).unwrap();
        }

        for (i, b) in s.blocks().unwrap().iter().enumerate() {
            if b.address != addresses[i] {
                panic!("wrong address on deallocation");
            }
        }

        // Mencoba alokasi blok yang cukup besar di antara
        // kedua blok yang telah ada
        let address = s.alloc(64 * 3).unwrap();

        s.blocks().unwrap()[1].address == address
    });
}
