use crate::error::Error;

#[repr(C)]
pub struct Options {
    pub max_colors: i32,
    pub add_fixed_colors: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self{
            max_colors: 256,
            add_fixed_colors: true,
        }
    }
}

impl Options {
    pub fn set_max_colors(&mut self, colors: i32) -> Error {
        if colors > 256 || colors < 2 {
            return Error::ValueOutOfRange
        }

        self.max_colors = colors;
        self.add_fixed_colors = colors > 128;

        Error::Ok
    }
}
