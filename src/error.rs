use core::fmt;

#[non_exhaustive]
pub enum Error {
    /// The value provided to the fuction is out of allowed range
    ValueOutOfRange,
    /// The slice provided to the function is too small
    BufferTooSmall,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValueOutOfRange => write!(f, "Value out of range"),
            Self::BufferTooSmall => write!(f, "Buffer is too small"),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl core::error::Error for Error {}
