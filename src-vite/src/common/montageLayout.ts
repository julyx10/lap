// Photo montage layouts (grid, mosaic, pile).
// Pure functions: the result is expressed relative to the page (0..1 on each axis),
// so the preview and the backend render (render_montage) share one layout.

export type MontageMode = 'grid' | 'mosaic' | 'pile';

export interface MontagePhoto {
  fileId: number;
  ratio: number; // width / height, upright
}

export interface MontageItem {
  fileId: number;
  x: number; // frame (border included), relative to the page
  y: number;
  w: number;
  h: number;
  rotation: number; // degrees, clockwise
  border: number;   // relative to the page width
}

export interface MontageStyle {
  spacing: number;  // grid / mosaic gap, relative to the page width
  border: number;   // photo border, relative to the page width (0 = none)
  rotation: number; // pile: maximum random rotation, in degrees
}

// Deterministic PRNG (mulberry32): the same seed always gives the same montage.
function createRandom(seed: number) {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

export function shuffled<T>(items: T[], random: () => number = Math.random): T[] {
  const result = items.slice();
  for (let i = result.length - 1; i > 0; i--) {
    const j = Math.floor(random() * (i + 1));
    [result[i], result[j]] = [result[j], result[i]];
  }
  return result;
}

// Layouts are computed in a page of width 1 and height pageH = 1 / pageRatio,
// then converted to page-relative coordinates.
type Box = { fileId: number; x: number; y: number; w: number; h: number };

function gridLayout(photos: MontagePhoto[], pageH: number, gap: number): Box[] {
  const n = photos.length;
  let best = { cols: 1, score: Infinity };
  for (let cols = 1; cols <= n; cols++) {
    const rows = Math.ceil(n / cols);
    const cellW = (1 - (cols + 1) * gap) / cols;
    const cellH = (pageH - (rows + 1) * gap) / rows;
    if (cellW <= 0 || cellH <= 0) continue;
    // closest to square cells, then fewest empty cells
    const score = Math.abs(Math.log(cellW / cellH)) + (rows * cols - n) * 0.01;
    if (score < best.score) best = { cols, score };
  }

  const cols = best.cols;
  const rows = Math.ceil(n / cols);
  const cellW = (1 - (cols + 1) * gap) / cols;
  const cellH = (pageH - (rows + 1) * gap) / rows;
  return photos.map((photo, i) => {
    const row = Math.floor(i / cols);
    const inRow = row === rows - 1 ? n - row * cols : cols;
    const offset = (cols - inRow) * (cellW + gap) / 2; // center the last row
    return {
      fileId: photo.fileId,
      x: gap + (i % cols) * (cellW + gap) + offset,
      y: gap + row * (cellH + gap),
      w: cellW,
      h: cellH,
    };
  });
}

// Split photos, in order, into `rowCount` rows of similar total aspect ratio.
function splitRows(photos: MontagePhoto[], rowCount: number): MontagePhoto[][] {
  const total = photos.reduce((sum, p) => sum + p.ratio, 0);
  const rows: MontagePhoto[][] = Array.from({ length: rowCount }, () => []);
  let cumulative = 0;
  for (const photo of photos) {
    const index = Math.min(rowCount - 1, Math.floor((cumulative + photo.ratio / 2) / (total / rowCount)));
    rows[index].push(photo);
    cumulative += photo.ratio;
  }
  return rows.filter(row => row.length > 0);
}

function mosaicLayout(photos: MontagePhoto[], pageH: number, gap: number): Box[] {
  // Justify every row to the page width, pick the row count whose natural height
  // is closest to the page, then stretch the rows to fill it exactly
  // (the backend center-crops each photo to its cell).
  let best: { rows: MontagePhoto[][]; heights: number[]; distortion: number } | null = null;
  for (let rowCount = 1; rowCount <= photos.length; rowCount++) {
    const rows = splitRows(photos, rowCount);
    const heights = rows.map(row => (1 - (row.length + 1) * gap) / row.reduce((sum, p) => sum + p.ratio, 0));
    const available = pageH - (rows.length + 1) * gap;
    const natural = heights.reduce((sum, h) => sum + h, 0);
    if (available <= 0 || heights.some(h => h <= 0)) continue;
    const distortion = Math.abs(Math.log(available / natural));
    if (!best || distortion < best.distortion) best = { rows, heights: heights.map(h => h * available / natural), distortion };
  }
  if (!best) return gridLayout(photos, pageH, gap);

  const boxes: Box[] = [];
  let y = gap;
  best.rows.forEach((row, r) => {
    const rowRatio = row.reduce((sum, p) => sum + p.ratio, 0);
    const rowWidth = 1 - (row.length + 1) * gap;
    let x = gap;
    for (const photo of row) {
      const w = rowWidth * photo.ratio / rowRatio;
      boxes.push({ fileId: photo.fileId, x, y, w, h: best!.heights[r] });
      x += w + gap;
    }
    y += best!.heights[r] + gap;
  });
  return boxes;
}

function pileLayout(photos: MontagePhoto[], pageH: number, border: number, maxRotation: number, random: () => number): (Box & { rotation: number })[] {
  const n = photos.length;
  const shortSide = Math.min(1, pageH);
  const baseSize = shortSide * Math.min(0.5, Math.max(0.2, 1.1 / Math.sqrt(n)));
  // spread centers over the page: one cell of a coarse grid per photo, jittered
  const cols = Math.ceil(Math.sqrt(n / pageH));
  const rows = Math.ceil(n / cols);
  const cells = shuffled(Array.from({ length: cols * rows }, (_, i) => i), random).slice(0, n);

  return photos.map((photo, i) => {
    const longSide = baseSize * (0.85 + random() * 0.3);
    const innerW = photo.ratio >= 1 ? longSide : longSide * photo.ratio;
    const innerH = photo.ratio >= 1 ? longSide / photo.ratio : longSide;
    const w = innerW + 2 * border;
    const h = innerH + 2 * border;
    const cell = cells[i];
    const cx = ((cell % cols) + 0.2 + random() * 0.6) / cols;
    const cy = (Math.floor(cell / cols) + 0.2 + random() * 0.6) / rows * pageH;
    return {
      fileId: photo.fileId,
      x: Math.min(Math.max(cx - w / 2, 0), Math.max(0, 1 - w)),
      y: Math.min(Math.max(cy - h / 2, 0), Math.max(0, pageH - h)),
      w,
      h,
      rotation: (random() * 2 - 1) * maxRotation,
    };
  });
}

export function computeMontageLayout(
  photos: MontagePhoto[],
  pageRatio: number, // page width / height
  mode: MontageMode,
  style: MontageStyle,
  seed: number, // pile placement; photos keep the given order in every mode
): MontageItem[] {
  if (photos.length === 0 || !(pageRatio > 0)) return [];
  const pageH = 1 / pageRatio;

  const boxes = mode === 'pile'
    ? pileLayout(photos, pageH, style.border, style.rotation, createRandom(seed))
    : (mode === 'mosaic' ? mosaicLayout : gridLayout)(photos, pageH, style.spacing).map(box => ({ ...box, rotation: 0 }));

  return boxes.map(box => ({
    fileId: box.fileId,
    x: box.x,
    y: box.y / pageH,
    w: box.w,
    h: box.h / pageH,
    rotation: box.rotation,
    border: style.border,
  }));
}
