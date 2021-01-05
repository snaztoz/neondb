#[derive(Debug)]
pub enum ErrorKind {
    AllocatorNotInitialized,
    BlockNotFound,
    VolumeCorrupted,
    VolumeNotEnoughSpace,
    VolumeNotFound,
    VolumeInvalidExt,
    VolumeInvalidSize,
    VolumeInaccessible,
}
