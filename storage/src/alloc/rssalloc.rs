use super::*;

use std::convert::TryInto;

/// Really-Simple Storage Allocator.
///
/// Mengimplementasikan alokasi storage dengan menggunakan struktur
/// data linked-list sebagai inti utamanya.
pub struct RSSAllocator {
    blocks: Vec<RSSBlock>,
}

impl RSSAllocator {
    pub fn new() -> RSSAllocator {
        let blocks = vec![RSSBlock {
            address: NEONDB_FILE_ALLOCATABLE_START,
            size: NEONDB_FILE_ALLOCATABLE_SIZE,
            is_used: false,
        }];

        RSSAllocator { blocks }
    }

    fn mark_block(&mut self, vol: &mut File, index: usize) {
        let next_block_address = self.blocks.get(index).map_or(
            0, // null address
            |b| b.address,
        );

        temp_write(
            vol,
            self.blocks[index].address,
            &self.blocks[index].construct_meta(next_block_address),
        );
    }

    fn find_unused_block(&self, size: u64) -> Option<usize> {
        self.blocks
            .iter()
            .enumerate()
            .filter(|(_, b)| !b.is_used)
            .min_by_key(|(_, b)| b.size >= size)
            .and_then(|(i, _)| Some(i))
    }

    // Hanya mengambil bagian dari blok kosong, tetapi belum dilakukan
    // reservasi blok baru (jika seandainya memang demikian)
    fn get_unused_block(&mut self, index: usize, size: u64) -> u64 {
        debug_assert!(!self.blocks[index].is_used);

        let address = self.blocks[index].address;

        self.blocks[index].size -= size;
        if self.blocks[index].size == 0 {
            self.blocks.remove(index + 1);
        } else {
            self.blocks[index].address += size;
        }

        address
    }
}

impl Allocator for RSSAllocator {
    fn alloc(&mut self, vol: &mut File, size: usize) -> Result<u64> {
        let size: u64 = size.try_into().unwrap();
        let real_size = size + RSSBlock::BLOCK_META_SIZE;

        let i = match self.find_unused_block(real_size) {
            Some(index) => index,
            None => return Err(ErrorKind::VolumeNotEnoughSpace),
        };

        let address = self.get_unused_block(i, real_size);
        self.blocks.insert(
            i,
            RSSBlock {
                address,
                size: real_size,
                is_used: true,
            },
        );

        if i > 0 {
            // update blok sebelumnya
            self.mark_block(vol, i - 1);
        }
        self.mark_block(vol, i);

        let abstract_address = address + RSSBlock::BLOCK_META_SIZE;
        Ok(abstract_address)
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

impl RSSBlock {
    // BLOCK_META ada di bagian awal dari tiap blok, yang berguna
    // untuk menyimpan data mengenai blok tersebut.
    //
    // 8 byte untuk panjang blok, dan
    // 8 byte sisanya untuk alamat blok selanjutnya
    const BLOCK_META_SIZE: u64 = 16;

    fn construct_meta(&self, next_block_address: u64) -> Vec<u8> {
        self.size
            .to_be_bytes()
            .iter()
            .chain(&next_block_address.to_be_bytes())
            .map(|b| *b)
            .collect::<Vec<u8>>()
    }
}

// ?todo
//
// Hapus method berikut. Gantikan oleh operator asli dari volume.
use std::io::{prelude::*, SeekFrom};
fn temp_write(vol: &mut File, address: u64, bytes: &[u8]) {
    vol.seek(SeekFrom::Start(address)).expect("seeking address");
    vol.write(bytes).expect("writing bytes");
}
