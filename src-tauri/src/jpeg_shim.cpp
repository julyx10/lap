#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <csetjmp>
#include <vector>

#include "jpeglib.h"

#ifdef _WIN32
#include <windows.h>
#endif

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

static FILE *lap_fopen_utf8(const char *file_path, const char *mode) {
#ifdef _WIN32
  int path_len = MultiByteToWideChar(CP_UTF8, 0, file_path, -1, nullptr, 0);
  int mode_len = MultiByteToWideChar(CP_UTF8, 0, mode, -1, nullptr, 0);
  if (path_len <= 0 || mode_len <= 0) {
    return nullptr;
  }

  std::vector<wchar_t> wide_path(static_cast<size_t>(path_len));
  std::vector<wchar_t> wide_mode(static_cast<size_t>(mode_len));
  if (MultiByteToWideChar(CP_UTF8, 0, file_path, -1, wide_path.data(),
                          path_len) <= 0 ||
      MultiByteToWideChar(CP_UTF8, 0, mode, -1, wide_mode.data(), mode_len) <=
          0) {
    return nullptr;
  }

  return _wfopen(wide_path.data(), wide_mode.data());
#else
  return std::fopen(file_path, mode);
#endif
}

static bool lap_read_file(const char *file_path,
                          std::vector<unsigned char> &out) {
  FILE *file = lap_fopen_utf8(file_path, "rb");
  if (!file) {
    return false;
  }

  unsigned char buffer[64 * 1024];
  while (true) {
    size_t bytes_read = std::fread(buffer, 1, sizeof(buffer), file);
    if (bytes_read > 0) {
      out.insert(out.end(), buffer, buffer + bytes_read);
    }
    if (bytes_read < sizeof(buffer)) {
      bool ok = std::feof(file) != 0;
      std::fclose(file);
      return ok;
    }
  }
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

int lap_jpeg_decode_rgb8(const char *file_path, unsigned int target_width,
                         unsigned int target_height, unsigned int *out_width,
                         unsigned int *out_height, unsigned char **out_data,
                         char *err_buf, size_t err_buf_len) {
  if (!file_path || !out_width || !out_height || !out_data) {
    if (err_buf && err_buf_len > 0) {
      std::snprintf(err_buf, err_buf_len, "Invalid JPEG decode arguments");
    }
    return 0;
  }

  // Initialize early to prevent leak in setjmp handler
  *out_data = nullptr;

  std::vector<unsigned char> file_data;
  if (!lap_read_file(file_path, file_data)) {
    if (err_buf && err_buf_len > 0) {
      std::snprintf(err_buf, err_buf_len, "Could not open file: %s", file_path);
    }
    return 0;
  }

  if (file_data.empty()) {
    if (err_buf && err_buf_len > 0) {
      std::snprintf(err_buf, err_buf_len, "File is empty: %s", file_path);
    }
    return 0;
  }

  jpeg_decompress_struct cinfo;
  std::memset(&cinfo, 0, sizeof(cinfo));
  LapJpegErrorManager jerr;
  cinfo.err = jpeg_std_error(&jerr.pub);
  jerr.pub.error_exit = lap_jpeg_error_exit;
  jerr.message[0] = '\0';

  if (setjmp(jerr.setjmp_buffer)) {
    jpeg_destroy_decompress(&cinfo);
    if (*out_data) {
      std::free(*out_data);
      *out_data = nullptr;
    }
    if (err_buf && err_buf_len > 0) {
      std::snprintf(err_buf, err_buf_len, "%s", jerr.message);
    }
    return 0;
  }

  jpeg_create_decompress(&cinfo);
  jpeg_mem_src(&cinfo, file_data.data(),
               static_cast<unsigned long>(file_data.size()));
  jpeg_read_header(&cinfo, TRUE);

  // Calculate scaling factor (libjpeg supports 1/1, 1/2, 1/4, 1/8)
  unsigned int scale_denom = 1;
  if (target_width > 0 && target_height > 0) {
    while (scale_denom < 8 &&
           cinfo.image_width >= target_width * scale_denom * 2 &&
           cinfo.image_height >= target_height * scale_denom * 2) {
      scale_denom *= 2;
    }
  }
  cinfo.scale_num = 1;
  cinfo.scale_denom = scale_denom;
  cinfo.out_color_space = JCS_RGB;

  jpeg_start_decompress(&cinfo);

  *out_width = cinfo.output_width;
  *out_height = cinfo.output_height;
  size_t row_stride = static_cast<size_t>(cinfo.output_width) * 3;
  size_t total_size = row_stride * cinfo.output_height;

  *out_data = static_cast<unsigned char *>(std::malloc(total_size));
  if (!(*out_data)) {
    jpeg_destroy_decompress(&cinfo);
    if (err_buf && err_buf_len > 0) {
      std::snprintf(err_buf, err_buf_len, "Memory allocation failed for JPEG decode");
    }
    return 0;
  }

  while (cinfo.output_scanline < cinfo.output_height) {
    unsigned char *buffer_array[1];
    buffer_array[0] = *out_data + (cinfo.output_scanline * row_stride);
    jpeg_read_scanlines(&cinfo, buffer_array, 1);
  }

  jpeg_finish_decompress(&cinfo);
  jpeg_destroy_decompress(&cinfo);
  return 1;
}

void lap_jpeg_free_buffer(unsigned char *data) {
  if (data) {
    std::free(data);
  }
}

}
