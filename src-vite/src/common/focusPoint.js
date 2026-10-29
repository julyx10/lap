// Map a rectangle from the encoded image into the browser's EXIF-oriented image.
export function orientedFocusRect(info) {
  const rect = info?.focus;
  if (!rect || ![rect.x, rect.y, rect.width, rect.height].every(Number.isFinite)
    || rect.width <= 0 || rect.height <= 0 || rect.x - rect.width / 2 < 0
    || rect.y - rect.height / 2 < 0 || rect.x + rect.width / 2 > 1
    || rect.y + rect.height / 2 > 1) return null;
  const { x, y, width, height } = rect;
  switch (info.orientation ?? 1) {
    case 1: return { x, y, width, height };
    case 2: return { x: 1 - x, y, width, height };
    case 3: return { x: 1 - x, y: 1 - y, width, height };
    case 4: return { x, y: 1 - y, width, height };
    case 5: return { x: y, y: x, width: height, height: width };
    case 6: return { x: 1 - y, y: x, width: height, height: width };
    case 7: return { x: 1 - y, y: 1 - x, width: height, height: width };
    case 8: return { x: y, y: 1 - x, width: height, height: width };
    default: return null;
  }
}
