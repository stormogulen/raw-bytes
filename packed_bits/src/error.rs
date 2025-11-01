use thiserror::Error;

#[derive(Debug, Error)]
pub enum PackedBitsError {
    #[error("Bit width N must be in the range 1..=32, got {0}")]
    InvalidBitWidth(usize),

    #[error("Value {0} does not fit in {1} bits")]
    ValueOverflow(u32, usize),

    #[error("Index {0} is out of bounds for length {1}")]
    IndexOutOfBounds(usize, usize),

    #[error("Insufficient bytes for {0} elements")]
    InsufficientBytes(usize),

    #[error("Unexpected error")]
    Unexpected,
}
