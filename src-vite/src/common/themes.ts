export type ThemeAppearance = 'light' | 'dark';

export const LIGHT_THEME_IDS = [
  'light', 'cupcake', 'bumblebee', 'emerald', 'corporate', 'retro',
  'cyberpunk', 'valentine', 'garden', 'lofi', 'pastel', 'fantasy',
  'wireframe', 'cmyk', 'autumn', 'acid', 'lemonade', 'winter',
  'nord', 'caramellatte', 'silk',
] as const;

export const DARK_THEME_IDS = [
  'dark', 'synthwave', 'halloween', 'forest', 'aqua', 'black',
  'luxury', 'dracula', 'business', 'night', 'coffee', 'dim',
  'sunset', 'abyss',
] as const;

const LIGHT_FALLBACK_NAMES: Record<string, string> = {
  light: 'Default', cupcake: 'Cupcake', bumblebee: 'Bumblebee', emerald: 'Emerald',
  corporate: 'Corporate', retro: 'Retro', cyberpunk: 'Cyberpunk', valentine: 'Valentine',
  garden: 'Garden', lofi: 'Lofi', pastel: 'Pastel', fantasy: 'Fantasy',
  wireframe: 'Wireframe', cmyk: 'Cmyk', autumn: 'Autumn', acid: 'Acid',
  lemonade: 'Lemonade', winter: 'Winter', nord: 'Nord', caramellatte: 'Caramellatte',
  silk: 'Silk',
};

const DARK_FALLBACK_NAMES: Record<string, string> = {
  dark: 'Default', synthwave: 'Synthwave', halloween: 'Halloween', forest: 'Forest',
  aqua: 'Aqua', black: 'Black', luxury: 'Luxury', dracula: 'Dracula',
  business: 'Business', night: 'Night', coffee: 'Coffee', dim: 'Dim',
  sunset: 'Sunset', abyss: 'Abyss',
};

export interface BuiltinTheme {
  id: string;
  appearance: ThemeAppearance;
  builtin: true;
  nameKey: string;
  fallbackName: string;
}

export interface CustomTheme {
  schema: 1;
  id: string;
  name: string;
  appearance: ThemeAppearance;
  builtin: false;
  colors: Record<string, string>;
  shape?: Record<string, string>;
}

export type ThemeDefinition = BuiltinTheme | CustomTheme;

export const BUILTIN_THEMES: BuiltinTheme[] = [
  ...LIGHT_THEME_IDS.map(id => ({
    id,
    appearance: 'light' as const,
    builtin: true as const,
    nameKey: `settings.general.theme_names.${id}`,
    fallbackName: LIGHT_FALLBACK_NAMES[id] || id,
  })),
  ...DARK_THEME_IDS.map(id => ({
    id,
    appearance: 'dark' as const,
    builtin: true as const,
    nameKey: `settings.general.theme_names.${id}`,
    fallbackName: DARK_FALLBACK_NAMES[id] || id,
  })),
];

export const THEME_COLOR_TOKENS = [
  ['--color-base-100', 'Base 100'],
  ['--color-base-200', 'Base 200'],
  ['--color-base-300', 'Base 300'],
  ['--color-base-content', 'Base content'],
  ['--color-primary', 'Primary'],
  ['--color-primary-content', 'Primary content'],
  ['--color-secondary', 'Secondary'],
  ['--color-secondary-content', 'Secondary content'],
  ['--color-accent', 'Accent'],
  ['--color-accent-content', 'Accent content'],
  ['--color-neutral', 'Neutral'],
  ['--color-neutral-content', 'Neutral content'],
  ['--color-info', 'Info'],
  ['--color-info-content', 'Info content'],
  ['--color-success', 'Success'],
  ['--color-success-content', 'Success content'],
  ['--color-warning', 'Warning'],
  ['--color-warning-content', 'Warning content'],
  ['--color-error', 'Error'],
  ['--color-error-content', 'Error content'],
] as const;

const ALLOWED_COLOR_TOKENS = new Set(THEME_COLOR_TOKENS.map(([token]) => token));

const SHAPE_TOKEN_MAP: Record<string, string> = {
  '--rounded-box': '--radius-box',
  '--rounded-btn': '--radius-field',
  '--rounded-badge': '--radius-selector',
  '--border-btn': '--border',
};
const ALLOWED_SHAPE_TOKENS = new Set([
  '--radius-selector', '--radius-field', '--radius-box',
  '--size-selector', '--size-field', '--border', '--depth', '--noise',
]);

