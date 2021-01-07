use super::*;
use crate::{alloc::Block, Storage};

use serial_test::serial;

fn init_storage() -> Storage {
    util::fresh_volume(path_of!("tmp/storage/startup.neondb"));

    let mut s = Storage::new();
    s.mount(path_of!("tmp/storage/startup.neondb")).unwrap();

    s
}

#[test]
#[serial]
fn startup_with_blocks_exist() {
    let blocks = {
        let mut s = init_storage();

        for _ in 0..5 {
            s.alloc(64).unwrap();
        }

        s.blocks()
            .unwrap()
            .iter()
            .map(|b| Block { ..*b })
            .collect::<Vec<Block>>()
    }; // drop s terlebih dulu

    assert!({
        let mut s = Storage::new();
        s.mount(path_of!("tmp/storage/startup.neondb")).unwrap();

        blocks == s.blocks().unwrap()
    });

    // Coba alokasi kembali setelah startup
    assert!({
        let mut s = Storage::new();
        s.mount(path_of!("tmp/storage/startup.neondb")).unwrap();

        let address = s.alloc(64).unwrap();

        s.blocks().unwrap().last().unwrap().address == address
    });
}

#[test]
#[serial]
fn startup_after_dealloc_blocks() {
    let blocks = {
        let mut s = init_storage();
        let mut dealloc_addresses = vec![];

        for i in 0..10 {
            let address = s.alloc(64).unwrap();
            if i == 0 || i == 9 {
                continue;
            }
            dealloc_addresses.push(address);
        }

        // buat gap di antara blok awal dengan akhir
        for address in dealloc_addresses {
            s.dealloc(address).unwrap();
        }

        s.blocks()
            .unwrap()
            .iter()
            .map(|b| Block { ..*b })
            .collect::<Vec<Block>>()
    }; // drop s terlebih dulu

    assert!({
        let mut s = Storage::new();
        s.mount(path_of!("tmp/storage/startup.neondb")).unwrap();

        blocks == s.blocks().unwrap()
    });

    // Coba alokasi kembali setelah startup
    assert!({
        let mut s = Storage::new();
        s.mount(path_of!("tmp/storage/startup.neondb")).unwrap();

        let address = s.alloc(64 * 9).unwrap();

        s.blocks().unwrap()[1].address == address
    });
}
