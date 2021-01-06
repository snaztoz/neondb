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
