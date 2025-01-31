use std::io;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O Error: {0}")]
    IoError(#[from] io::Error),

    #[error("error parsing VoxelTagName: {0}")]
    VoxelTagNameParseError(String),

    #[error("error parsing VoxelAtttibuteName: {0}")]
    VoxelAttributeNameParseError(String),

    #[error("error parsing VoxelData: {0}")]
    VoxelDataParseError(String),

    #[error("error parsing VoxelElement: {0}")]
    VoxelElementParseError(String),

    #[error("error parsing VXStyleName: {0}")]
    VXStyleNameParseError(String),
}
