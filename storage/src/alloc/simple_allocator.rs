use super::*;

use std::cmp::Ordering;
use std::convert::TryInto;

/// Allocator data di dalam volume NeonDB yang menggunakan struktur
/// data linked-list sebagai inti dari metode alokasinya.
pub struct SimpleAllocator {
    // Kedua vector ini merupakan refleksi antar satu sama lain.
    //
    // Vector blocks memegang urutan utama. Yakni, ia terurut berdasarkan
    // alamat. Sedangkan untuk free_blocks terurut berdasarkan ukurannya,
    // sehingga dapat mempermudah pencarian blok kosong.
    //
    // Diharapkan berhati-hati dalam penggunaan Block ini agar tidak tertukar
    // antara alamat/size yang asli dengan yang sudah terabstraksi, dimana yang
    // akan ditampilkan ke dunia luar adalah yang terabstraksi.
    //
    // (dimana yang asli memasukkan BLOCK_META ke dalam perhitungannya, sedangkan
    // untuk yang sudah diabstraksikan tidak)
    blocks: Vec<Block>,
    free_blocks: Vec<Block>,
}

impl SimpleAllocator {
    // BLOCK_META ada di bagian awal dari tiap blok, yang berguna
    // untuk menyimpan data mengenai blok tersebut.
    //
    // 8 byte untuk panjang blok, dan
    // 8 byte sisanya untuk alamat blok selanjutnya
    const BLOCK_META_SIZE: u64 = 16;

    pub fn new() -> Self {
        // ?optimize
        //
        // Apakah 10 cukup? Jika operasi pengalokasian blok dapat dilakukan
        // dengan baik (yakni fragmentasi yang ada diminimalisir), maka
        // ukuran ini sudah cukup.
        //
        // Tentu, vector bisa melakukan re-grow. Tapi bukankah lebih baik
        // jika ia tidak perlu melakukan regrow sama sekali?
        let mut free_blocks = Vec::with_capacity(10);

        // inisialisasi, semua blok dianggap kosong
        free_blocks.push(Block {
            address: NEONDB_FILE_ALLOCATABLE_START,
            size: NEONDB_FILE_ALLOCATABLE_SIZE,
        });

        SimpleAllocator {
            blocks: Vec::with_capacity(20),
            free_blocks,
        }
    }

    // Ok jika block sudah direservasi, Err jika belum.
    // Keduanya berisikan index posisi dari block.
    fn find_block_index(&self, raw_address: u64) -> std::result::Result<usize, usize> {
        self.blocks.binary_search_by(|b| {
            if b.address <= raw_address && raw_address < b.address + b.size {
                Ordering::Equal
            } else if raw_address < b.address {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        })
    }

    // Mengambil blok kosong dengan ukuran yang diberikan
    fn take_free_block(&mut self, size: u64) -> Result<u64> {
        let i = self.find_free_block_index(size)?;
        let address = self.free_blocks[i].address;

        debug_assert!(size <= self.free_blocks[i].size);

        if size < self.free_blocks[i].size {
            // majukan saja addressnya
            self.free_blocks[i].address += size;
            self.free_blocks[i].size -= size;
            self.free_blocks.sort_by_key(|b| b.size);
        } else {
            self.free_blocks.remove(i);
        }

        Ok(address)
    }

    fn find_free_block_index(&self, size: u64) -> Result<usize> {
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
        self.free_blocks
            .iter()
            .position(|b| b.size >= size)
            .ok_or_else(|| ErrorKind::VolumeNotEnoughSpace)
    }

    fn mark_block(&mut self, vol: &mut File, index: usize) {
        // ?todo
        //
        // Gunakan mekanisme dari operator volume.

        // update meta block sebelumnya
        if index > 0 {
            temp_write(
                vol,
                self.blocks[index - 1].address,
                &self.construct_block_meta(index - 1),
            );
        }

        temp_write(
            vol,
            self.blocks[index].address,
            &self.construct_block_meta(index),
        );
    }

    fn construct_block_meta(&self, index: usize) -> Vec<u8> {
        let next_block_address = self.blocks.get(index + 1).map_or(
            0, // null address
            |b| b.address,
        );

        self.blocks[index]
            .size
            .to_be_bytes()
            .iter()
            .chain(&next_block_address.to_be_bytes())
            .map(|byte| *byte)
            .collect::<Vec<u8>>()
    }
}

impl Allocator for SimpleAllocator {
    fn alloc(&mut self, vol: &mut File, size: usize) -> Result<u64> {
        let size: u64 = size.try_into().unwrap();

        let raw_address = self.take_free_block(SimpleAllocator::BLOCK_META_SIZE + size)?;
        let block_index = self
            .find_block_index(raw_address)
            .expect_err("expecting the block is still free");

        self.blocks.insert(
            block_index,
            Block {
                address: raw_address,
                size,
            },
        );
        self.mark_block(vol, block_index);

        let abstract_address = raw_address + SimpleAllocator::BLOCK_META_SIZE;
        Ok(abstract_address)
    }

    fn dealloc(&mut self, _vol: &mut File, _address: u64) -> Result<()> {
        todo!();
    }

    fn blocks(&self, _vol: &mut File) -> Vec<Block> {
        // Data mengenai blok yang disimpan oleh allocator haruslah di-
        // abstraksikan terlebih dahulu agar user tidak perlu mengetahui
        // adanya keberadaan dari BLOCK_META.
        self.blocks
            .iter()
            .map(|raw_block| Block {
                address: raw_block.address + SimpleAllocator::BLOCK_META_SIZE,
                size: raw_block.size - SimpleAllocator::BLOCK_META_SIZE,
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
