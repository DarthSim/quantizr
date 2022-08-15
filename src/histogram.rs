use std::collections::HashMap;
use std::hash::{BuildHasher,Hasher};

use crate::image::Image;

macro_rules! hist_key {
    ($c: expr) => {
        (($c[0] as u64) << 24) +
        (($c[1] as u64) << 16) +
        (($c[2] as u64) << 8) +
        ($c[3] as u64)
    };
}

#[derive(Clone,Copy)]
pub(crate) struct HistogramEntry {
    pub color: [u8; 4],
    pub weight: u32,
}

/// Color histogram
pub struct Histogram{
    pub(crate) map: HashMap<u64, HistogramEntry, ColorHasher>,
}

impl Histogram {
    /// Creates new empty [`Histogram`]
    pub fn new() -> Self {
        Self{map: HashMap::with_hasher(ColorHasher(0))}
    }

    /// Adds colors from [`Image`] to the histogram
    pub fn add_image(&mut self, image: &Image) {
        for ind in (0..image.width*image.height*4).step_by(4) {
            let pix = &image.data[ind..ind+4];
            let color = if pix[3] != 0 {
                [pix[0], pix[1], pix[2], pix[3]]
            } else {
                [0, 0, 0, 0]
            };

            let key = hist_key!(color);

            self.map.entry(key)
                .and_modify(|e| e.weight += 1)
                .or_insert(HistogramEntry{
                    color: color,
                    weight: 1,
                });
        }
    }
}

pub(crate )struct ColorHasher(u64);
impl BuildHasher for ColorHasher {
    type Hasher = Self;
    #[inline(always)]
    fn build_hasher(&self) -> Self {
        Self(0)
    }
}

impl Hasher for ColorHasher {
    // Magick number from https://github.com/cbreeden/fxhash/blob/master/lib.rs
    #[inline(always)]
    fn finish(&self) -> u64 { self.0.wrapping_mul(0x517cc1b727220a95) }

    #[inline(always)]
    fn write_u64(&mut self, i: u64) { self.0 = i; }

    fn write(&mut self, _bytes: &[u8]) { unimplemented!() }
    fn write_u8(&mut self, _i: u8) { unimplemented!() }
    fn write_u16(&mut self, _i: u16) { unimplemented!() }
    fn write_u32(&mut self, _i: u32) { unimplemented!() }
    fn write_u128(&mut self, _i: u128) { unimplemented!() }
    fn write_usize(&mut self, _i: usize) { unimplemented!() }
    fn write_i8(&mut self, _i: i8) { unimplemented!() }
    fn write_i16(&mut self, _i: i16) { unimplemented!() }
    fn write_i32(&mut self, _i: i32) { unimplemented!() }
    fn write_i64(&mut self, _i: i64) { unimplemented!() }
    fn write_i128(&mut self, _i: i128) { unimplemented!() }
    fn write_isize(&mut self, _i: isize) { unimplemented!() }
}
