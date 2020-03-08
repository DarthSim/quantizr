use std::os::raw::c_uchar;

use crate::quantize::{QuantizeResult,Palette};
use crate::image::{CData,Image};
use crate::options::Options;
use crate::error::Error;

#[repr(C)]
#[allow(dead_code)]
pub enum LiqError {
    Ok = 0,
    QualityTooLow = 99,
    ValueOutOfRange = 100,
    OutOfMemory,
    Aborted,
    BitmapNotAvailable,
    BufferTooSmall,
    InvalidPointer,
    Unsupported,
}

impl std::convert::From<Error> for LiqError {
    fn from(error: Error) -> Self {
        match error {
            Error::Ok => Self::Ok,
            Error::ValueOutOfRange => Self::ValueOutOfRange,
            Error::BufferTooSmall => Self::BufferTooSmall,
        }
    }
}

#[no_mangle]
pub extern fn liq_attr_create() -> *mut Options {
    Box::into_raw(Box::new(Options::default()))
}

#[no_mangle]
pub extern fn liq_set_max_colors(options: *mut Options, colors: i32) -> LiqError {
    let opts = unsafe { &mut *options };
    opts.set_max_colors(colors).into()
}

#[no_mangle]
pub extern fn liq_set_quality(_attr: *mut Options, _min: i32, _max: i32) -> LiqError {
    // TODO
    LiqError::Ok
}

#[no_mangle]
pub extern fn liq_image_create_rgba(_options: *mut Options, data: *mut c_uchar, width: i32, height: i32) -> *mut Image {
    Box::into_raw(Box::new(Image::new(data, width, height)))
}

#[no_mangle]
pub extern fn liq_image_quantize(image: *mut Image, options: *mut Options, result: *mut *mut QuantizeResult) -> LiqError {
    let img = unsafe { &*image };
    let opts = unsafe { &*options };

    let res = QuantizeResult::quantize(img, opts);

    unsafe{ *result = Box::into_raw(Box::new(res)) }

    LiqError::Ok
}

#[no_mangle]
pub extern fn liq_set_dithering_level(result: *mut QuantizeResult, dither: f32) -> LiqError {
    let res = unsafe { &mut *result };
    res.set_dithering_level(dither).into()
}

#[no_mangle]
pub extern fn liq_get_palette(result: *mut QuantizeResult) -> *mut Palette {
    let res = unsafe { &*result };
    res.get_palette_ptr()
}

#[no_mangle]
pub extern fn liq_write_remapped_image(result: *mut QuantizeResult, image: *mut Image, buffer: *mut c_uchar, buffer_size: usize) -> LiqError {
    let res = unsafe { &*result };
    let img = unsafe { &*image };

    let mut buf = CData::new(buffer, buffer_size);

    res.remap_image(img, &mut buf).into()
}

#[no_mangle]
pub extern fn liq_result_destroy(result: *mut QuantizeResult) {
    unsafe { std::mem::drop(Box::from_raw(result)) };
}

#[no_mangle]
pub extern fn liq_image_destroy(image: *mut Image) {
    unsafe { std::mem::drop(Box::from_raw(image)) };
}

#[no_mangle]
pub extern fn liq_attr_destroy(options: *mut Options) {
    unsafe { std::mem::drop(Box::from_raw(options)) };
}
