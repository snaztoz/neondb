use super::{ErrorKind, Result, NEONDB_FILE_ALLOCATABLE_SIZE, NEONDB_FILE_ALLOCATABLE_START};

use std::fs::File;

pub trait Allocator {
    fn alloc(&mut self, vol: &mut File, size: usize) -> Result<u64>;
    fn dealloc(&mut self, vol: &mut File, address: u64) -> Result<()>;
}

struct Block {
    address: u64,
    size: u64,
}

pub mod simple_allocator;