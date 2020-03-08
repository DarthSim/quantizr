use std::collections::BinaryHeap;
use std::ops::{Deref,DerefMut};
use std::os::raw::c_uchar;

use crate::options::Options;
use crate::image::{CData,Image};
use crate::cluster::Cluster;
use crate::error::Error;

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

#[repr(C)]
#[derive(Clone)]
pub struct Palette {
    count: u32,
    entries: [Color; 256],
}

impl Default for Palette {
    fn default() -> Self {
        Self{
            count: 0,
            entries: [Color::default(); 256],
        }
    }
}

#[repr(C)]
pub struct QuantizeResult {
    palette: *mut Palette,
    pub dithering_level: f32,
}

impl Drop for QuantizeResult {
    fn drop(&mut self) {
        unsafe { std::mem::drop(Box::from_raw(self.palette)) };
    }
}

impl QuantizeResult {
    pub fn quantize(image: &Image, attr: &Options) -> Self {
        let mut res = Self{
            palette: Box::into_raw(Box::new(Palette::default())),
            dithering_level: 1.0,
        };

        let mut heap = BinaryHeap::new();
        let mut clusters = Vec::<Cluster>::with_capacity(attr.max_colors as usize);

        let mut root = Cluster::populate(image.width*image.height);
        root.calc_mean_and_range(image);

        if root.chan_range > 0 {
            heap.push(root);
        } else {
            clusters.push(root);
        }

        loop {
            // Try to pop cluster from queue
            // If nothing there, this means everything is ready
            let mut to_split = match heap.pop() {
                Some(c) => c,
                None => break,
            };

            let (mut c1, mut c2) = to_split.split(image);

            c1.calc_mean_and_range(image);
            c2.calc_mean_and_range(image);

            if c1.indexes.is_empty() {
                clusters.push(c2);
                continue;
            }

            if c2.indexes.is_empty() {
                clusters.push(c1);
                continue;
            }

            let colors = clusters.len() + heap.len() + 2;

            // Looks like we reached the maximum of colors
            // Add new clusters to ready and flush queue
            if attr.max_colors == colors as i32 {
                clusters.push(c1);
                clusters.push(c2);
                clusters.append(&mut heap.into_vec());
                break
            }

            if c1.chan_range > 0 {
                heap.push(c1);
            } else {
                clusters.push(c1)
            }

            if c2.chan_range > 0 {
                heap.push(c2);
            } else {
                clusters.push(c2)
            }
        }

        res.generate_palette(&clusters);

        res
    }

    fn generate_palette(&mut self, clusters: &Vec<Cluster>) {
        let mut palette = unsafe { &mut *self.palette };

        palette.count = clusters.len() as u32;

        for (i, cl) in clusters.iter().enumerate() {
            palette.entries[i] = cl.mean
        }
    }

    pub fn get_palette_ptr(&self) -> *mut Palette {
        self.palette
    }

    pub fn set_dithering_level(&mut self, level: f32) -> Error {
        if level > 1.0 || level < 0.0 {
            return Error::ValueOutOfRange
        }

        self.dithering_level = level;

        Error::Ok
    }

    pub fn remap_image(&self, image: &Image, buffer: &mut CData) -> Error {
        let buf = buffer.deref_mut();

        if buf.len() < image.width * image.height {
            return Error::BufferTooSmall
        }

        let palette = unsafe { &*self.palette };

        let mut palette_i32: [i32; 1024] = [0; 1024];
        for i in 0..(palette.count as usize) {
            palette_i32[i*4 + 0] = palette.entries[i].r as i32;
            palette_i32[i*4 + 1] = palette.entries[i].g as i32;
            palette_i32[i*4 + 2] = palette.entries[i].b as i32;
            palette_i32[i*4 + 3] = palette.entries[i].a as i32;
        }

        if self.dithering_level > 0.0 {
            self.remap_image_dither(image, buf, &palette_i32, palette.count as usize)
        } else {
            self.remap_image_no_dither(image, buf, &palette_i32, palette.count as usize)
        }
    }

