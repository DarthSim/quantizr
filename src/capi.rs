use std::slice;

use crate::histogram::Histogram;
use crate::quantize::QuantizeResult;
use crate::palette::Palette;
use crate::image::Image;
use crate::options::Options;
use crate::error::Error;

#[repr(C)]
#[allow(dead_code)]
pub enum QuantizrError {
    QuantizrOk = 0,
    QuantizrValueOutOfRange = 100,
    QuantizrBufferTooSmall = 1,
}

impl std::convert::From<Error> for QuantizrError {
    fn from(error: Error) -> Self {
        match error {
            Error::ValueOutOfRange => Self::QuantizrValueOutOfRange,
            Error::BufferTooSmall => Self::QuantizrBufferTooSmall,
        }
    }
}

#[no_mangle]
pub extern fn quantizr_new_options() -> *mut Options {
    Box::into_raw(Box::new(Options::default()))
}

#[no_mangle]
pub unsafe extern fn quantizr_set_max_colors(options: *mut Options, colors: i32) -> QuantizrError {
    match (*options).set_max_colors(colors) {
        Ok(_) => QuantizrError::QuantizrOk,
        Err(e) => e.into()
    }
}

#[no_mangle]
pub unsafe extern fn quantizr_create_image_rgba(data: *const u8, width: i32, height: i32) -> *mut Image<'static> {
    let uwidth = width as usize;
    let uheight = height as usize;
    let size: usize = uwidth * uheight * 4;

    let data_slice = slice::from_raw_parts(data, size);
    let image = match Image::new(data_slice, uwidth, uheight) {
        Ok(img) => img,
        Err(e) => panic!("{}", e), // Should never reach this
    };

    Box::into_raw(Box::new(image))
}

#[no_mangle]
pub unsafe extern fn quantizr_create_histogram() -> *mut Histogram {
    Box::into_raw(Box::new(Histogram::new()))
}

#[no_mangle]
pub unsafe extern fn quantizr_histogram_add_image(hist: *mut Histogram, image: *const Image) -> QuantizrError {
    (*hist).add_image(&(*image));
    QuantizrError::QuantizrOk
}

#[no_mangle]
pub unsafe extern fn quantizr_quantize(image: *const Image, options: *const Options) -> *mut QuantizeResult {
    Box::into_raw(Box::new(QuantizeResult::quantize(&(*image), &(*options))))
}

#[no_mangle]
pub unsafe extern fn quantizr_quantize_histogram(hist: *const Histogram, options: *const Options) -> *mut QuantizeResult {
    Box::into_raw(Box::new(QuantizeResult::quantize_histogram(&(*hist), &(*options))))
}

#[no_mangle]
pub unsafe extern fn quantizr_set_dithering_level(result: *mut QuantizeResult, dither: f32) -> QuantizrError {
    match (*result).set_dithering_level(dither) {
        Ok(_) => QuantizrError::QuantizrOk,
        Err(e) => e.into()
    }
}

#[no_mangle]
pub unsafe extern fn quantizr_get_palette(result: *const QuantizeResult) -> *const Palette {
    (*result).get_palette()
}

#[no_mangle]
pub unsafe extern fn quantizr_get_error(result: *const QuantizeResult) -> f32 {
    (*result).get_error()
}

#[no_mangle]
pub unsafe extern fn quantizr_remap(result: *const QuantizeResult, image: *const Image, buffer: *mut u8, buffer_size: usize) -> QuantizrError {
    let mut buf = slice::from_raw_parts_mut(buffer, buffer_size);

    match (*result).remap_image(&(*image), &mut buf) {
        Ok(_) => QuantizrError::QuantizrOk,
        Err(e) => e.into()
    }
}

#[no_mangle]
pub unsafe extern fn quantizr_free_result(result: *mut QuantizeResult) {
    std::mem::drop(Box::from_raw(result))
}

#[no_mangle]
pub unsafe extern fn quantizr_free_histogram(hist: *mut Histogram) {
    std::mem::drop(Box::from_raw(hist))
}

#[no_mangle]
pub unsafe extern fn quantizr_free_image(image: *mut Image) {
    std::mem::drop(Box::from_raw(image))
}

#[no_mangle]
pub unsafe extern fn quantizr_free_options(options: *mut Options) {
    std::mem::drop(Box::from_raw(options))
}
