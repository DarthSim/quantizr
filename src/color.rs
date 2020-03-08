use std::os::raw::c_uchar;

#[repr(C)]
#[derive(Clone,Copy)]
pub struct Color {
    pub r: c_uchar,
    pub g: c_uchar,
    pub b: c_uchar,
    pub a: c_uchar,
}

impl Default for Color {
    fn default() -> Self {
        Self{r:0, g:0, b:0, a:0}
    }
}

impl Color {
    pub fn new(r: c_uchar, g: c_uchar, b: c_uchar, a: c_uchar) -> Self {
        Self{r: r, g: g, b: b, a: a}
    }

    pub fn as_slice(&self) -> [c_uchar; 4] {
        [self.r, self.g, self.b, self.a]
    }
}
