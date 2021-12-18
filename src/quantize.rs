use std::collections::BinaryHeap;

use crate::histogram::Historgram;
use crate::cluster::Cluster;
use crate::color::Color;
use crate::error::Error;
use crate::image::Image;
use crate::options::Options;
use crate::colormap::Colormap;

#[repr(C)]
pub struct Palette {
    pub count: u32,
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

pub struct QuantizeResult {
    pub palette: Palette,
    pub dithering_level: f32,
    colormap: Colormap,
}

impl QuantizeResult {
    pub fn quantize(image: &Image, attr: &Options) -> Self {
        let mut hist = Historgram::new();
        hist.add_image(image);

        Self::quantize_histogram(&hist, attr)
    }

    pub fn quantize_histogram(hist: &Historgram, attr: &Options) -> Self {
        let mut heap = BinaryHeap::new();
        let mut clusters = Vec::<Cluster>::with_capacity(attr.max_colors as usize);

        let mut root = Cluster::from_histogram(&hist);
        root.calc_mean_and_weight();
        root.calc_widest_and_priority();

        // If priority is zero, then all colors in cluster are the same
        if root.priority > 0 {
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

            let (mut c1, mut c2) = to_split.split();

            c1.calc_mean_and_weight();
            c2.calc_mean_and_weight();

            if c1.entries.is_empty() {
                clusters.push(c2);
                continue;
            }

            if c2.entries.is_empty() {
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

            c1.calc_widest_and_priority();
            c2.calc_widest_and_priority();

            if c1.priority > 0 {
                heap.push(c1);
            } else {
                clusters.push(c1)
            }

            if c2.priority > 0 {
                heap.push(c2);
            } else {
                clusters.push(c2)
            }
        }

        let mut palette = Palette::default();
        let colormap = Colormap::new(&clusters);
        colormap.generate_palette(&mut palette);

        Self{
            palette: palette,
            colormap: colormap,
            dithering_level: 1.0,
        }
    }

    pub fn set_dithering_level(&mut self, level: f32) -> Error {
        if level > 1.0 || level < 0.0 {
            return Error::ValueOutOfRange
        }

        self.dithering_level = level;

        Error::Ok
    }

    pub fn remap_image(&mut self, image: &Image, buf: &mut [u8]) -> Error {
        if buf.len() < image.width * image.height {
            return Error::BufferTooSmall
        }

        if self.dithering_level > 0.0 {
            self.remap_image_dither(image, buf);
        } else {
            self.remap_image_no_dither(image, buf);
        }

        self.colormap.generate_palette(&mut self.palette);

        Error::Ok
    }

    fn remap_image_no_dither(&self, image: &Image, buf: &mut [u8]) {
        let mut last_ind = 0;

        for point in 0..image.width*image.height {
            let data_point = point*4;

            let pix = &image.data[data_point..data_point+4];
            let r = pix[0] as f32;
            let g = pix[1] as f32;
            let b = pix[2] as f32;
            let a = pix[3] as f32;

            last_ind = self.colormap.nearest_ind(&[r, g, b, a], last_ind);

            buf[point] = last_ind as u8;
        }
    }

    fn remap_image_dither(&self, image: &Image, buf: &mut [u8]) {
        let mut last_ind = 0;

        let error_size = image.width+2;
        let mut error_curr = vec![[0f32; 4]; error_size].into_boxed_slice();
        let mut error_next = vec![[0f32; 4]; error_size].into_boxed_slice();

        let mut x_reverse = true;

        for y in 0..image.height {
            x_reverse = !x_reverse;

            let mut x = match x_reverse {
                false => 0,
                true => image.width - 1,
            };

            loop {
                let point = image.width*y + x;
                let data_point = point * 4;

                let err_ind = x + 1;
                let err_inds = match x_reverse{
                    false => [err_ind - 1, err_ind, err_ind + 1],
                    true => [err_ind + 1, err_ind, err_ind - 1],
                };

                let pix = &image.data[data_point..data_point+4];
                let r = pix[0] as f32;
                let g = pix[1] as f32;
                let b = pix[2] as f32;
                let a = pix[3] as f32;

                let err_pix = &error_curr[err_ind];
                let dr = (r + err_pix[0]).clamp(0.0, 255.0);
                let dg = (g + err_pix[1]).clamp(0.0, 255.0);
                let db = (b + err_pix[2]).clamp(0.0, 255.0);
                let da = (a + err_pix[3]).clamp(0.0, 255.0);

                last_ind = self.colormap.nearest_ind(&[dr, dg, db, da], last_ind);
                buf[point] = last_ind as u8;

                let pal_pix = self.colormap.color(last_ind);
                let err_r = (r - pal_pix[0]) / 16.0 * self.dithering_level;
                let err_g = (g - pal_pix[1]) / 16.0 * self.dithering_level;
                let err_b = (b - pal_pix[2]) / 16.0 * self.dithering_level;
                let err_a = (a - pal_pix[3]) / 16.0 * self.dithering_level;

                let err = &mut error_next[err_inds[0]];
                err[0] += err_r * 3.0;
                err[1] += err_g * 3.0;
                err[2] += err_b * 3.0;
                err[3] += err_a * 3.0;

                let err = &mut error_next[err_inds[1]];
                err[0] += err_r * 5.0;
                err[1] += err_g * 5.0;
                err[2] += err_b * 5.0;
                err[3] += err_a * 5.0;

                let err = &mut error_next[err_inds[2]];
                err[0] += err_r * 1.0;
                err[1] += err_g * 1.0;
                err[2] += err_b * 1.0;
                err[3] += err_a * 1.0;

                let err = &mut error_curr[err_inds[2]];
                err[0] += err_r * 7.0;
                err[1] += err_g * 7.0;
                err[2] += err_b * 7.0;
                err[3] += err_a * 7.0;

                if x_reverse {
                    if x <= 0 {
                        break
                    }
                    x -= 1;
                } else {
                    x += 1;
                    if x >= image.width {
                        break
                    }
                }
            }

            let tmp_err = error_curr;
            error_curr = error_next;
            error_next = tmp_err;

            for i in 0..error_size {
                error_next[i] = [0.0, 0.0, 0.0, 0.0];
            }
        }
    }
}
