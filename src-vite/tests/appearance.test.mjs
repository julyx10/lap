import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile, readdir } from 'node:fs/promises';
import vm from 'node:vm';
import { computed, effectScope, nextTick, onScopeDispose, reactive, ref, watch } from 'vue';
import ts from 'typescript';

const source = await readFile(new URL('../src/composables/useAppearance.ts', import.meta.url), 'utf8');
const utils = await readFile(new URL('../src/common/utils.ts', import.meta.url), 'utf8');
const themeCode = ts.transpile(utils.slice(utils.indexOf('export function setTheme'), utils.indexOf('/// get the select options')).replace('export ', ''));

function systemTheme(dark = false) {
  const listeners = new Set();
  return {
    matches: dark,
    addEventListener(type, listener) {
      assert.equal(type, 'change');
      listeners.add(listener);
    },
    removeEventListener(type, listener) {
      assert.equal(type, 'change');
      listeners.delete(listener);
    },
    change(dark) {
      this.matches = dark;
      for (const listener of listeners) listener({ matches: dark });
    },
    get listenerCount() { return listeners.size; },
  };
}

async function fixture(t, { appearance = 2, lightTheme = 1, darkTheme = 7, media = systemTheme() } = {}) {
  let theme;
  const config = reactive({ settings: { appearance, lightTheme, darkTheme } });
  const document = { documentElement: { setAttribute(key, value) {
    assert.equal(key, 'data-theme');
    theme = value;
  } } };
  const setTheme = new Function('document', themeCode + '; return setTheme;')(document);
  const context = vm.createContext({ window: { matchMedia(query) {
    assert.equal(query, '(prefers-color-scheme: dark)');
    return media;
  } } });
  const module = new vm.SourceTextModule(ts.transpile(source, { module: ts.ModuleKind.ESNext }), { context });
  await module.link(specifier => {
    const exports = specifier === 'vue'
      ? { computed, onScopeDispose, ref, watch }
      : specifier === '@/stores/configStore' ? { useConfigStore: () => config } : { setTheme };
    return new vm.SyntheticModule(Object.keys(exports), function () {
      for (const [key, value] of Object.entries(exports)) this.setExport(key, value);
    }, { context });
  });
  await module.evaluate();
  const scope = effectScope();
  t.after(() => scope.stop());
  scope.run(module.namespace.useAppTheme);
  return { config, media, scope, useAppearance: module.namespace.useAppearance, get theme() { return theme; } };
}

test('startup immediately uses the saved light or dark theme for the current system appearance', async t => {
  for (const dark of [false, true]) {
    const f = await fixture(t, { media: systemTheme(dark) });
    assert.equal(f.theme, dark ? 'dracula' : 'cupcake');
    assert.equal(f.config.settings.appearance, 2);
  }
});

test('the boot background follows the saved mode before Vue starts', async () => {
  const html = await readFile(new URL('../index.html', import.meta.url), 'utf8');
  const script = html.match(/<script>([\s\S]*?)<\/script>/)[1];
  for (const appearance of [0, 1, 2]) {
    for (const dark of [false, true]) {
      const style = {};
      vm.runInNewContext(script, {
        localStorage: { getItem: () => JSON.stringify({ settings: { appearance } }) },
        window: { matchMedia: () => ({ matches: dark }) },
        document: { documentElement: { style } },
      });
      const expectDark = appearance === 2 ? dark : appearance === 1;
      assert.equal(style.backgroundColor, expectDark ? '#1d232a' : '#ffffff');
    }
  }
});

test('system changes update all windows and closing one removes only its listeners', async t => {
  const media = systemTheme();
  const main = await fixture(t, { media });
  const viewer = await fixture(t, { media, lightTheme: 3, darkTheme: 9 });
  assert.equal(media.listenerCount, 2);
  media.change(true);
  await nextTick();
  assert.equal(main.theme, 'dracula');
  assert.equal(viewer.theme, 'night');
  assert.equal(main.config.settings.appearance, 2);
  main.scope.stop();
  assert.equal(media.listenerCount, 1);
  media.change(false);
  await nextTick();
  assert.equal(main.theme, 'dracula');
  assert.equal(viewer.theme, 'emerald');
  viewer.scope.stop();
  assert.equal(media.listenerCount, 0);
});

test('manual appearance stays fixed and selecting system immediately uses its latest state', async t => {
  const f = await fixture(t, { appearance: 0 });
  f.media.change(true);
  await nextTick();
  assert.equal(f.theme, 'cupcake');
  f.config.settings.appearance = 2;
  await nextTick();
  assert.equal(f.theme, 'dracula');
  f.config.settings.appearance = 1;
  f.media.change(false);
  await nextTick();
  assert.equal(f.theme, 'dracula');
  f.config.settings.appearance = 2;
  await nextTick();
  assert.equal(f.theme, 'cupcake');
});

test('settings edits the currently displayed theme and preserves the other appearance', async t => {
  const f = await fixture(t);
  const settings = await readFile(new URL('../src/views/Settings.vue', import.meta.url), 'utf8');
  const code = settings.slice(settings.indexOf('const appearance = useAppearance();'), settings.indexOf('const scaleOptions ='));
  const { currentTheme, themeOptions } = f.scope.run(() => new Function('useAppearance', 'computed', 'config', 'localeMsg',
    code + '; return { currentTheme, themeOptions };')(f.useAppearance, computed, f.config, { value: { settings: { general: {
      theme_options_light: ['Light', 'Cupcake', 'Bumblebee'],
      theme_options_dark: ['Dark', 'Synthwave', 'Halloween'],
    } } } }));
  assert.equal(themeOptions.value[0].label, 'Light');
  currentTheme.value = 2;
  await nextTick();
  assert.equal(f.theme, 'bumblebee');
  assert.equal(f.config.settings.darkTheme, 7);
  f.media.change(true);
  await nextTick();
  assert.equal(currentTheme.value, 7);
  assert.equal(themeOptions.value[0].label, 'Dark');
  currentTheme.value = 1;
  await nextTick();
  assert.equal(f.theme, 'synthwave');
  assert.equal(f.config.settings.lightTheme, 2);
  // A settings sync for the inactive appearance must not change the active palette.
  f.config.settings.lightTheme = 3;
  await nextTick();
  assert.equal(f.theme, 'synthwave');
  f.media.change(false);
  await nextTick();
  assert.equal(f.theme, 'emerald');
});

test('every app language exposes system appearance as the third option', async () => {
  const directory = new URL('../src/locales/', import.meta.url);
  for (const file of await readdir(directory)) {
    if (!file.endsWith('.json')) continue;
    const { settings: { general } } = JSON.parse(await readFile(new URL(file, directory), 'utf8'));
    assert.equal(general.appearance_options.length, 3, file);
    assert.equal(general.appearance_options[2], general.date_time_format_options[0], file);
  }
});
