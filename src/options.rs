use crate::error::Error;

/// Quantization options
pub struct Options {
    max_colors: i32,
}

impl Default for Options {
    fn default() -> Self {
        Self { max_colors: 256 }
    }
}

impl Options {
    pub fn get_max_colors(&self) -> i32 {
        self.max_colors
    }

    /// Sets the maximum number of colors in the resultant palette.
    ///
    /// Returns [`Error::ValueOutOfRange`] if the provided number is greater
    /// than 256 or less than 2
    pub fn set_max_colors(&mut self, colors: i32) -> Result<(), Error> {
        if colors > 256 || colors < 2 {
            return Err(Error::ValueOutOfRange);
        }

        self.max_colors = colors;

        Ok(())
    }
}
