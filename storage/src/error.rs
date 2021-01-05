#[derive(Debug)]
pub enum ErrorKind {
    AllocatorNotInitialized,
    BlockNotFound,
    VolumeAlreadyExists,
    VolumeCorrupted,
    VolumeInaccessible,
    VolumeInitFailed,
    VolumeInvalidExt,
    VolumeInvalidSize,
    VolumeNotEnoughSpace,
    VolumeNotFound,
}
