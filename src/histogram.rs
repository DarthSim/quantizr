use core::hash::{BuildHasher, Hasher};

#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(not(feature = "std"))]
use hashbrown::HashMap;

use crate::image::Image;

pub(crate) struct HistogramEntry {
    pub color: [u8; 4],
    pub weight: u32,
}

/// Color histogram
pub struct Histogram {
    pub(crate) map: HashMap<u64, HistogramEntry, ColorHasher>,
}

impl Histogram {
    /// Creates new empty [`Histogram`]
    pub fn new() -> Self {
        Self {
            map: HashMap::with_hasher(ColorHasher(0)),
        }
    }

    /// Adds colors from [`Image`] to the histogram
    pub fn add_image(&mut self, image: &Image) {
        let size = image.width * image.height;

        let to_reserve = if self.map.is_empty() {
            size / 7
        } else {
            size / 21
        }
        .min(512 * 512);
        self.map.reserve(to_reserve);

        for ind in (0..size * 4).step_by(4) {
            let pix = &image.data[ind..ind + 4];

            let mut color: [u8; 4] = [0; 4];
            if pix[3] != 0 {
                color.copy_from_slice(pix);
            }

            let key = u32::from_le_bytes(color) as u64;

            self.map
                .entry(key)
                .and_modify(|e| e.weight = e.weight.saturating_add(1))
                .or_insert(HistogramEntry { color, weight: 1 });
        }
    }
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) struct ColorHasher(u64);
impl BuildHasher for ColorHasher {
    type Hasher = Self;
    #[inline(always)]
    fn build_hasher(&self) -> Self {
        Self(0)
    }
}

impl Hasher for ColorHasher {
    // Magick numbers from https://github.com/rust-lang/rustc-hash/blob/master/src/lib.rs
    #[inline(always)]
    fn finish(&self) -> u64 {
        self.0.wrapping_mul(0xf1357aea2e62a9c5).rotate_left(26)
    }

    #[inline(always)]
    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }

    fn write(&mut self, _bytes: &[u8]) {
        unimplemented!()
    }
}
