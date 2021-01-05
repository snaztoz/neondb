use super::*;
use rssblock::RSSBlock;

use std::convert::TryInto;

mod init;
mod rssblock;

/// Really-Simple Storage Allocator.
///
/// Mengimplementasikan alokasi storage dengan menggunakan struktur
/// data linked-list sebagai inti utamanya.
pub struct RSSAllocator {
    blocks: Vec<RSSBlock>,
    is_initialized: bool,
}

impl RSSAllocator {
    pub fn new() -> RSSAllocator {
        // Blok-blok yang ada di dalam volume belum dapat dideteksi,
        // oleh karenanya perlu dilakukan inisialisasi terlebih dulu
        // sebelum alokator mulai digunakan.
        let blocks = vec![];

        RSSAllocator {
            blocks,
            is_initialized: false,
        }
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
            .filter(|(_, b)| !b.is_used && b.size >= size)
            .min_by_key(|(_, b)| b.size)
            .and_then(|(i, _)| Some(i))
    }

    // Hanya mengambil bagian dari blok kosong, tetapi belum dilakukan
    // reservasi blok baru (jika seandainya memang demikian)
    fn get_unused_block(&mut self, index: usize, size: u64) -> u64 {
        debug_assert!(!self.blocks[index].is_used);
        debug_assert!(self.blocks[index].size >= size);

        let address = self.blocks[index].address;

        self.blocks[index].size -= size;

        if self.blocks[index].size == 0 {
            self.blocks.remove(index);
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

    fn find_used_block_index(&self, address: u64) -> Option<usize> {
        self.blocks
            .binary_search_by_key(&address, |b| b.address)
            .and_then(|i| {
                if self.blocks[i].is_used {
                    Ok(i)
                } else {
                    Err(i)
                }
            })
            .ok()
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

        let i = self
            .find_unused_block_index(real_size)
            .ok_or_else(|| ErrorKind::VolumeNotEnoughSpace)?;

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

        let i = self
            .find_used_block_index(real_address)
            .ok_or_else(|| ErrorKind::BlockNotFound)?;

        self.free_block(i);
        self.mark_block_before(i, vol);

        Ok(())
    }

    fn init(&mut self, vol: &mut File) -> Result<Block> {
        todo!();
    }

    fn init_new(&mut self, vol: &mut File) -> Result<()> {
        todo!();
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

// ?todo
//
// Hapus method berikut. Gantikan oleh operator asli dari volume.
use std::io::{prelude::*, SeekFrom};
fn temp_write(vol: &mut File, address: u64, bytes: &[u8]) {
    vol.seek(SeekFrom::Start(address)).expect("seeking address");
    vol.write(bytes).expect("writing bytes");
}

fn temp_read(vol: &mut File, address: u64, buff: &mut [u8]) {
    vol.seek(SeekFrom::Start(address)).expect("seeking address");
    vol.read(buff).expect("reading bytes");
}
