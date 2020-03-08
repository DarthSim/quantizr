mod cluster;
mod quantize;
mod image;
mod options;
mod error;

#[cfg(not(feature = "imagequant_compat"))]
mod c_api;

#[cfg(feature = "imagequant_compat")]
mod liq_compat;


