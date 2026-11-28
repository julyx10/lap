const GRID = [
  { units: 0, padding: 'p-0', x: 'pr-0', y: 'pb-0' },
  { units: 2, padding: 'p-2', x: 'pr-2', y: 'pb-2' },
  { units: 5, padding: 'p-5', x: 'pr-5', y: 'pb-5' },
];
// Values are additional space on top of the existing compact layout. One
// Tailwind spacing unit is .25rem; the app's root font is 16px * scale.
export function thumbnailSpacing(mode: unknown, scale = 1) {
  const index = mode === 0 || mode === 2 ? mode : 1;
  const preset = GRID[index];
  return { ...preset, pixels: preset.units * 4 * scale };
}

export function cardItemMetrics(scale = 1, primary = true, secondary = true, spacing: unknown = 0) {
  // p-1 on both sides, a 2px border on both sides, and .5rem between cards.
  const paddingAndBorder = 8 * scale + 4;
  const extra = spacing === 2 ? 8 : spacing === 1 ? 4 : 0;
  return {
    primaryLineHeight: 20 + extra,
    secondaryLineHeight: 16 + extra,
    frame: paddingAndBorder + 8 * scale,
    paddingAndBorder,
    labelHeight: ((primary ? 20 + extra : 0) + (secondary ? 16 + extra : 0)) * scale,
  };
}