export const DEFAULT_CUSTOM_THEME_COLORS: Record<ThemeAppearance, Record<string, string>> = {
  light: {
    '--color-base-100': '#ffffff',
    '--color-base-200': '#f2f2f2',
    '--color-base-300': '#e5e6e6',
    '--color-base-content': '#1f2937',
    '--color-primary': '#4f46e5',
    '--color-primary-content': '#ffffff',
    '--color-secondary': '#f000b8',
    '--color-secondary-content': '#ffffff',
    '--color-accent': '#37cdbe',
    '--color-accent-content': '#163835',
    '--color-neutral': '#3d4451',
    '--color-neutral-content': '#ffffff',
    '--color-info': '#3abff8',
    '--color-info-content': '#002b3d',
    '--color-success': '#36d399',
    '--color-success-content': '#003d2e',
    '--color-warning': '#fbbd23',
    '--color-warning-content': '#382800',
    '--color-error': '#f87272',
    '--color-error-content': '#470000',
  },
  dark: {
    '--color-base-100': '#1d232a',
    '--color-base-200': '#191e24',
    '--color-base-300': '#15191e',
    '--color-base-content': '#a6adbb',
    '--color-primary': '#38bdf8',
    '--color-primary-content': '#00131f',
    '--color-secondary': '#f471b5',
    '--color-secondary-content': '#1f0716',
    '--color-accent': '#22d3ee',
    '--color-accent-content': '#00212b',
    '--color-neutral': '#2a323c',
    '--color-neutral-content': '#a6adbb',
    '--color-info': '#0ea5e9',
    '--color-info-content': '#00131f',
    '--color-success': '#22c55e',
    '--color-success-content': '#001a0a',
    '--color-warning': '#f59e0b',
    '--color-warning-content': '#1f1300',
    '--color-error': '#ef4444',
    '--color-error-content': '#1f0303',
  },
};

