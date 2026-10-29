// Pair labels describe the displayed logical item, not a duplicate verdict.
export function dedupFormatLabel(file, groupRawJpegPairs = true) {
  if (!file) return '';
  const extension = (path) => String(path || '').split(/[\\/]/).pop()?.match(/\.([^.]+)$/)?.[1]?.toUpperCase() || '';
  if (groupRawJpegPairs && file.media_subtype === 'raw_jpeg_pair') {
    const companion = extension(file.live_photo_video_path);
    return companion ? `RAW+${companion}` : 'RAW+…';
  }
  return Number(file.file_type) === 3 ? 'RAW' : extension(file.name || file.file_path);
}
