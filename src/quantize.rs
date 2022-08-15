use crate::histogram::Histogram;
use crate::cluster::Cluster;
use crate::palette::Palette;
use crate::error::Error;
use crate::image::Image;
use crate::options::Options;
use crate::colormap::Colormap;

// Result of quantization
pub struct QuantizeResult {
    palette: Palette,
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
        let colormap: Colormap;

        let max_colors = attr.get_max_colors() as usize;

        if hist.map.len() <= max_colors {
            colormap = Colormap::from_histogram(&hist);
        } else {
            let root = Cluster::from_histogram(&hist);
            let clusters = root.split_into(max_colors);

            colormap = Colormap::from_clusters(&clusters);
        }

        let mut palette = Palette::default();
        colormap.generate_palette(&mut palette);

        Self{
            palette: palette,
            error: colormap.error,
            colormap: colormap,
            dithering_level: 1.0,
        }
    }

    /// Sets the dithering level.
    ///
    /// Returns [`Error::ValueOutOfRange`] if the provided value is greater
    /// than 1.0 or lesser than 0.0
    pub fn set_dithering_level(&mut self, level: f32) -> Result<(), Error> {
        if level > 1.0 || level < 0.0 {
            return Err(Error::ValueOutOfRange)
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
        &self.palette
    }

    /// Remaps the proxided [`Image`] to a slize of bytes.
    ///
    /// Returns [`Error::BufferTooSmall`] if the provided buffer is smaller
    /// than `image.width * image.height`
    pub fn remap_image(&self, image: &Image, buf: &mut [u8]) -> Result<(), Error> {
        if buf.len() < image.width * image.height {
            return Err(Error::BufferTooSmall)
        }

        if self.dithering_level > 0.0 {
            self.remap_image_dither(image, buf);
        } else {
            self.remap_image_no_dither(image, buf);
        }

        Ok(())
    }

    fn remap_image_no_dither(&self, image: &Image, buf: &mut [u8]) {
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

    fn remap_image_dither(&self, image: &Image, buf: &mut [u8]) {
        let error_size = image.width+2;
        let mut error_curr = vec![[0f32; 4]; error_size];
        let mut error_next = vec![[0f32; 4]; error_size];

        let dithering_coeff = self.dithering_level * 15.0 / 16.0 / 16.0;
        let err_threshold = self.error;
        // println!("Err threshold {}", self.error);

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
