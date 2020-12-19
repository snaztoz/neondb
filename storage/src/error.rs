#[derive(Debug)]
pub enum ErrorKind {
	VolumeCorrupted,
    VolumeNotFound,
    VolumeInvalidExt,
    VolumeInvalidSize,
    VolumeInaccessible,
}
