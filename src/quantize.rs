use std::cmp::Ordering;

use crate::histogram::Histogram;
use crate::cluster::Cluster;
use crate::palette::Palette;
use crate::error::Error;
use crate::image::Image;
use crate::options::Options;
use crate::colormap::Colormap;

pub struct QuantizeResult {
    pub palette: Palette,
    pub dithering_level: f32,
    colormap: Colormap,
}

impl QuantizeResult {
    pub fn quantize(image: &Image, attr: &Options) -> Self {
        let mut hist = Histogram::new();
        hist.add_image(image);

        Self::quantize_histogram(&hist, attr)
    }

    pub fn quantize_histogram(hist: &Histogram, attr: &Options) -> Self {
        let colormap: Colormap;

        let max_colors_f64 = attr.max_colors as f64;
        let max_colors_usize = attr.max_colors as usize;

        if hist.0.len() <= max_colors_usize {
            colormap = Colormap::from_histogram(&hist);
        } else {
            let mut clusters = Vec::<Cluster>::with_capacity(attr.max_colors as usize);

            let root = Cluster::from_histogram(&hist);

            clusters.push(root);

            while clusters.len() < max_colors_usize {
                // We want to split bigger clusters in the beginning,
                // and clusters with bigger chan_diff in the end
                let weight_ratio = 0.75 - (clusters.len() as f64 + 1.0) / max_colors_f64 / 2.0;

                // Get the best cluster to split
                let to_split_opt = clusters.iter().enumerate()
                    .filter(|(_, c)| c.chan_diff > 0.0)
                    .map(|(i, c)|{
                        let priority = c.chan_diff * c.weight.powf(weight_ratio);
                        (i, priority)
                    })
                    .max_by(|&(_, a), &(_, b)| a.partial_cmp(&b).unwrap_or(Ordering::Equal))
                    .map(|(i, _)| clusters.swap_remove(i) );

                // If nothing there, this means everything is ready
                let mut to_split = match to_split_opt {
                    Some(c) => c,
                    None => break,
                };

                let (mut c1, mut c2) = to_split.split();

                if c1.entries.is_empty() {
                    c2.chan_diff = 0.0;
                    clusters.push(c2);
                    continue;
                }

                if c2.entries.is_empty() {
                    c1.chan_diff = 0.0;
                    clusters.push(c1);
                    continue;
                }

                clusters.push(c1);
                clusters.push(c2);
            }

            colormap = Colormap::from_clusters(&clusters);
        }

        let mut palette = Palette::default();
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

    fn remap_image_no_dither(&mut self, image: &Image, buf: &mut [u8]) {
        for point in 0..image.width*image.height {
            let data_point = point*4;

            let pix = &image.data[data_point..data_point+4];
            let r = pix[0] as f32;
            let g = pix[1] as f32;
            let b = pix[2] as f32;
            let a = pix[3] as f32;

            let (ind, _) = self.colormap.nearest_ind(&[r, g, b, a]);

            buf[point] = ind as u8;
        }
    }

    fn remap_image_dither(&mut self, image: &Image, buf: &mut [u8]) {
        let error_size = image.width+2;
        let mut error_curr = vec![[0f32; 4]; error_size];
        let mut error_next = vec![[0f32; 4]; error_size];

        let dithering_coeff = self.dithering_level * 15.0 / 16.0 / 16.0;
        let err_threshold = self.colormap.error;
        // println!("Err threshold {}", self.colormap.error);

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

                let err_pix = &mut error_curr[err_ind];

                let err_total = err_pix[0].powi(2) + err_pix[1].powi(2) + err_pix[2].powi(2) + err_pix[3].powi(2);
                if err_total > err_threshold {
                    err_pix[0] = err_pix[0] * 0.8;
                    err_pix[1] = err_pix[1] * 0.8;
                    err_pix[2] = err_pix[2] * 0.8;
                    err_pix[3] = err_pix[3] * 0.8;
                }

                let pix = &image.data[data_point..data_point+4];
                let dith_pix = [
                    pix[0] as f32 + err_pix[0],
                    pix[1] as f32 + err_pix[1],
                    pix[2] as f32 + err_pix[2],
                    pix[3] as f32 + err_pix[3],
                ];

                let (ind, _) = self.colormap.nearest_ind(&dith_pix);
                buf[point] = ind as u8;

                let pal_pix = self.colormap.color(ind);
                let mut err_r = dith_pix[0] - pal_pix[0];
                let mut err_g = dith_pix[1] - pal_pix[1];
                let mut err_b = dith_pix[2] - pal_pix[2];
                let mut err_a = dith_pix[3] - pal_pix[3];

                let err_total = err_r.powi(2) + err_g.powi(2) + err_b.powi(2) + err_a.powi(2);
                if err_total > err_threshold {
                    err_r = err_r * 0.75;
                    err_g = err_g * 0.75;
                    err_b = err_b * 0.75;
                    err_a = err_a * 0.75;
                }

                err_r = err_r * dithering_coeff;
                err_g = err_g * dithering_coeff;
                err_b = err_b * dithering_coeff;
                err_a = err_a * dithering_coeff;

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

            std::mem::swap(&mut error_curr, &mut error_next);
            error_next.fill_with(|| [0f32; 4]);
        }
    }
}
