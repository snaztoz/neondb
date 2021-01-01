use super::{ErrorKind, Result, NEONDB_FILE_ALLOCATABLE_SIZE, NEONDB_FILE_ALLOCATABLE_START};

use std::cmp::Ordering;
use std::convert::TryInto;
use std::fs::File;

// 8 byte untuk panjang blok, dan
// 8 byte sisanya untuk alamat blok selanjutnya
const BLOCK_META_SIZE: u64 = 16;

pub struct Allocator {
    // Kedua vector ini merupakan refleksi antar satu sama lain.
    //
    // Vector blocks memegang urutan utama. Yakni, ia terurut berdasarkan
    // alamat. Sedangkan untuk free_blocks terurut berdasarkan ukurannya,
    // sehingga dapat mempermudah pencarian blok kosong.
    blocks: Vec<Block>,
    free_blocks: Vec<Block>,
}

impl Allocator {
    pub fn new() -> Self {
        // ?optimize
        //
        // Apakah 10 cukup? Jika operasi pengalokasian blok dapat dilakukan
        // dengan baik (yakni defragmentasi yang ada diminimalisir), maka
        // ukuran ini cukup.
        //
        // Tentu, vector bisa melakukan re-grow. Tapi bukankah lebih baik
        // jika ia tidak perlu melakukan regrow sama sekali?
        let mut free_blocks = Vec::with_capacity(10);

        // inisialisasi, semua blok dianggap kosong
        free_blocks.push(Block {
            address: NEONDB_FILE_ALLOCATABLE_START,
            size: NEONDB_FILE_ALLOCATABLE_SIZE,
        });

        Allocator {
            blocks: Vec::with_capacity(20),
            free_blocks,
        }
    }

    pub fn alloc(&mut self, vol: &mut File, size: usize) -> Result<u64> {
        let size: u64 = size.try_into().unwrap();

        let address = self.take_free_block(BLOCK_META_SIZE + size)?;
        let new_block_index = self.find_block_index(address);

        self.blocks.insert(new_block_index, Block { address, size });
        self.mark_block(vol, new_block_index);

        Ok(address)
    }

    // Tidak peduli apakah blok tersebut kosong atau tidak
    fn find_block_index(&self, address: u64) -> usize {
        let search_res = self.blocks.binary_search_by(|b| {
            if b.address <= address && address < b.address + b.size {
                Ordering::Equal
            } else if address < b.address {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        });

        match search_res {
            Ok(i) | Err(i) => i,
        }
    }

    // Mengambil blok kosong dengan ukuran yang diberikan
    fn take_free_block(&mut self, size: u64) -> Result<u64> {
        let (i, free_block) = self.find_free_block(size)?;
        let free_block_addr = free_block.address;

        debug_assert!(size <= self.free_blocks[i].size);

        if size < self.free_blocks[i].size {
            // majukan saja addressnya
            self.free_blocks[i].address += size;
            self.free_blocks[i].size -= size;
            self.free_blocks.sort_by_key(|b| b.size);
        } else {
            self.free_blocks.remove(i);
        }

        Ok(free_block_addr)
    }

    // Mencari index sebuah blok kosong dari free_blocks
    fn find_free_block(&self, size: u64) -> Result<(usize, &Block)> {
        // ?optimize
        //
        // Pencarian blok kosong dilakukan secara linear. Hal ini dilakukan
        // karena bookkeeping blok kosong yang ada menggunakan vector yang
        // selalu terurut (baca di bagian deklarasi struct Allocator), sehingga
        // blok yang dapat digunakan selalu dicari dari yang terkecil
        // (smallest-sufficient block).
        //
        // Sehingga, algoritma ini sepertinya cukup baik untuk digunakan.
        // (sepertinya)
        let free_block = self
            .free_blocks
            .iter()
            .enumerate()
            .find(|(_, b)| b.size >= size);

        free_block.ok_or_else(|| ErrorKind::VolumeNotEnoughSpace)
    }

    fn mark_block(&mut self, vol: &mut File, index: usize) {
        todo!()
    }
}

struct Block {
    address: u64,
    size: u64,
}
