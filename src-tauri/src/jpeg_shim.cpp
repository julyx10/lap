#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <csetjmp>

#include "jpeglib.h"

extern "C" {

struct LapJpegErrorManager {
  jpeg_error_mgr pub;
  jmp_buf setjmp_buffer;
  char message[JMSG_LENGTH_MAX];
};

static void lap_jpeg_error_exit(j_common_ptr cinfo) {
  auto *err = reinterpret_cast<LapJpegErrorManager *>(cinfo->err);
  (*cinfo->err->format_message)(cinfo, err->message);
  longjmp(err->setjmp_buffer, 1);
}

int lap_jpeg_encode_rgb8(const unsigned char *rgb_data, unsigned int width,
                         unsigned int height, int quality,
                         unsigned char **out_data, unsigned long *out_len,
                         char *err_buf, size_t err_buf_len) {
  if (!rgb_data || width == 0 || height == 0 || !out_data || !out_len) {
    if (err_buf && err_buf_len > 0) {
      std::snprintf(err_buf, err_buf_len, "Invalid JPEG encode arguments");
    }
    return 0;
  }

  *out_data = nullptr;
  *out_len = 0;

  jpeg_compress_struct cinfo;
  std::memset(&cinfo, 0, sizeof(cinfo));
  LapJpegErrorManager jerr;
  cinfo.err = jpeg_std_error(&jerr.pub);
  jerr.pub.error_exit = lap_jpeg_error_exit;
  jerr.message[0] = '\0';

  if (setjmp(jerr.setjmp_buffer)) {
    jpeg_destroy_compress(&cinfo);
    if (err_buf && err_buf_len > 0) {
      if (jerr.message[0] != '\0') {
        std::snprintf(err_buf, err_buf_len, "%s", jerr.message);
      } else {
        std::snprintf(err_buf, err_buf_len, "libjpeg-turbo encode failed");
      }
    }
    if (*out_data) {
      std::free(*out_data);
      *out_data = nullptr;
    }
    *out_len = 0;
    return 0;
  }

  jpeg_create_compress(&cinfo);
  jpeg_mem_dest(&cinfo, out_data, out_len);
  cinfo.image_width = width;
  cinfo.image_height = height;
  cinfo.input_components = 3;
  cinfo.in_color_space = JCS_RGB;

  jpeg_set_defaults(&cinfo);
  jpeg_set_quality(&cinfo, quality, TRUE);
  jpeg_start_compress(&cinfo, TRUE);

  while (cinfo.next_scanline < cinfo.image_height) {
    JSAMPROW row = const_cast<JSAMPROW>(
        rgb_data + static_cast<size_t>(cinfo.next_scanline) * width * 3);
    jpeg_write_scanlines(&cinfo, &row, 1);
  }

  jpeg_finish_compress(&cinfo);
  jpeg_destroy_compress(&cinfo);
  return 1;
}

void lap_jpeg_free_buffer(unsigned char *data) {
  if (data) {
    std::free(data);
  }
}

}
