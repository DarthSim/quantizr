use std::os::raw::c_uchar;

use crate::quantize::{QuantizeResult,Palette};
use crate::image::{CData,Image};
use crate::options::Options;
use crate::error::Error;

#[no_mangle]
pub extern fn quantizr_new_options() -> *mut Options {
    Box::into_raw(Box::new(Options::default()))
}

#[no_mangle]
pub extern fn quantizr_set_max_colors(options: *mut Options, colors: i32) -> Error {
    let opts = unsafe { &mut *options };
    opts.set_max_colors(colors)
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
pub extern fn quantizr_set_dithering_level(result: *mut QuantizeResult, dither: f32) -> Error {
    let res = unsafe { &mut *result };
    res.set_dithering_level(dither)
}

#[no_mangle]
pub extern fn quantizr_get_palette(result: *mut QuantizeResult) -> *mut Palette {
    let res = unsafe { &*result };
    res.get_palette_ptr()
}

#[no_mangle]
pub extern fn quantizr_remap(result: *mut QuantizeResult, image: *mut Image, buffer: *mut c_uchar, buffer_size: usize) -> Error {
    let res = unsafe { &*result };
    let img = unsafe { &*image };

    let mut buf = CData::new(buffer, buffer_size);

    res.remap_image(img, &mut buf)
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
