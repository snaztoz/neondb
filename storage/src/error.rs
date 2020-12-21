#[derive(Debug)]
pub enum ErrorKind {
    VolumeCorrupted,
    VolumeNotEnoughSpace,
    VolumeNotFound,
    VolumeInvalidExt,
    VolumeInvalidSize,
    VolumeInaccessible,
}
