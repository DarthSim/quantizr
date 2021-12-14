use std::collections::BinaryHeap;

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
}

impl QuantizeResult {
    pub fn quantize(image: &Image, attr: &Options) -> Self {
        let mut res = Self{
            palette: Palette::default(),
            dithering_level: 1.0,
        };

        let mut heap = BinaryHeap::new();
        let mut clusters = Vec::<Cluster>::with_capacity(attr.max_colors as usize);

        let mut root = Cluster::populate(image);
        root.calc_mean_and_priority();

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

            c1.calc_mean_and_priority();
            c2.calc_mean_and_priority();

            if c1.colors.is_empty() {
                clusters.push(c2);
                continue;
            }

            if c2.colors.is_empty() {
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

        clusters.sort_by_cached_key(|cl| {
            std::usize::MAX - cl.colors.len()
        });

        res.generate_palette(&clusters);

        res
    }

    fn generate_palette(&mut self, clusters: &Vec<Cluster>) {
        self.palette.count = clusters.len() as u32;

        for (i, cl) in clusters.iter().enumerate() {
            self.palette.entries[i] = cl.mean
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

        let cm = Colormap::new(&self.palette);

        self.remap_image_no_dither(image, buf, &cm);
        self.fix_palette(image, buf);

        if self.dithering_level > 0.0 {
            let cm = Colormap::new(&self.palette);
            self.dither_image(image, buf, &cm);
        }

        Error::Ok
    }

    fn remap_image_no_dither(&self, image: &Image, buf: &mut [u8], cm: &Colormap) {
        let mut last_ind = 0;

        for point in 0..image.width*image.height {
            let data_point = point*4;

            let pix = &image.data[data_point..data_point+4];
            let r = pix[0] as f32;
            let g = pix[1] as f32;
            let b = pix[2] as f32;
            let a = pix[3] as f32;

            last_ind = cm.nearest_ind(&[r, g, b, a], last_ind);

            buf[point] = last_ind as u8;
        }
    }

    fn fix_palette(&mut self, image: &Image, buf: &[u8]) {
        let mut colors = [[0usize; 4]; 256];
        let mut counts = [0usize; 256];

        for point in 0..image.width*image.height {
            let data_point = point*4;

            let pix = &image.data[data_point..data_point+4];
            let r = pix[0] as usize;
            let g = pix[1] as usize;
            let b = pix[2] as usize;
            let a = pix[3] as usize;

            let ind = buf[point] as usize;

            let color = &mut colors[ind];
            color[0] += r;
            color[1] += g;
            color[2] += b;
            color[3] += a;
            counts[ind] += 1;
        }

        for (i, c) in colors.iter().enumerate() {
            let count = counts[i];

            if count > 0 {
                let pal_c = &mut self.palette.entries[i];
                pal_c.r = (c[0] / count) as u8;
                pal_c.g = (c[1] / count) as u8;
                pal_c.b = (c[2] / count) as u8;
                pal_c.a = (c[3] / count) as u8;
            }
        }
    }

    fn dither_image(&self, image: &Image, buf: &mut [u8], cm: &Colormap) {
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

                let mut pal_ind = buf[point] as usize;
                pal_ind = cm.nearest_ind(&[dr, dg, db, da], pal_ind);
                buf[point] = pal_ind as u8;

                let pal_pix = cm.color(pal_ind);
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
