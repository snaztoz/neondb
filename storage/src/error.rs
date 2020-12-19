#[derive(Debug)]
pub enum ErrorKind {
    VolumeNotFound,
    VolumeInvalidExt,
    VolumeInvalidSize,
    VolumeInaccessible,
}
