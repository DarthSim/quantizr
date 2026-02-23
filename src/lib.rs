//! Fast library for conversion of RGBA images to 8-bit paletted images.
//!
//! This crate supports `no_std` environments with the `alloc` crate.
//! Disable the default `std` feature to use in `no_std` mode.
//!
//! ### Quantizing an image
//!
//! ```
//! let (bytes, width, height) = load_image();
//!
//! let image = quantizr::Image::new(bytes, width, height)?;
//!
//! let mut opts = quantizr::Options::default();
//! opts.set_max_colors(256)?;
//!
//! let mut result = quantizr::QuantizeResult::quantize(&image, &opts);
//! result.set_dithering_level(1.0)?;
//!
//! let mut indexes = vec![0u8; width * height];
//! result.remap_image(&image, indexes.as_mut_slice())?;
//!
//! let palette = result.get_palette();
//!
//! save_image(palette, indexes, width, height);
//! ```
//!
//! See `example/` directory for complete example code.
//!
//! ### Quantizing multiple image into a single palette
//!
//! ```
//! let mut hist = quantizr::Histogram::new();
//!
//! let (bytes1, width1, height1) = load_image1();
//! let image1 = quantizr::Image::new(bytes1, width1, height1)?;
//! hist.add_image(&image1);
//!
//! let (bytes2, width2, height2) = load_image2();
//! let image2 = quantizr::Image::new(bytes2, width2, height2)?;
//! hist.add_image(&image2);
//!
//! let mut opts = quantizr::Options::default();
//! opts.set_max_colors(256)?;
//!
//! let mut result = quantizr::QuantizeResult::quantize_histogram(&hist, &opts);
//! result.set_dithering_level(1.0)?;
//!
//! let mut indexes1 = vec![0u8; width * height];
//! result.remap_image(&image1, indexes1.as_mut_slice())?;
//!
//! let mut indexes2 = vec![0u8; width * height];
//! result.remap_image(&image2, indexes2.as_mut_slice())?;
//!
//! let palette = result.get_palette();
//!
//! save_image1(palette, indexes1, width1, height1);
//! save_image2(palette, indexes2, width2, height2);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod cluster;
mod colormap;
mod error;
mod histogram;
mod image;
mod options;
mod ord_float;
mod palette;
mod quantize;
mod vpsearch;

pub use error::Error;
pub use histogram::Histogram;
pub use image::Image;
pub use options::Options;
pub use palette::Color;
pub use palette::Palette;
pub use quantize::QuantizeResult;

#[cfg(feature = "capi")]
pub mod capi;
