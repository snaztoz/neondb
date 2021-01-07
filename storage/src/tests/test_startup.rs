use super::*;
use crate::Storage;

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

        s.blocks().unwrap()
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
