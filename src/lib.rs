use std::os::raw::c_uchar;

mod cluster;
mod quantize;
mod image;

use quantize::{QuantizeResult,Palette};
use image::{CData,Image};

#[repr(C)]
pub struct Attr {
    max_colors: i32,
    add_fixed_colors: bool,
}

impl Default for Attr {
    fn default() -> Self {
        Attr{
            max_colors: 256,
            add_fixed_colors: true,
        }
    }
}

#[repr(C)]
pub enum Error {
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

#[no_mangle]
pub extern fn liq_attr_create() -> *mut Attr {
    Box::into_raw(Box::new(Attr::default()))
}

#[no_mangle]
pub extern fn liq_set_max_colors(attr: *mut Attr, colors: i32) -> Error {
    if colors > 256 || colors < 2 {
        return Error::ValueOutOfRange
    }

    let at = unsafe { &mut *attr };

    at.max_colors = colors;
    at.add_fixed_colors = colors > 128;

    Error::Ok
}

#[no_mangle]
pub extern fn liq_set_quality(_attr: *mut Attr, _min: i32, _max: i32) -> Error {
    // TODO
    Error::Ok
}

#[no_mangle]
pub extern fn liq_image_create_rgba(_attr: *mut Attr, data: *mut c_uchar, width: i32, height: i32) -> *mut Image {
    Box::into_raw(Box::new(Image::new(data, width, height)))
}

#[no_mangle]
pub extern fn liq_image_quantize(image: *mut Image, attr: *mut Attr, result: *mut *mut QuantizeResult) -> Error {
    let img = unsafe { &*image };
    let at = unsafe { &*attr };

    let res = QuantizeResult::quantize(img, at);

    unsafe{ *result = Box::into_raw(Box::new(res)) }

    Error::Ok
}

#[no_mangle]
pub extern fn liq_set_dithering_level(result: *mut QuantizeResult, dither: f32) -> Error {
    if dither > 1.0 || dither < 0.0 {
        return Error::ValueOutOfRange
    }

    unsafe { (*result).dithering_level = dither; }
    Error::Ok
}

#[no_mangle]
pub extern fn liq_get_palette(result: *mut QuantizeResult) -> *mut Palette {
    let res = unsafe { &*result };
    res.get_palette_ptr()
}

#[no_mangle]
pub extern fn liq_write_remapped_image(result: *mut QuantizeResult, image: *mut Image, buffer: *mut c_uchar, buffer_size: usize) -> Error {
    let res = unsafe { &*result };
    let img = unsafe { &*image };

    if buffer_size < img.width * img.height {
        return Error::BufferTooSmall
    }

    let mut buf = CData::new(buffer, buffer_size);

    res.remap_image(img, &mut buf);

    Error::Ok
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
pub extern fn liq_attr_destroy(attr: *mut Attr) {
    unsafe { std::mem::drop(Box::from_raw(attr)) };
}




