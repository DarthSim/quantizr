use std::slice;

use crate::quantize::{QuantizeResult,Palette};
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
            Error::Ok => Self::QuantizrOk,
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
    (*options).set_max_colors(colors).into()
}

#[no_mangle]
pub unsafe extern fn quantizr_create_image_rgba(data: *const u8, width: i32, height: i32) -> *mut Image {
    let uwidth = width as usize;
    let uheight = height as usize;
    let size: usize = uwidth * uheight * 4;

    let data_slice = slice::from_raw_parts(data, size);

    Box::into_raw(Box::new(Image::new(data_slice, uwidth, uheight)))
}

#[no_mangle]
pub unsafe extern fn quantizr_quantize(image: *const Image, options: *const Options) -> *mut QuantizeResult {
    Box::into_raw(Box::new(QuantizeResult::quantize(&(*image), &(*options))))
}

#[no_mangle]
pub unsafe extern fn quantizr_set_dithering_level(result: *mut QuantizeResult, dither: f32) -> QuantizrError {
    (*result).set_dithering_level(dither).into()
}

#[no_mangle]
pub unsafe extern fn quantizr_get_palette(result: *mut QuantizeResult) -> *const Palette {
    &(*result).palette
}

#[no_mangle]
pub unsafe extern fn quantizr_remap(result: *mut QuantizeResult, image: *mut Image, buffer: *mut u8, buffer_size: usize) -> QuantizrError {
    let mut buf = slice::from_raw_parts_mut(buffer, buffer_size);

    (*result).remap_image(&(*image), &mut buf).into()
}

#[no_mangle]
pub unsafe extern fn quantizr_free_result(result: *mut QuantizeResult) {
    std::mem::drop(Box::from_raw(result))
}

#[no_mangle]
pub unsafe extern fn quantizr_free_image(image: *mut Image) {
    std::mem::drop(Box::from_raw(image))
}

#[no_mangle]
pub unsafe extern fn quantizr_free_options(options: *mut Options) {
    std::mem::drop(Box::from_raw(options))
}

#[repr(C)]
#[allow(dead_code)]
pub enum LiqError {
    LiqOk = 0,
    LiqQualityTooLow = 99,
    LiqValueOutOfRange = 100,
    LiqOutOfMemory,
    LiqAborted,
    LiqBitmapNotAvailable,
    LiqBufferTooSmall,
    LiqInvalidPointer,
    LiqUnsupported,
}

impl std::convert::From<QuantizrError> for LiqError {
    fn from(error: QuantizrError) -> Self {
        match error {
            QuantizrError::QuantizrOk => Self::LiqOk,
            QuantizrError::QuantizrValueOutOfRange => Self::LiqValueOutOfRange,
            QuantizrError::QuantizrBufferTooSmall => Self::LiqBufferTooSmall,
        }
    }
}

type LiqAttr = Options;
type LiqImage = Image;
type LiqResult = QuantizeResult;
type LiqPalette = Palette;

#[no_mangle]
pub extern fn liq_attr_create() -> *mut LiqAttr {
    quantizr_new_options()
}

#[no_mangle]
pub unsafe extern fn liq_set_max_colors(options: *mut LiqAttr, colors: i32) -> LiqError {
    quantizr_set_max_colors(options, colors).into()
}

#[no_mangle]
pub extern fn liq_set_quality(_attr: *mut LiqAttr, _min: i32, _max: i32) -> LiqError {
    // TODO
    LiqError::LiqOk
}

#[no_mangle]
pub extern fn liq_set_speed(_attr: *mut LiqAttr, _speed: i32) -> LiqError {
    // TODO
    LiqError::LiqOk
}

#[no_mangle]
pub unsafe extern fn liq_image_create_rgba(_options: *const LiqAttr, data: *const u8, width: i32, height: i32, _gamma: f64) -> *mut LiqImage {
    quantizr_create_image_rgba(data, width, height).into()
}

#[no_mangle]
pub unsafe extern fn liq_image_quantize(image: *const LiqImage, options: *const LiqAttr, result: *mut *mut LiqResult) -> LiqError {
    *result = quantizr_quantize(image, options);
    LiqError::LiqOk
}

#[no_mangle]
pub unsafe extern fn liq_set_dithering_level(result: *mut LiqResult, dither: f32) -> LiqError {
    quantizr_set_dithering_level(result, dither).into()
}

#[no_mangle]
pub unsafe extern fn liq_get_palette(result: *mut LiqResult) -> *const LiqPalette {
    quantizr_get_palette(result)
}

#[no_mangle]
pub unsafe extern fn liq_write_remapped_image(result: *mut LiqResult, image: *mut LiqImage, buffer: *mut u8, buffer_size: usize) -> LiqError {
    quantizr_remap(result, image, buffer, buffer_size).into()
}

#[no_mangle]
pub extern fn liq_result_destroy(result: *mut LiqResult) {
    unsafe { std::mem::drop(Box::from_raw(result)) };
}

#[no_mangle]
pub extern fn liq_image_destroy(image: *mut LiqImage) {
    unsafe { std::mem::drop(Box::from_raw(image)) };
}

#[no_mangle]
pub extern fn liq_attr_destroy(options: *mut LiqAttr) {
    unsafe { std::mem::drop(Box::from_raw(options)) };
}

