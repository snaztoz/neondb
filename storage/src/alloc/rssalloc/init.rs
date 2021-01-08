use super::*;

pub fn obtain_head(vol: &mut File, allocator: &mut RSSAllocator) -> Result<u64> {
    let address = NEONDB_FILE_ALLOCATABLE_START;
    let mut buff = [0u8; 16];

    Ops::read(address, &mut buff, vol);

    let (size, next_address) = extract_values(&buff[..]);

    if size != RSSBlock::BLOCK_META_SIZE {
        return Err(ErrorKind::VolumeCorrupted);
    }

    push_block(address, size, allocator);
    Ok(next_address)
}

pub fn new_volume(vol: &mut File, allocator: &mut RSSAllocator) -> Result<()> {
    debug_assert!(allocator.blocks.len() == 0);

    // Menambahkan head
    push_block(
        NEONDB_FILE_ALLOCATABLE_START,
        RSSBlock::BLOCK_META_SIZE,
        allocator,
    );
    allocator.mark_block(0, vol);

    push_remaining_space(allocator)?;

    Ok(())
}

pub fn scan_blocks(vol: &mut File, start_address: u64, allocator: &mut RSSAllocator) -> Result<()> {
    let mut address = start_address;
    let mut buff = [0u8; 16];

    while address != NULL_ADDRESS {
        Ops::read(address, &mut buff, vol);

        let (size, next_address) = extract_values(&buff[..]);

        if gap_exist_before(address, allocator)? {
            push_unused_block_before(address, allocator);
        }
        push_block(address, size, allocator);

        address = next_address;
    }

    push_remaining_space(allocator)?;

    Ok(())
}

fn push_block(address: u64, size: u64, allocator: &mut RSSAllocator) {
    allocator.blocks.push(RSSBlock {
        address,
        size,
        is_used: true,
    });
}

fn push_unused_block_before(next_block_address: u64, allocator: &mut RSSAllocator) {
    debug_assert!(allocator.blocks.last().unwrap().is_used);

    let address = next_push_address(allocator);
    allocator.blocks.push(RSSBlock {
        address,
        size: next_block_address - address,
        is_used: false,
    });
}

fn push_remaining_space(allocator: &mut RSSAllocator) -> Result<()> {
    // alamat byte terakhir yang dapat ditempati oleh data
    let last_address = NEONDB_FILE_ALLOCATABLE_SIZE;

    if gap_exist_before(last_address, allocator)? {
        push_unused_block_before(last_address, allocator);
    }

    Ok(())
}

fn gap_exist_before(next_block_address: u64, allocator: &RSSAllocator) -> Result<bool> {
    debug_assert!(allocator.blocks.last().unwrap().is_used);

    let address = next_push_address(allocator);
    if address > next_block_address {
        return Err(ErrorKind::VolumeCorrupted);
    }

    Ok(address < next_block_address)
}

// Address awal dari blok yang akan dipush berikutnya
fn next_push_address(allocator: &RSSAllocator) -> u64 {
    let address = allocator.blocks.last().unwrap().address;
    let size = allocator.blocks.last().unwrap().size;

    address + size
}

fn extract_values(bytes: &[u8]) -> (u64, u64) {
    (bytes_to_u64(&bytes[..8]), bytes_to_u64(&bytes[8..]))
}

fn bytes_to_u64(bytes: &[u8]) -> u64 {
    let bytes = bytes.try_into().unwrap();
    u64::from_be_bytes(bytes)
}
