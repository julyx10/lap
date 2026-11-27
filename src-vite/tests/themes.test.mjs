import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';

const source = fs.readFileSync(new URL('../src/common/themes.ts', import.meta.url), 'utf8');
const code = ts.transpile(source.replace(/export /g, ''), { target: ts.ScriptTarget.ES2022 });
const api = new Function('crypto', `${code}; return {
  BUILTIN_THEMES, LIGHT_THEME_IDS, DARK_THEME_IDS, THEME_COLOR_TOKENS,
  sanitizeCustomTheme, importCustomTheme, themeToCss, findTheme, DEFAULT_CUSTOM_THEME_COLORS
};`)({ randomUUID: () => 'test-id' });

const colors = Object.fromEntries(api.THEME_COLOR_TOKENS.map(([token], index) => [token, index === 0 ? '#ffffff' : '#123456']));
const theme = {
  schema: 1,
  id: 'custom-ocean',
  name: 'Ocean',
  appearance: 'dark',
  colors,
};

test('built-in theme IDs are unique and appearance-scoped', () => {
  const ids = api.BUILTIN_THEMES.map(theme => theme.id);
  assert.equal(new Set(ids).size, ids.length);
  assert.equal(ids.length, api.LIGHT_THEME_IDS.length + api.DARK_THEME_IDS.length);
  assert.ok(api.BUILTIN_THEMES.every(theme => theme.nameKey.startsWith('settings.general.theme_names.')));
});

test('editor preserves shape when editing, importing, saving, and exporting themes', async () => {
  const editor = fs.readFileSync(new URL('../src/components/ThemeEditorDialog.vue', import.meta.url), 'utf8');
  const script = editor.slice(editor.indexOf('const themeId ='), editor.indexOf('</script>'));
  const saved = [];
  let exported;
  let clipboardFailure = false;
  const initial = { ...theme, shape: { '--radius-box': '2rem', '--border': '3px' } };
  const instance = new Function('props', 'ref', 'DEFAULT_CUSTOM_THEME_COLORS', 'createCustomThemeId',
    'sanitizeCustomTheme', 'importCustomTheme', 'emit', 'navigator',
    `${ts.transpile(script, { target: ts.ScriptTarget.ES2022 })}; return { save, exportTheme, importTheme, name, error, exportMessage };`)(
    { theme: initial, appearance: 'dark' }, value => ({ value }), api.DEFAULT_CUSTOM_THEME_COLORS,
    () => 'custom-test', api.sanitizeCustomTheme, api.importCustomTheme,
    (event, value) => { if (event === 'save') saved.push(value); },
    { clipboard: { writeText: async value => {
      if (clipboardFailure) throw new Error('Clipboard unavailable');
      exported = JSON.parse(value);
    } } },
  );
  instance.save();
  assert.deepEqual(saved.at(-1).shape, initial.shape);
  await instance.exportTheme();
  assert.deepEqual(exported.shape, initial.shape);
  assert.equal(instance.exportMessage.value, 'JSON copied to clipboard.');
  const importJSON = async value => instance.importTheme({ target: {
    files: [{ text: async () => JSON.stringify(value) }], value: 'theme.json',
  } });
  await importJSON({ 'Imported dark': { primary: '#123456', '--rounded-box': '1rem', '--border-btn': '2px' } });
  instance.save();
  assert.deepEqual(saved.at(-1).shape, { '--radius-box': '1rem', '--border': '2px' });
  await instance.exportTheme();
  assert.deepEqual(exported.shape, saved.at(-1).shape);
  await importJSON({ 'No shape dark': { primary: '#123456' } });
  instance.save();
  assert.equal(saved.at(-1).shape, undefined);
  instance.name.value = '';
  await instance.exportTheme();
  assert.equal(instance.error.value, 'Enter a name and valid CSS color values.');
  assert.equal(instance.exportMessage.value, '');
  instance.name.value = 'Theme';
  clipboardFailure = true;
  await instance.exportTheme();
  assert.equal(instance.error.value, 'Could not copy JSON.');
  assert.equal(instance.exportMessage.value, '');
});

test('custom themes validate names, IDs, colors, and reject CSS injection', () => {
  const valid = api.sanitizeCustomTheme(theme);
  assert.equal(valid?.id, 'custom-ocean');
  assert.equal(api.sanitizeCustomTheme({ ...theme, id: 'not-a-custom-theme' }), null);
  assert.equal(api.sanitizeCustomTheme({ ...theme, name: '' }), null);
  assert.equal(api.sanitizeCustomTheme({ ...theme, colors: { ...colors, '--color-primary': 'red;}body{display:none' } }), null);
});

test('custom themes serialize only whitelisted variables and shape defaults', () => {
  const css = api.themeToCss(api.sanitizeCustomTheme(theme));
  assert.match(css, /\[data-theme="custom-ocean"\]/);
  assert.match(css, /--color-primary:#123456/);
  assert.match(css, /--radius-box:0\.5rem/);
  assert.doesNotMatch(css, /body/);
});

test('theme lookup falls back to the appearance default', () => {
  assert.equal(api.findTheme('custom-ocean', 'dark', [theme]).id, 'custom-ocean');
  assert.equal(api.findTheme('missing', 'light', []).id, 'light');
  assert.equal(api.findTheme('missing', 'dark', []).id, 'dark');
});

test('imports flattened daisyUI theme JSON with legacy rounded tokens', () => {
  const imported = api.importCustomTheme({
    'lap-dark': {
      primary: '#6c9bd2',
      'primary-content': '#0c1420',
      'base-100': '#1a1a1a',
      'base-content': '#d6d6d6',
      '--rounded-box': '0.5rem',
      '--rounded-btn': '0.375rem',
      '--rounded-badge': '0.25rem',
      '--border-btn': '1px',
    },
  });
  assert.equal(imported?.name, 'lap-dark');
  assert.equal(imported?.appearance, 'dark');
  assert.equal(imported?.colors['--color-base-100'], '#1a1a1a');
  assert.equal(imported?.colors['--color-primary-content'], '#0c1420');
  assert.equal(imported?.shape['--radius-box'], '0.5rem');
  assert.equal(imported?.shape['--radius-field'], '0.375rem');
  assert.equal(imported?.shape['--radius-selector'], '0.25rem');
  assert.match(api.themeToCss(imported), /--radius-field:0\.375rem/);
});
