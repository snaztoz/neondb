use super::*;
use crate::alloc::Block;

use std::cmp::Ordering;
use std::convert::TryInto;
use std::fs::File;
use std::io::{prelude::*, SeekFrom};

pub enum Ops {}

impl Ops {
    fn find_block_index_of(address: u64, blocks: &[Block]) -> Result<usize> {
        blocks
            .binary_search_by(|b| {
                if b.address <= address && address < b.address + b.size {
                    Ordering::Equal
                } else if address < b.address {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            })
            .map_err(|_| ErrorKind::BlockNotFound)
    }

    pub fn max_operation_len_at(address: u64, blocks: &[Block]) -> Result<usize> {
        Ops::find_block_index_of(address, blocks).and_then(|i| {
            let block = &blocks[i];
            let max_len = block.size - (address - block.address);

            Ok(max_len.try_into().unwrap())
        })
    }

    pub fn read(address: u64, buff: &mut [u8], vol: &mut File) -> usize {
        vol.seek(SeekFrom::Start(address))
            .and_then(|_| vol.read(buff))
            .expect("reading bytes from volume")
    }

    pub fn write(address: u64, buff: &[u8], vol: &mut File) -> usize {
        vol.seek(SeekFrom::Start(address))
            .and_then(|_| vol.write(buff))
            .expect("writing bytes to volume")
    }
}
