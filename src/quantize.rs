use alloc::vec;
use core::mem;

use crate::cluster::Cluster;
use crate::colormap::Colormap;
use crate::error::Error;
use crate::histogram::Histogram;
use crate::image::Image;
use crate::options::Options;
use crate::palette::Palette;

const EMPTY_PIX: [u8; 4] = [0; 4];

// Result of quantization
pub struct QuantizeResult {
    error: f32,
    dithering_level: f32,
    colormap: Colormap,
}

impl QuantizeResult {
    /// Quantizes the provided [`Image`]
    pub fn quantize(image: &Image, attr: &Options) -> Self {
        let mut hist = Histogram::new();
        hist.add_image(image);

        Self::quantize_histogram(&hist, attr)
    }

    /// Quantizes the provided [`Histogram`]
    pub fn quantize_histogram(hist: &Histogram, attr: &Options) -> Self {
        let max_colors = attr.get_max_colors() as usize;

        let colormap = if hist.map.len() <= max_colors {
            Colormap::from_histogram(hist)
        } else {
            let root = Cluster::from_histogram(hist);
            let clusters = root.split_into(max_colors);

            Colormap::from_clusters(&clusters)
        };

        Self {
            error: colormap.error,
            colormap,
            dithering_level: 1.0,
        }
    }

    /// Sets the dithering level.
    ///
    /// Returns [`Error::ValueOutOfRange`] if the provided value is greater
    /// than 1.0 or lesser than 0.0
    pub fn set_dithering_level(&mut self, level: f32) -> Result<(), Error> {
        if !(0.0..=1.0).contains(&level) {
            return Err(Error::ValueOutOfRange);
        }

        self.dithering_level = level;

        Ok(())
    }

    /// Returns quantization error. The lesser the error the better the image
    /// was quantized
    pub fn get_error(&self) -> f32 {
        self.error
    }

    /// Returns the [`Palette`] generated after quantization
    pub fn get_palette(&self) -> &Palette {
        self.colormap.get_palette()
    }

    /// Remaps the proxided [`Image`] to a slize of bytes.
    ///
    /// Returns [`Error::BufferTooSmall`] if the provided buffer is smaller
    /// than `image.width * image.height`
    pub fn remap_image(&self, image: &Image, buf: &mut [u8]) -> Result<(), Error> {
        if buf.len() < image.width * image.height {
            return Err(Error::BufferTooSmall);
        }

        if self.dithering_level > 0.0 {
            self.remap_image_dither(image, buf);
        } else {
            self.remap_image_no_dither(image, buf);
        }

        Ok(())
    }

    fn remap_image_no_dither(&self, image: &Image, buf: &mut [u8]) {
        #[allow(clippy::needless_range_loop)]
        for point in 0..image.width * image.height {
            let data_point = point * 4;

            let pix = pix_or_empty(&image.data[data_point..data_point + 4]);
            let r = pix[0] as f32;
            let g = pix[1] as f32;
            let b = pix[2] as f32;
            let a = pix[3] as f32;

            let (ind, _, _) = self.colormap.nearest_ind(&[r, g, b, a]);

            buf[point] = ind;
        }
    }

    fn remap_image_dither(&self, image: &Image, buf: &mut [u8]) {
        let error_size = image.width + 2;
        let mut error_curr = vec![[0f32; 4]; error_size];
        let mut error_next = vec![[0f32; 4]; error_size];

        let dithering_coeff = self.dithering_level * 15.0 / 16.0 / 16.0;
        let err_threshold = self.error;
        // println!("Err threshold {}", self.error);

        let mut x_reverse = true;

        for y in 0..image.height {
            x_reverse = !x_reverse;

            for xx in 0..image.width {
                let x = if x_reverse { image.width - 1 - xx } else { xx };

                let point = image.width * y + x;
                let data_point = point * 4;

                let err_ind = x + 1;
                let err_inds = if x_reverse {
                    (err_ind + 1, err_ind, err_ind - 1)
                } else {
                    (err_ind - 1, err_ind, err_ind + 1)
                };

                let err_pix = &mut error_curr[err_ind];

                let err_total = err_pix[0] * err_pix[0]
                    + err_pix[1] * err_pix[1]
                    + err_pix[2] * err_pix[2]
                    + err_pix[3] * err_pix[3];

                if err_total > err_threshold {
                    err_pix[0] *= 0.8;
                    err_pix[1] *= 0.8;
                    err_pix[2] *= 0.8;
                    err_pix[3] *= 0.8;
                }

                let pix = pix_or_empty(&image.data[data_point..data_point + 4]);
                let dith_pix = [
                    pix[0] as f32 + err_pix[0],
                    pix[1] as f32 + err_pix[1],
                    pix[2] as f32 + err_pix[2],
                    pix[3] as f32 + err_pix[3],
                ];

                let (ind, pal_pix, _) = self.colormap.nearest_ind(&dith_pix);

                buf[point] = ind;

                let mut err_r = dith_pix[0] - pal_pix[0];
                let mut err_g = dith_pix[1] - pal_pix[1];
                let mut err_b = dith_pix[2] - pal_pix[2];
                let mut err_a = dith_pix[3] - pal_pix[3];

                let err_total = err_r * err_r + err_g * err_g + err_b * err_b + err_a * err_a;
                if err_total > err_threshold {
                    err_r *= 0.75;
                    err_g *= 0.75;
                    err_b *= 0.75;
                    err_a *= 0.75;
                }

                err_r *= dithering_coeff;
                err_g *= dithering_coeff;
                err_b *= dithering_coeff;
                err_a *= dithering_coeff;

                let err = &mut error_next[err_inds.0];
                err[0] += err_r * 3.0;
                err[1] += err_g * 3.0;
                err[2] += err_b * 3.0;
                err[3] += err_a * 3.0;

                let err = &mut error_next[err_inds.1];
                err[0] += err_r * 5.0;
                err[1] += err_g * 5.0;
                err[2] += err_b * 5.0;
                err[3] += err_a * 5.0;

                let err = &mut error_next[err_inds.2];
                err[0] += err_r * 1.0;
                err[1] += err_g * 1.0;
                err[2] += err_b * 1.0;
                err[3] += err_a * 1.0;

                let err = &mut error_curr[err_inds.2];
                err[0] += err_r * 7.0;
                err[1] += err_g * 7.0;
                err[2] += err_b * 7.0;
                err[3] += err_a * 7.0;
            }

            mem::swap(&mut error_curr, &mut error_next);
            error_next.fill_with(|| [0f32; 4]);
        }
    }
}

#[inline(always)]
fn pix_or_empty(pix: &[u8]) -> &[u8] {
    if pix[3] == 0 {
        return &EMPTY_PIX;
    }
    pix
}
