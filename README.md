# Quantizr

Fast C library for conversion of RGBA images to 8-bit paletted images written in Rust.

## Installation

### DEB-package

deb-packages can be downloaded from the [releases](https://github.com/DarthSim/quantizr/releases) page.

### Build from source

Quantizr written in Rust, so you need to [install it](https://www.rust-lang.org/tools/install) first. Then simply run:

```bash
./configure
make
make install
```

## Usage

```c
#include "quantizr.h"

// Create new Quantizr options.
// You're responsible for freeing it when the work is done (see below).
quantizr_options *opts = quantizr_new_options();
// (optional) Set desired number of colors. The default number is 256.
// This function returns QUANTIZR_VALUE_OUT_OF_RANGE if provided number is less than 2 or
// greater than 255.
quantizr_set_max_colors(128);
// Create new image object.
// You're responsible for freeing it when the work is done (see below).
// image_data is unsigned char array with raw RGBA pixels.
quantizr_image *img = quantizr_create_image_rgba(image_data, image_width, image_height);
// Quantize image.
// This function returns quantization result, which you're responsible to free when
// the work is done (see below).
quantizr_result *res = quantizr_quantize(img, opts);
// Set dithering level for the future remapping. The default level is 1.0.
// This function returns QUANTIZR_VALUE_OUT_OF_RANGE if provided level is less than 0.0 or
// greater than 1.0.
quantizr_set_dithering_level(res, 0.8);
// Fetch palette from the quantization result.
// Fetched pallette is read-only. You should not modify or free it.
// pal->count is a number of colors in the palette.
// pal->entries is an array of colors.
// pal->entries[i].r, pal->entries[i].g, pal->entries[i].b, and pal->entries[i].a are color channels
// of palette colors.
quantizr_palette *pal = quantizr_get_palette(res);
// Write quantized image in the provided buffer.
// The buffer should be prealocated and be large enough to fit entire image (width*height bytes).
// This function returns QUANTIZR_BUFFER_TOO_SMALL if the buffer is not large enough.
quantizr_remap(res, img, out_buffer, out_buffer_length);
// Cleanup. Free quantization result, image, and options.
quantizr_free_result(res);
quantizr_free_image(img);
quantizr_free_options(opts);
```

## Using with [libvips](https://github.com/libvips/libvips)

libvips currently doesn't have Quantizr support. That's why Quantizr has libimagequant-compatible mode. In this mode, Quantizr partly implements libimagequant API enough to be used with libvips.

deb-packages with libimagequan-compatible build can be downloaded from the [releases](https://github.com/DarthSim/quantizr/releases) page.

If you;d like to build libimagequan-compatible Quantizr from the source, run the following:

```bash
./configure --enable-imagequant-compatibility
make
make install
```

**Warning:** If you have libimagequant installed, `make install` can overwrite it's header (`libimagequant.h`) and/or pkg-config file (`imagequant.pc`).

After installing libimagequan-compatible Quantizr you can build and run libvips as usual.

## Author

Sergey "[DarthSim](https://github.com/DarthSim)" Alexandrovich

## License

Quantizr is licensed under the MIT license.

See LICENSE for the full license text.
