#include <stddef.h>

typedef void quantizr_options;
typedef void quantizr_image;
typedef void quantizr_result;

typedef enum {
    QUANTIZR_OK = 0,
    QUANTIZR_VALUE_OUT_OF_RANGE,
    QUANTIZR_BUFFER_TOO_SMALL,
} quantizr_error;

typedef struct {
  unsigned char r, g, b, a;
} quantizr_color;

typedef struct {
  unsigned int count;
  quantizr_color entries[256];
} quantizr_palette;

quantizr_options* quantizr_new_options(void);

quantizr_error quantizr_set_max_colors(quantizr_options* opts, int colors);

quantizr_image* quantizr_create_image_rgba(const void *data, int width, int height);

quantizr_result* quantizr_quantize(quantizr_image *const image, quantizr_options *const options);

quantizr_error quantizr_set_dithering_level(quantizr_result *result, float level);

const quantizr_palette* quantizr_get_palette(quantizr_result *result);

quantizr_error quantizr_remap(quantizr_result *result, quantizr_image *image, void *buffer, size_t buffer_size);

void quantizr_free_result(quantizr_result *result);

void quantizr_free_image(quantizr_image *image);

void quantizr_free_options(quantizr_options *opts);
