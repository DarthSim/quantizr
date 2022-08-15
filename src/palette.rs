/// RGBA color
#[cfg_attr(feature="capi", repr(C))]
#[derive(Clone,Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for Color {
    fn default() -> Self {
        Self{r:0, g:0, b:0, a:0}
    }
}

/// Color palette
#[cfg_attr(feature="capi", repr(C))]
pub struct Palette {
    /// The number of colors in the palette
    pub count: u32,
    /// The palette colors
    pub entries: [Color; 256],
}

impl Default for Palette {
    fn default() -> Self {
        Self{
            count: 0,
            entries: [Color::default(); 256],
        }
    }
}
