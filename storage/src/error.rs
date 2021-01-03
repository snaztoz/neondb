#[derive(Debug)]
pub enum ErrorKind {
    BlockNotFound,
    VolumeCorrupted,
    VolumeNotEnoughSpace,
    VolumeNotFound,
    VolumeInvalidExt,
    VolumeInvalidSize,
    VolumeInaccessible,
}