const CUSTOM_THEME_ID = /^custom-[a-z0-9][a-z0-9-]{0,63}$/;
const SAFE_CSS_VALUE = /^(?:#[0-9a-f]{3,8}|(?:rgb|hsl|oklch|oklab|lab|lch|color)\([^;{}<>]*\)|[a-z]+)$/i;

export function createCustomThemeId() {
  const uuid = typeof crypto !== 'undefined' && 'randomUUID' in crypto
    ? crypto.randomUUID()
    : `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  return `custom-${uuid}`;
}

export function isCustomThemeId(id: unknown): id is string {
  return typeof id === 'string' && CUSTOM_THEME_ID.test(id);
}

export function isThemeAppearance(value: unknown): value is ThemeAppearance {
  return value === 'light' || value === 'dark';
}

export function isValidColorValue(value: unknown): value is string {
  return typeof value === 'string'
    && value.length > 0
    && value.length <= 160
    && SAFE_CSS_VALUE.test(value.trim());
}

function isValidShapeValue(value: unknown): value is string {
  return typeof value === 'string'
    && value.length > 0
    && value.length <= 80
    && /^(?:[0-9.]+(?:px|rem|em|%|s)?|'(?:[^'\\]|\\.)*'|"(?:[^"\\]|\\.)*")$/.test(value.trim());
}

export function sanitizeCustomTheme(input: any): CustomTheme | null {
  if (!input || input.schema !== 1) return null;
  if (!isCustomThemeId(input.id)) return null;
  const name = String(input.name || '').trim();
  if (!name || name.length > 80) return null;
  if (!isThemeAppearance(input.appearance)) return null;
  if (!input.colors || typeof input.colors !== 'object') return null;

  const colors: Record<string, string> = {};
  for (const [token] of THEME_COLOR_TOKENS) {
    const value = input.colors[token];
    if (value === undefined) continue;
    if (!isValidColorValue(value)) return null;
    colors[token] = String(value).trim();
  }
  if (Object.keys(colors).length === 0) return null;

  const shape: Record<string, string> = {};
  for (const [token, value] of Object.entries(input.shape || {})) {
    if (!ALLOWED_SHAPE_TOKENS.has(token)) continue;
    if (!isValidShapeValue(value)) return null;
    shape[token] = String(value).trim();
  }

  return {
    schema: 1,
    id: input.id,
    name,
    appearance: input.appearance,
    builtin: false,
    colors,
    ...(Object.keys(shape).length ? { shape } : {}),
  };
}

const DEFAULT_THEME_SHAPE = {
  '--radius-selector': '1rem',
  '--radius-field': '0.25rem',
  '--radius-box': '0.5rem',
  '--size-selector': '0.25rem',
  '--size-field': '0.25rem',
  '--border': '1px',
  '--depth': '1',
  '--noise': '0',
};

export function themeToCss(theme: CustomTheme): string {
  const vars = Object.entries({ ...DEFAULT_THEME_SHAPE, ...(theme.shape || {}), ...theme.colors })
    .filter(([token]) => ALLOWED_COLOR_TOKENS.has(token) || token in DEFAULT_THEME_SHAPE || ALLOWED_SHAPE_TOKENS.has(token))
    .map(([token, value]) => `${token}:${value};`)
    .join('');
  return `[data-theme="${theme.id}"]{${vars}}`;
}

function normalizeColorKey(key: string): string | null {
  const normalized = key.trim().toLowerCase();
  if (ALLOWED_COLOR_TOKENS.has(normalized)) return normalized;
  if (normalized.startsWith('--color-')) return normalized;
  const colorName = normalized.startsWith('--') ? normalized.slice(2) : normalized;
  if (colorName === 'base-content' || colorName === 'primary-content' || colorName === 'secondary-content'
    || colorName === 'accent-content' || colorName === 'neutral-content'
    || colorName === 'info-content' || colorName === 'success-content'
    || colorName === 'warning-content' || colorName === 'error-content') {
    return `--color-${colorName}`;
  }
  if (['base-100', 'base-200', 'base-300', 'primary', 'secondary', 'accent', 'neutral',
    'info', 'success', 'warning', 'error'].includes(colorName)) {
    return `--color-${colorName}`;
  }
  return null;
}

function inferAppearance(name: string, body: Record<string, any>, colors: Record<string, string>): ThemeAppearance {
  const scheme = String(body['color-scheme'] || body.colorScheme || '').toLowerCase();
  if (scheme === 'dark' || scheme === 'light') return scheme;
  if (/dark/i.test(name)) return 'dark';
  if (/light/i.test(name)) return 'light';
  const base = colors['--color-base-100'];
  if (/^#[0-9a-f]{6}$/i.test(base || '')) {
    const value = parseInt(base.slice(1), 16);
    const r = (value >> 16) & 255;
    const g = (value >> 8) & 255;
    const b = value & 255;
    return (0.2126 * r + 0.7152 * g + 0.0722 * b) < 128 ? 'dark' : 'light';
  }
  return 'dark';
}

/**
 * Accepts both Lap's canonical theme object and the flat daisyUI JSON shape:
 * { "lap-dark": { primary: "...", "base-100": "...", "--rounded-box": "..." } }
 */
export function importCustomTheme(input: any, fallbackAppearance: ThemeAppearance = 'dark'): CustomTheme | null {
  if (!input || typeof input !== 'object' || Array.isArray(input)) return null;
  if (input.schema === 1) return sanitizeCustomTheme(input);

  let name = String(input.name || '').trim();
  let body: Record<string, any> = input;
  const keys = Object.keys(input);
  if (keys.length === 1 && input[keys[0]] && typeof input[keys[0]] === 'object' && !Array.isArray(input[keys[0]])) {
    name = keys[0];
    body = input[keys[0]];
  }

  const colors: Record<string, string> = {};
  const shape: Record<string, string> = {};
  for (const [rawKey, rawValue] of Object.entries(body)) {
    if (typeof rawValue !== 'string') continue;
    const token = normalizeColorKey(rawKey);
    if (token) {
      colors[token] = rawValue;
      continue;
    }
    const shapeToken = SHAPE_TOKEN_MAP[rawKey] || rawKey;
    if (ALLOWED_SHAPE_TOKENS.has(shapeToken)) shape[shapeToken] = rawValue;
  }
  if (!Object.keys(colors).length) return null;

  return sanitizeCustomTheme({
    schema: 1,
    id: createCustomThemeId(),
    name: name || 'Custom theme',
    appearance: inferAppearance(name, body, colors) || fallbackAppearance,
    colors,
    shape,
  });
}

export function findTheme(
  id: unknown,
  appearance: ThemeAppearance,
  customThemes: CustomTheme[] = [],
): ThemeDefinition {
  const allCustom = customThemes.filter(theme => theme.appearance === appearance);
  const custom = isCustomThemeId(id) ? allCustom.find(theme => theme.id === id) : null;
  if (custom) return custom;

  const builtin = BUILTIN_THEMES.find(theme => theme.appearance === appearance && theme.id === id);
  return builtin || (appearance === 'light' ? BUILTIN_THEMES[0] : BUILTIN_THEMES[LIGHT_THEME_IDS.length]);
}

export function applyTheme(theme: ThemeDefinition): void {
  let style = document.getElementById('lap-custom-theme') as HTMLStyleElement | null;
  if (!style) {
    style = document.createElement('style');
    style.id = 'lap-custom-theme';
    document.head.appendChild(style);
  }

  // Install custom variables before switching data-theme, and remove them only
  // after switching to a built-in theme, so the frame never renders unstyled.
  if (!theme.builtin) style.textContent = themeToCss(theme);
  document.documentElement.setAttribute('data-theme', theme.id);
  document.documentElement.style.colorScheme = theme.appearance;
  if (theme.builtin) style.textContent = '';
}

export function themeLabel(theme: ThemeDefinition, t: (key: string, fallback: string) => string): string {
  return theme.builtin ? t(theme.nameKey, theme.fallbackName) : theme.name;
}
