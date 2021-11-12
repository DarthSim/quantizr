#include <stddef.h>

typedef void liq_attr;
typedef void liq_image;
typedef void liq_result;

typedef enum {
    LIQ_OK = 0,
    LIQ_QUALITY_TOO_LOW = 99,
    LIQ_VALUE_OUT_OF_RANGE = 100,
    LIQ_OUT_OF_MEMORY,
    LIQ_ABORTED,
    LIQ_BITMAP_NOT_AVAILABLE,
    LIQ_BUFFER_TOO_SMALL,
    LIQ_INVALID_POINTER,
    LIQ_UNSUPPORTED,
} liq_error;

typedef struct {
  unsigned char r, g, b, a;
} liq_color;

typedef struct {
  unsigned int count;
  liq_color entries[256];
} liq_palette;

liq_attr* liq_attr_create(void);

liq_error liq_set_max_colors(liq_attr* attr, int colors);

liq_error liq_set_speed(liq_attr* attr, int speed);

liq_error liq_set_quality(liq_attr* attr, int minimum, int maximum);

liq_image* liq_image_create_rgba(const liq_attr *attr, const void *bitmap, int width, int height, double gamma);

liq_error liq_image_quantize(liq_image *const input_image, liq_attr *const options, liq_result **result_output);

liq_error liq_set_dithering_level(liq_result *res, float dither_level);

const liq_palette* liq_get_palette(liq_result *result);

liq_error liq_write_remapped_image(liq_result *result, liq_image *input_image, void *buffer, size_t buffer_size);

void liq_result_destroy(liq_result *);

void liq_image_destroy(liq_image *img);

void liq_attr_destroy(liq_attr *attr);
