use std::io;
use thiserror::Error;

/// Error type for container operations
#[derive(Debug, Error)]
pub enum ContainerError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("{0}")]
    UnsupportedOperation(&'static str),

    #[error("Alignment error: {0}")]
    AlignmentError(String),
}

// /// Error type for container operations
// #[derive(Debug)]
// pub enum ContainerError {
//     Io(io::Error),
//     UnsupportedOperation(&'static str),
//     AlignmentError(String),
// }

// impl From<io::Error> for ContainerError {
//     fn from(err: io::Error) -> Self {
//         ContainerError::Io(err)
//     }
// }

// impl std::fmt::Display for ContainerError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             ContainerError::Io(err) => write!(f, "IO error: {}", err),
//             ContainerError::UnsupportedOperation(msg) => write!(f, "{}", msg),
//             ContainerError::AlignmentError(msg) => write!(f, "Alignment error: {}", msg),
//         }
//     }
// }

// impl std::error::Error for ContainerError {}
