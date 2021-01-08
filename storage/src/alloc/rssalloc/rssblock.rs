#[derive(Debug)]
pub struct RSSBlock {
    pub address: u64,
    pub size: u64,
    pub is_used: bool,
}

impl RSSBlock {
    // BLOCK_META ada di bagian awal dari tiap blok, yang berguna
    // untuk menyimpan data mengenai blok tersebut.
    //
    // 8 byte untuk panjang blok, dan
    // 8 byte sisanya untuk alamat blok selanjutnya
    pub const META_SIZE: u64 = 16;

    pub fn construct_meta(&self, next_block_address: u64) -> Vec<u8> {
        self.size
            .to_be_bytes()
            .iter()
            .chain(&next_block_address.to_be_bytes())
            .map(|b| *b)
            .collect::<Vec<u8>>()
    }
}
