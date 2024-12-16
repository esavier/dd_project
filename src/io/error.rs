use std::fmt::Display;



impl std::error::Error for IoError {}
impl std::error::Error for DdError {}
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum IoError {
    InputFileDoesNotExist(String),
    InputFileNoReadPermission(String),
    InputFileOpenError(String),
    FileMetadataAcquireError(String),
    ChannelEror(String),
}

impl Display for IoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IoError::InputFileDoesNotExist(e) => write!(f, "Input file does not exist: {}", e),
            IoError::InputFileNoReadPermission(e) => write!(f, "Input file is read-only: {}", e),
            IoError::InputFileOpenError(e) => write!(f, "Input file open error: {}", e),
            IoError::FileMetadataAcquireError(e) => write!(f, "File metadata acquire error: {}", e),
            IoError::ChannelEror(e) => write!(f, "Channel error: {}", e),
        }
    }
}



#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum DdError {
    IoError(IoError),
    OtherError(String),
}

impl Display for DdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DdError::IoError(e) => write!(f, "IO error: {}", e),
            DdError::OtherError(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl From<IoError> for DdError {
    fn from(e: IoError) -> Self {
        DdError::IoError(e)
    }
}
