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

    fn mark_block(&mut self, index: usize, vol: &mut File) {
        debug_assert!(self.blocks[index].is_used);

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

    // Menandai block dengan posisi index sebelum index yang diberikan,
    // dimana block tersebut bukanlah sebuah block kosong.
    fn mark_block_before(&mut self, index: usize, vol: &mut File) {
        let prev_block = self
            .blocks
            .iter()
            .enumerate()
            .filter(|(i, b)| *i < index && b.is_used)
            .max_by_key(|(i, _)| *i);

        if let Some((i, _)) = prev_block {
            self.mark_block(i, vol);
        }
    }

    fn find_unused_block_index(&self, size: u64) -> Option<usize> {
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

    fn merge_unused_blocks(&mut self, start_index: usize) {
        debug_assert!(!self.blocks[start_index].is_used);
        debug_assert!(start_index < self.blocks.len());

        let mut new_size = self.blocks[start_index].size;

        for i in start_index + 1..self.blocks.len() {
            if self.blocks[start_index + i].is_used {
                break;
            }

            let block = self.blocks.remove(start_index + i);
            new_size += block.size;
        }

        self.blocks[start_index].size = new_size;
    }

    // Ok jika blok bukan merupakan blok kosong, dan Err
    // jika sebaliknya
    fn find_block_index(&self, address: u64) -> Result<usize> {
        self.blocks
            .binary_search_by_key(&address, |b| b.address)
            .and_then(|i| {
                if self.blocks[i].is_used {
                    Ok(i)
                } else {
                    Err(i)
                }
            })
            .map_err(|_| ErrorKind::BlockNotFound)
    }

    fn free_block(&mut self, index: usize) {
        self.blocks[index].is_used = false;

        if index > 0 && !self.blocks[index - 1].is_used {
            self.merge_unused_blocks(index - 1);
        } else {
            self.merge_unused_blocks(index);
        }
    }
}

impl Allocator for RSSAllocator {
    fn alloc(&mut self, vol: &mut File, size: usize) -> Result<u64> {
        let size: u64 = size.try_into().unwrap();
        let real_size = size + RSSBlock::BLOCK_META_SIZE;

        let i = match self.find_unused_block_index(real_size) {
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

        self.mark_block_before(i, vol);
        self.mark_block(i, vol);

        let abstract_address = address + RSSBlock::BLOCK_META_SIZE;
        Ok(abstract_address)
    }

    fn dealloc(&mut self, vol: &mut File, address: u64) -> Result<()> {
        let real_address = address - RSSBlock::BLOCK_META_SIZE;
        let index = self.find_block_index(real_address)?;

        self.free_block(index);
        self.mark_block_before(index, vol);

        Ok(())
    }

    fn blocks(&self, _vol: &mut File) -> Vec<Block> {
        self.blocks
            .iter()
            .filter(|b| b.is_used)
            .map(|b| Block {
                // abstraksi
                address: b.address + RSSBlock::BLOCK_META_SIZE,
                size: b.size - RSSBlock::BLOCK_META_SIZE,
            })
            .collect::<Vec<Block>>()
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
