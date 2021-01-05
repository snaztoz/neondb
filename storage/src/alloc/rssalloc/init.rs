use super::*;

pub fn obtain_head(vol: &mut File, allocator: &mut RSSAllocator) -> Result<u64> {
    let address = NEONDB_FILE_ALLOCATABLE_START;

    let mut buff = [0u8; 16];
    temp_read(vol, address, &mut buff);

    let size = bytes_to_u64(&buff[..8]);
    if size != RSSBlock::BLOCK_META_SIZE {
        return Err(ErrorKind::VolumeCorrupted);
    }

    push_block(allocator, address, size);

    let next_block_address = bytes_to_u64(&buff[8..]);
    Ok(next_block_address)
}

fn push_block(allocator: &mut RSSAllocator, address: u64, size: u64) {
    allocator.blocks.push(RSSBlock {
        address,
        size,
        is_used: true,
    });
}

fn bytes_to_u64(bytes: &[u8]) -> u64 {
    let bytes = bytes.try_into().unwrap();
    u64::from_be_bytes(bytes)
}
