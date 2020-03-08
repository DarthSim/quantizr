use crate::error::Error;

#[repr(C)]
pub struct Options {
    pub max_colors: i32,
}

impl Default for Options {
    fn default() -> Self {
        Self{
            max_colors: 256,
        }
    }
}

impl Options {
    pub fn set_max_colors(&mut self, colors: i32) -> Error {
        if colors > 256 || colors < 2 {
            return Error::ValueOutOfRange
        }

        self.max_colors = colors;

        Error::Ok
    }
}
