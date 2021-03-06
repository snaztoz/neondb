use super::*;
use crate::ops::Ops;
use rssblock::RSSBlock;

use std::convert::TryInto;

const NULL_ADDRESS: u64 = 0;

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

        let next_block_address = self
            .find_used_block_address_after(index)
            .or_else(|| Some(NULL_ADDRESS))
            .unwrap();

        Ops::write(
            self.blocks[index].address,
            &self.blocks[index].construct_meta(next_block_address),
            vol,
        );
    }

    // Menandai block dengan posisi index sebelum index yang diberikan,
    // dimana block tersebut bukanlah sebuah block kosong.
    fn mark_block_before(&mut self, index: usize, vol: &mut File) {
        let prev_block_index = &self.blocks[..index].iter().rposition(|b| b.is_used);

        if let Some(i) = prev_block_index {
            self.mark_block(*i, vol);
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
        let i = start_index + 1;

        loop {
            if i == self.blocks.len() || self.blocks[i].is_used {
                break;
            }

            let block = self.blocks.remove(i);
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

    fn find_used_block_address_after(&self, index: usize) -> Option<u64> {
        debug_assert!(index < self.blocks.len());

        self.blocks
            .iter()
            .skip(index + 1)
            .find(|b| b.is_used)
            .and_then(|b| Some(b.address))
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
        if !self.is_initialized {
            return Err(ErrorKind::AllocatorNotInitialized);
        }

        let size: u64 = size.try_into().unwrap();
        let real_size = size + RSSBlock::META_SIZE;

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

        let abstract_address = address + RSSBlock::META_SIZE;
        Ok(abstract_address)
    }

    fn dealloc(&mut self, vol: &mut File, address: u64) -> Result<()> {
        debug_assert!(address != NEONDB_FILE_ALLOCATABLE_START);

        if !self.is_initialized {
            return Err(ErrorKind::AllocatorNotInitialized);
        }

        let real_address = address - RSSBlock::META_SIZE;

        let i = self
            .find_used_block_index(real_address)
            .ok_or_else(|| ErrorKind::BlockNotFound)?;

        self.free_block(i);
        self.mark_block_before(i, vol);

        Ok(())
    }

    fn init(&mut self, vol: &mut File) -> Result<Vec<Block>> {
        // disini sudah didapatkan posisi blok selanjutnya
        let start_address = init::obtain_head(vol, self)?;

        init::scan_blocks(vol, start_address, self)?;

        self.is_initialized = true;
        Ok(self.blocks(vol))
    }

    fn init_new(&mut self, vol: &mut File) -> Result<()> {
        init::new_volume(vol, self)?;

        self.is_initialized = true;
        Ok(())
    }

    fn blocks(&self, _vol: &mut File) -> Vec<Block> {
        self.blocks
            .iter()
            .skip(1) // tidak perlu tampilkan head
            .filter(|b| b.is_used)
            .map(|b| Block {
                // abstraksi
                address: b.address + RSSBlock::META_SIZE,
                size: b.size - RSSBlock::META_SIZE,
            })
            .collect::<Vec<Block>>()
    }

    fn reset(&mut self) {
        self.blocks.clear();
        self.is_initialized = false;
    }
}
