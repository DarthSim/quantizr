use std::os::raw::c_uchar;

use crate::quantize::{QuantizeResult,Palette};
use crate::image::{CData,Image};
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
pub extern fn quantizr_set_max_colors(options: *mut Options, colors: i32) -> QuantizrError {
    let opts = unsafe { &mut *options };
    opts.set_max_colors(colors).into()
}

#[no_mangle]
pub extern fn quantizr_create_image_rgba(data: *mut c_uchar, width: i32, height: i32) -> *mut Image {
    Box::into_raw(Box::new(Image::new(data, width, height)))
}

#[no_mangle]
pub extern fn quantizr_quantize(image: *mut Image, options: *mut Options) -> *mut QuantizeResult {
    let img = unsafe { &*image };
    let opts = unsafe { &*options };

    Box::into_raw(Box::new(QuantizeResult::quantize(img, opts)))
}

#[no_mangle]
pub extern fn quantizr_set_dithering_level(result: *mut QuantizeResult, dither: f32) -> QuantizrError {
    let res = unsafe { &mut *result };
    res.set_dithering_level(dither).into()
}

#[no_mangle]
pub extern fn quantizr_get_palette(result: *mut QuantizeResult) -> *mut Palette {
    let res = unsafe { &*result };
    res.get_palette_ptr()
}

#[no_mangle]
pub extern fn quantizr_remap(result: *mut QuantizeResult, image: *mut Image, buffer: *mut c_uchar, buffer_size: usize) -> QuantizrError {
    let res = unsafe { &*result };
    let img = unsafe { &*image };

    let mut buf = CData::new(buffer, buffer_size);

    res.remap_image(img, &mut buf).into()
}

#[no_mangle]
pub extern fn quantizr_free_result(result: *mut QuantizeResult) {
    unsafe { std::mem::drop(Box::from_raw(result)) };
}

#[no_mangle]
pub extern fn quantizr_free_image(image: *mut Image) {
    unsafe { std::mem::drop(Box::from_raw(image)) };
}

#[no_mangle]
pub extern fn quantizr_free_options(options: *mut Options) {
    unsafe { std::mem::drop(Box::from_raw(options)) };
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
pub extern fn liq_set_max_colors(options: *mut LiqAttr, colors: i32) -> LiqError {
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
pub extern fn liq_image_create_rgba(_options: *mut LiqAttr, data: *mut c_uchar, width: i32, height: i32, _gamma: f64) -> *mut LiqImage {
    quantizr_create_image_rgba(data, width, height).into()
}

#[no_mangle]
pub extern fn liq_image_quantize(image: *mut LiqImage, options: *mut LiqAttr, result: *mut *mut LiqResult) -> LiqError {
    unsafe{ *result = quantizr_quantize(image, options) };
    LiqError::LiqOk
}

#[no_mangle]
pub extern fn liq_set_dithering_level(result: *mut LiqResult, dither: f32) -> LiqError {
    quantizr_set_dithering_level(result, dither).into()
}

#[no_mangle]
pub extern fn liq_get_palette(result: *mut LiqResult) -> *mut LiqPalette {
    quantizr_get_palette(result)
}

#[no_mangle]
pub extern fn liq_write_remapped_image(result: *mut LiqResult, image: *mut LiqImage, buffer: *mut c_uchar, buffer_size: usize) -> LiqError {
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

