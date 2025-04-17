/// RGBA color
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }
}

/// Color palette
#[repr(C)]
pub struct Palette {
    /// The number of colors in the palette
    pub count: u32,
    /// The palette colors
    pub entries: [Color; 256],
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            count: 0,
            entries: [Color::default(); 256],
        }
    }
}

impl From<&[[f32; 4]]> for Palette {
    fn from(colors: &[[f32; 4]]) -> Self {
        let mut palette = Self::default();
        palette.count = colors.len() as u32;

        for (i, c) in colors.iter().enumerate() {
            palette.entries[i].r = c[0].round().clamp(0.0, 255.0) as u8;
            palette.entries[i].g = c[1].round().clamp(0.0, 255.0) as u8;
            palette.entries[i].b = c[2].round().clamp(0.0, 255.0) as u8;
            palette.entries[i].a = c[3].round().clamp(0.0, 255.0) as u8;
        }

        palette
    }
}
