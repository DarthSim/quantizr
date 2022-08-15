# Quantizr

Fast library for conversion of RGBA images to 8-bit paletted images. Can be used in C programs.

## Using with Rust

Add to Cargo.toml:

```toml
[dependencies]
quantizr = "1.4.0"
```

See [docs.rs](https://docs.rs/quantizr) for the library API documentation. Also, check out [examples](examples) directory for example code.

## Using with C

### Installation

Quantizr written in Rust, so you need to [install it](https://www.rust-lang.org/tools/install) first. Also, you need [cargo-c](https://github.com/lu-zero/cargo-c#installation). Then simply run:

```bash
cargo cinstall --release
```

### Usage

```c
#include "quantizr.h"

uint8_t *image_data;
int32_t image_width;
int32_t image_height;

QuantizrOptions *opts;
QuantizrImage *img;
QuantizrResult *res;
QuantizrPalette *pal;
QuantizrError err;

size_t out_buffer_length = width * height;
uint8_t *out_buffer = (uint8_t*)malloc(out_buffer_length);

// Load image data and get its dimensions. `load_image` is not a part of Quantizr
load_image(&data, &width, &height);

// Create new image object.
// You're responsible for freeing it when the work is done (see below).
// image_data is unsigned char array with raw RGBA pixels.
img = quantizr_create_image_rgba(image_data, image_width, image_height);

// Create new Quantizr options.
// You're responsible for freeing it when the work is done (see below).
opts = quantizr_new_options();

// (optional) Set desired number of colors. The default number is 256.
// This function returns QUANTIZR_VALUE_OUT_OF_RANGE if provided number is less than 2 or
// greater than 255.
err = quantizr_set_max_colors(opts, 128);
if (err != QuantizrOk) {
  // handle error...
}

// Quantize image.
// This function returns quantization result, which you're responsible to free when
// the work is done (see below).
res = quantizr_quantize(img, opts);

// Set dithering level for the future remapping. The default level is 1.0.
// This function returns QUANTIZR_VALUE_OUT_OF_RANGE if provided level is less than 0.0 or
// greater than 1.0.
err = quantizr_set_dithering_level(res, 0.8);
if (err != QuantizrOk) {
  // handle error...
}

// Write quantized image in the provided buffer.
// The buffer should be prealocated and be large enough to fit entire image (width*height bytes).
// This function returns QUANTIZR_BUFFER_TOO_SMALL if the buffer is not large enough.
err = quantizr_remap(res, img, out_buffer, out_buffer_length);
if (err != QuantizrOk) {
  // handle error...
}

// Fetch palette from the quantization result.
// Fetched pallette is read-only. You should not modify or free it.
// pal->count is a number of colors in the palette.
// pal->entries is an array of colors.
// pal->entries[i].r, pal->entries[i].g, pal->entries[i].b, and pal->entries[i].a are color channels
// of palette colors.
pal = quantizr_get_palette(res);

// Save the resulting image. `save_image` is not a part of Quantizr
save_image(pal, out_buffer, out_buffer_length, image_width, image_height);

// Cleanup. Free quantization result, image, and options.
quantizr_free_result(res);
quantizr_free_image(img);
quantizr_free_options(opts);
```

### Using histogram

Sometimes it's necessary to generate a single palette for multiple images like animation frames. In this case, you can use histogram API:

```c
#include "quantizr.h"

uint8_t *image_data;
int32_t image_width;
int32_t image_height;

QuantizrOptions *opts;
QuantizrImage *img;
QuantizrResult *res;
QuantizrPalette *pal;
QuantizrError err;

size_t out_buffer_length = width * height;
uint8_t *out_buffer = (uint8_t*)malloc(out_buffer_length);

// Create new histogram.
// You're responsible for freeing it when the work is done (see below).
QuantizrHistogram *hist = quantizr_create_histogram();

// Load image data and get its dimensions. `load_image` is not a part of Quantizr
load_image(&data, &width, &height);

// Create new image object.
// You're responsible for freeing it when the work is done (see below).
// image_data is unsigned char array with raw RGBA pixels.
img = quantizr_create_image_rgba(image_data, image_width, image_height);

// Add the image to the histogram.
// You can repeat these two steps multiple times to add multiple images to the histogram.
quantizr_histogram_add_image(hist, image);

// Create new Quantizr options.
// You're responsible for freeing it when the work is done (see below).
opts = quantizr_new_options();

// (optional) Set desired number of colors. The default number is 256.
// This function returns QUANTIZR_VALUE_OUT_OF_RANGE if provided number is less than 2 or
// greater than 255.
err = quantizr_set_max_colors(opts, 128);
if (err != QuantizrOk) {
  // handle error...
}

// Quantize histogram.
// This function returns quantization result, which you're responsible to free when
// the work is done (see below).
QuantizrResult *res = quantizr_quantize_histogram(hist, opts);

// Set dithering level for the future remapping. The default level is 1.0.
// This function returns QUANTIZR_VALUE_OUT_OF_RANGE if provided level is less than 0.0 or
// greater than 1.0.
err = quantizr_set_dithering_level(res, 0.8);
if (err != QuantizrOk) {
  // handle error...
}

// Write quantized image in the provided buffer.
// The buffer should be prealocated and be large enough to fit entire image (width*height bytes).
// This function returns QUANTIZR_BUFFER_TOO_SMALL if the buffer is not large enough.
err = quantizr_remap(res, img, out_buffer, out_buffer_length);
if (err != QuantizrOk) {
  // handle error...
}

// Fetch palette from the quantization result.
// Fetched pallette is read-only. You should not modify or free it.
// pal->count is a number of colors in the palette.
// pal->entries is an array of colors.
// pal->entries[i].r, pal->entries[i].g, pal->entries[i].b, and pal->entries[i].a are color channels
// of palette colors.
pal = quantizr_get_palette(res);

// Save the resulting image. `save_image` is not a part of Quantizr
save_image(pal, out_buffer, out_buffer_length, image_width, image_height);

// Cleanup. Free quantization result, image, and options.
quantizr_free_result(res);
quantizr_free_histogram(hist);
quantizr_free_image(img);
quantizr_free_options(opts);
```

## Using with [libvips](https://github.com/libvips/libvips)

libvips 8.13+ has first-class support of Quantizr.

Quantizr 1.3+ can't be used with earlier versions of libvips. If you want to use Quantizr with an earlier version of libvips, use Quantizr 1.2 and follow [the instructions](https://github.com/DarthSim/quantizr/tree/v1.2.0#using-with-libvips).

## Author

Sergey "[DarthSim](https://github.com/DarthSim)" Alexandrovich

## License

Quantizr is licensed under the MIT license.

See LICENSE for the full license text.
