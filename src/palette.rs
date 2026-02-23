#[cfg(not(feature = "std"))]
use num_traits::Float;

/// RGBA color
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// Color palette
#[repr(C)]
pub struct Palette {
    /// The number of colors in the palette
    pub count: u32,
    /// The palette colors
    pub entries: [Color; 256],
}

impl From<&[[f32; 4]]> for Palette {
    fn from(colors: &[[f32; 4]]) -> Self {
        assert!(colors.len() <= 256, "Palette can only have 256 colors");

        let mut entries = [Color::default(); 256];

        for (i, c) in colors.iter().enumerate() {
            entries[i].r = c[0].round().clamp(0.0, 255.0) as u8;
            entries[i].g = c[1].round().clamp(0.0, 255.0) as u8;
            entries[i].b = c[2].round().clamp(0.0, 255.0) as u8;
            entries[i].a = c[3].round().clamp(0.0, 255.0) as u8;
        }

        Self {
            count: colors.len() as u32,
            entries,
        }
    }
}
