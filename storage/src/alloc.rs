use super::{ErrorKind, Result, NEONDB_FILE_ALLOCATABLE_SIZE, NEONDB_FILE_ALLOCATABLE_START};

use std::fs::File;

pub trait Allocator {
    fn alloc(&mut self, vol: &mut File, size: usize) -> Result<u64>;
    fn dealloc(&mut self, vol: &mut File, address: u64) -> Result<()>;
    fn init(&mut self, vol: &mut File) -> Result<Block>;
    fn init_new(&mut self, vol: &mut File) -> Result<()>;
    fn blocks(&self, vol: &mut File) -> Vec<Block>;
}

pub struct Block {
    address: u64,
    size: u64,
}

pub mod rssalloc;
