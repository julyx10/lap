export const DEFAULT_REVERSE_PROMPT_MODEL = 'gpt-5.5';
export const DEFAULT_REVERSE_PROMPT_ENDPOINT = 'https://api.openai.com/v1/responses';

const TONE_INSTRUCTIONS = {
  concise: 'Keep it to one dense sentence with the most important visual details.',
  detailed: 'Include subject, setting, composition, lighting, color palette, materials, style, mood, and camera or medium cues.',
  tags: 'Use a compact comma-separated prompt with descriptive tags and no filler.',
};

const COMMON_RATIOS = [
  { w: 1, h: 1, label: '1:1' },
  { w: 4, h: 3, label: '4:3' },
  { w: 3, h: 4, label: '3:4' },
  { w: 3, h: 2, label: '3:2' },
  { w: 2, h: 3, label: '2:3' },
  { w: 16, h: 9, label: '16:9' },
  { w: 9, h: 16, label: '9:16' },
  { w: 21, h: 9, label: '21:9' },
];

export function buildReversePromptInstruction({ tone = 'detailed', extraContext = '' } = {}) {
  const toneInstruction = TONE_INSTRUCTIONS[tone] || TONE_INSTRUCTIONS.detailed;
  const context = String(extraContext || '').trim();

  return [
    'Describe this image as a reusable image-generation prompt.',
    'Focus on the visible subject, environment, composition, lighting, color palette, textures, style, mood, and camera or medium.',
    toneInstruction,
    'Return one prompt only. Do not include bullet points, labels, markdown, or explanations.',
    context ? `Additional intent: ${context}` : '',
  ].filter(Boolean).join(' ');
}

export function buildFallbackPrompt(analysis = {}) {
  const width = toPositiveInt(analysis.width);
  const height = toPositiveInt(analysis.height);
  const dimension = width && height ? `${width}x${height}` : '';
  const ratio = width && height ? getAspectRatioLabel(width, height) : '';
  const orientation = width && height ? getOrientationLabel(width, height) : '';
  const base = [orientation, ratio, 'image'].filter(Boolean).join(' ');
  const format = getFormatLabel(analysis.mimeType);
  const qualities = [
    analysis.hasAlpha ? 'transparent background support' : '',
    getBrightnessLabel(analysis.averageBrightness),
    getContrastLabel(analysis.contrast),
    getSaturationLabel(analysis.averageSaturation),
  ].filter(Boolean);
  const palette = normalizePalette(analysis.palette);

  const parts = [
    base || 'image',
    dimension,
    format,
    ...qualities,
    palette.length ? `dominant palette ${palette.join(', ')}` : '',
    'clean composition',
    'clear visual hierarchy',
    'suitable as a reusable image-generation prompt',
  ].filter(Boolean);

  return `${parts.join(', ')}.`;
}

export function normalizePalette(palette = []) {
  if (!Array.isArray(palette)) return [];

  return palette
    .map((color) => String(color || '').trim().toLowerCase())
    .filter((color) => /^#[0-9a-f]{6}$/.test(color))
    .slice(0, 5);
}

function toPositiveInt(value) {
  const number = Number(value);
  if (!Number.isFinite(number) || number <= 0) return 0;
  return Math.round(number);
}

function getAspectRatioLabel(width, height) {
  const actual = width / height;
  for (const ratio of COMMON_RATIOS) {
    if (Math.abs(actual - ratio.w / ratio.h) < 0.015) {
      return ratio.label;
    }
  }
  return `${actual.toFixed(2)}:1`;
}

function getOrientationLabel(width, height) {
  const ratio = width / height;
  if (ratio > 1.18) return 'wide';
  if (ratio < 0.85) return 'vertical';
  return 'square';
}

function getFormatLabel(mimeType) {
  const mime = String(mimeType || '').toLowerCase();
  if (!mime.startsWith('image/')) return '';
  return `${mime.slice('image/'.length).toUpperCase()} source`;
}

function getBrightnessLabel(value) {
  const number = Number(value);
  if (!Number.isFinite(number)) return '';
  if (number >= 0.67) return 'bright exposure';
  if (number <= 0.33) return 'dark exposure';
  return 'balanced exposure';
}

function getContrastLabel(value) {
  const number = Number(value);
  if (!Number.isFinite(number)) return '';
  if (number >= 0.35) return 'high-contrast';
  if (number <= 0.16) return 'soft contrast';
  return 'moderate contrast';
}

function getSaturationLabel(value) {
  const number = Number(value);
  if (!Number.isFinite(number)) return '';
  if (number >= 0.58) return 'saturated color';
  if (number <= 0.24) return 'muted color';
  return 'natural color';
}
