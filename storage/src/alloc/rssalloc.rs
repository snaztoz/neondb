use super::*;

use std::cmp::Ordering;
use std::convert::TryInto;

/// Really-Simple Storage Allocator.
///
/// Mengimplementasikan alokasi storage dengan menggunakan struktur
/// data linked-list sebagai inti utamanya.
pub struct RSSAllocator {
    blocks: Vec<RSSBlock>,
}

impl RSSAllocator {
    // BLOCK_META ada di bagian awal dari tiap blok, yang berguna
    // untuk menyimpan data mengenai blok tersebut.
    //
    // 8 byte untuk panjang blok, dan
    // 8 byte sisanya untuk alamat blok selanjutnya
    const BLOCK_META_SIZE: u64 = 16;

    pub fn new() -> RSSAllocator {
        let blocks = vec![RSSBlock {
            address: NEONDB_FILE_ALLOCATABLE_START,
            size: NEONDB_FILE_ALLOCATABLE_SIZE,
            is_used: false,
        }];

        RSSAllocator { blocks }
    }
}

impl Allocator for RSSAllocator {
    fn alloc(&mut self, vol: &mut File, size: usize) -> Result<u64> {
        todo!()
    }

    fn dealloc(&mut self, _vol: &mut File, address: u64) -> Result<()> {
        todo!()
    }

    fn blocks(&self, _vol: &mut File) -> Vec<Block> {
        todo!()
    }
}

struct RSSBlock {
    address: u64,
    size: u64,
    is_used: bool,
}

// ?todo
//
// Hapus method berikut. Gantikan oleh operator asli dari volume.
use std::io::{prelude::*, SeekFrom};
fn temp_write(vol: &mut File, address: u64, bytes: &[u8]) {
    vol.seek(SeekFrom::Start(address)).expect("seeking address");
    vol.write(bytes).expect("writing bytes");
}
