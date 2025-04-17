use crate::error::Error;

/// Image reference containing pixel data and dimensions info
pub struct Image<'data> {
    pub width: usize,
    pub height: usize,
    pub data: &'data [u8],
}

impl<'data> Image<'data> {
    /// Creates an [`Image`] from a slice of RGBA pixels.
    ///
    /// Returns [`Error::BufferTooSmall`] if the provided slice length is less
    /// than `width * height * 4`
    pub fn new(data: &'data [u8], width: usize, height: usize) -> Result<Self, Error> {
        if data.len() < width * height * 4 {
            return Err(Error::BufferTooSmall);
        }

        Ok(Self {
            data: data,
            width: width,
            height: height,
        })
    }
}
