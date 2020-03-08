mod cluster;
mod color;
mod error;
mod image;
mod options;
mod quantize;

#[cfg(not(feature = "imagequant_compat"))]
mod c_api;

#[cfg(feature = "imagequant_compat")]
mod liq_compat;