    pub fn remap_image_dither(&self, image: &Image, buf: &mut [c_uchar], palette: &[i32], colors_count: usize) -> Error {
        let image_data = image.data.deref();

        let error_size = (image.width+2)*4;

        let mut error_curr = Vec::<f32>::new();
        error_curr.resize(error_size, 0.0);

        let mut error_next = Vec::<f32>::new();
        error_next.resize(error_size, 0.0);

        for y in 0..image.height {
            for x in 0..image.width {
                let point = image.width*y + x;
                let err_ind = (x + 1)*4;

                let r = image_data[point*4 + 0] as i32;
                let g = image_data[point*4 + 1] as i32;
                let b = image_data[point*4 + 2] as i32;
                let a = image_data[point*4 + 3] as i32;

                let dr = clamp(r + error_curr[err_ind + 0] as i32);
                let dg = clamp(g + error_curr[err_ind + 1] as i32);
                let db = clamp(b + error_curr[err_ind + 2] as i32);
                let da = clamp(a + error_curr[err_ind + 3] as i32);

                let mut best_diff = std::u32::MAX;
                let mut best_ind = 0;

                for i in 0..colors_count {
                    let pr = palette[i*4 + 0];
                    let pg = palette[i*4 + 1];
                    let pb = palette[i*4 + 2];
                    let pa = palette[i*4 + 3];

                    let diff = sq_diff(dr, pr) + sq_diff(dg, pg) + sq_diff(db, pb) + sq_diff(da, pa);
                    if diff < best_diff {
                        best_diff = diff;
                        best_ind = i;
                    }
                }

                buf[point] = best_ind as u8;

                let err_r = (r - palette[best_ind*4 + 0]) as f32 / 16.0 * self.dithering_level;
				let err_g = (g - palette[best_ind*4 + 1]) as f32 / 16.0 * self.dithering_level;
				let err_b = (b - palette[best_ind*4 + 2]) as f32 / 16.0 * self.dithering_level;
                let err_a = (a - palette[best_ind*4 + 3]) as f32 / 16.0 * self.dithering_level;

                error_next[err_ind - 4 + 0] += err_r * 3.0;
                error_next[err_ind - 4 + 1] += err_g * 3.0;
                error_next[err_ind - 4 + 2] += err_b * 3.0;
                error_next[err_ind - 4 + 3] += err_a * 3.0;
                error_next[err_ind + 0 + 0] += err_r * 5.0;
                error_next[err_ind + 0 + 1] += err_g * 5.0;
                error_next[err_ind + 0 + 2] += err_b * 5.0;
                error_next[err_ind + 0 + 3] += err_a * 5.0;
                error_next[err_ind + 4 + 0] += err_r * 1.0;
                error_next[err_ind + 4 + 1] += err_g * 1.0;
                error_next[err_ind + 4 + 2] += err_b * 1.0;
                error_next[err_ind + 4 + 3] += err_a * 1.0;
                error_curr[err_ind + 4 + 0] += err_r * 7.0;
                error_curr[err_ind + 4 + 1] += err_g * 7.0;
                error_curr[err_ind + 4 + 2] += err_b * 7.0;
                error_curr[err_ind + 4 + 3] += err_a * 7.0;
            }

            let tmp_err = error_curr;
            error_curr = error_next;
            error_next = tmp_err;

			for i in 0..error_size {
				error_next[i] = 0.0;
			}
        }

        Error::Ok
    }

    pub fn remap_image_no_dither(&self, image: &Image, buf: &mut [c_uchar], palette: &[i32], colors_count: usize) -> Error {
        let image_data = image.data.deref();

        for y in 0..image.height {
            for x in 0..image.width {
                let point = image.width*y + x;

                let r = image_data[point*4 + 0] as i32;
                let g = image_data[point*4 + 1] as i32;
                let b = image_data[point*4 + 2] as i32;
                let a = image_data[point*4 + 3] as i32;

                let mut best_diff = std::u32::MAX;
                let mut best_ind = 0;

                for i in 0..colors_count {
                    let pr = palette[i*4 + 0];
                    let pg = palette[i*4 + 1];
                    let pb = palette[i*4 + 2];
                    let pa = palette[i*4 + 3];

                    let diff = sq_diff(r, pr) + sq_diff(g, pg) + sq_diff(b, pb) + sq_diff(a, pa);
                    if diff < best_diff {
                        best_diff = diff;
                        best_ind = i;
                    }
                }

                buf[point] = best_ind as u8;
            }
        }

        Error::Ok
    }
}

#[inline]
fn clamp(a: i32) -> i32 {
    if a < 0 {
        return 0
    }
    if a > 255 {
        return 255
    }
    a
}

#[inline]
fn sq_diff(a: i32, b: i32) -> u32 {
    let diff = a - b;
    (diff * diff) as u32
}
