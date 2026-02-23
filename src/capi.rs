#![allow(clippy::missing_safety_doc)]

use alloc::boxed::Box;
use core::slice;

use crate::error::Error;
use crate::histogram::Histogram;
use crate::image::Image;
use crate::options::Options;
use crate::palette::Palette;
use crate::quantize::QuantizeResult;

#[repr(C)]
#[allow(dead_code)]
pub enum QuantizrError {
    QuantizrOk = 0,
    QuantizrValueOutOfRange = 100,
    QuantizrBufferTooSmall = 1,
}

impl From<Error> for QuantizrError {
    fn from(error: Error) -> Self {
        match error {
            Error::ValueOutOfRange => Self::QuantizrValueOutOfRange,
            Error::BufferTooSmall => Self::QuantizrBufferTooSmall,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn quantizr_new_options() -> Option<Box<Options>> {
    Some(Options::default().into())
}

#[unsafe(no_mangle)]
pub extern "C" fn quantizr_set_max_colors(options: &mut Options, colors: i32) -> QuantizrError {
    options
        .set_max_colors(colors)
        .err()
        .map_or(QuantizrError::QuantizrOk, |e| e.into())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn quantizr_create_image_rgba<'data>(
    data: *const u8,
    width: i32,
    height: i32,
) -> Option<Box<Image<'data>>> {
    let uwidth = width as usize;
    let uheight = height as usize;
    let size: usize = uwidth * uheight * 4;

    let data_slice = unsafe { slice::from_raw_parts(data, size) };
    Some(
        Image::new(data_slice, uwidth, uheight)
            .unwrap_or_else(|e| {
                // Should never happen.
                // The only way this can happen is if the buffer is too small for the provided
                // width and height. But we determine the size from the provided width and height
                // so this is impossible.
                panic!("{}", e)
            })
            .into(),
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn quantizr_create_histogram() -> Option<Box<Histogram>> {
    Some(Histogram::new().into())
}

#[unsafe(no_mangle)]
pub extern "C" fn quantizr_histogram_add_image(
    hist: &mut Histogram,
    image: &Image,
) -> QuantizrError {
    hist.add_image(image);
    QuantizrError::QuantizrOk
}

#[unsafe(no_mangle)]
pub extern "C" fn quantizr_quantize(
    image: &Image,
    options: &Options,
) -> Option<Box<QuantizeResult>> {
    Some(QuantizeResult::quantize(image, options).into())
}

#[unsafe(no_mangle)]
pub extern "C" fn quantizr_quantize_histogram(
    hist: &Histogram,
    options: &Options,
) -> Option<Box<QuantizeResult>> {
    Some(QuantizeResult::quantize_histogram(hist, options).into())
}

#[unsafe(no_mangle)]
pub extern "C" fn quantizr_set_dithering_level(
    result: &mut QuantizeResult,
    dither: f32,
) -> QuantizrError {
    result
        .set_dithering_level(dither)
        .err()
        .map_or(QuantizrError::QuantizrOk, |e| e.into())
}

#[unsafe(no_mangle)]
pub extern "C" fn quantizr_get_palette(result: &QuantizeResult) -> Option<&Palette> {
    Some(result.get_palette())
}

#[unsafe(no_mangle)]
pub extern "C" fn quantizr_get_error(result: &QuantizeResult) -> f32 {
    result.get_error()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn quantizr_remap(
    result: &QuantizeResult,
    image: &Image,
    buffer: *mut u8,
    buffer_size: usize,
) -> QuantizrError {
    let buf = unsafe { slice::from_raw_parts_mut(buffer, buffer_size) };

    result
        .remap_image(image, buf)
        .err()
        .map_or(QuantizrError::QuantizrOk, |e| e.into())
}

#[unsafe(no_mangle)]
pub extern "C" fn quantizr_free_result(result: Box<QuantizeResult>) {
    drop(result)
}

#[unsafe(no_mangle)]
pub extern "C" fn quantizr_free_histogram(hist: Box<Histogram>) {
    drop(hist)
}

#[unsafe(no_mangle)]
pub extern "C" fn quantizr_free_image(image: Box<Image>) {
    drop(image)
}

#[unsafe(no_mangle)]
pub extern "C" fn quantizr_free_options(options: Box<Options>) {
    drop(options)
}
