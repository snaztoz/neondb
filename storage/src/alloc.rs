use super::{ErrorKind, Result, NEONDB_FILE_ALLOCATABLE_SIZE, NEONDB_FILE_ALLOCATABLE_START};

use std::fs::File;

pub trait Allocator {
    fn alloc(&mut self, vol: &mut File, size: usize) -> Result<u64>;
    fn dealloc(&mut self, vol: &mut File, address: u64) -> Result<()>;

    // Kedua method di atas tidak dapat dijalankan jika allocator
    // belum diinisialisasikan terlebih dulu.
    fn init(&mut self, vol: &mut File) -> Result<Vec<Block>>;
    fn init_new(&mut self, vol: &mut File) -> Result<()>;

    fn blocks(&self, vol: &mut File) -> Vec<Block>;
    fn reset(&mut self);
}

pub struct Block {
    address: u64,
    size: u64,
}

pub mod rssalloc;
