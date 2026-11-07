import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import vm from 'node:vm';
import { createI18n } from 'vue-i18n';
import { defineStore, createPinia, setActivePinia } from 'pinia';
import { ref, watch, nextTick } from 'vue';
import ts from 'typescript';

const source = await readFile(new URL('../src/common/i18n.js', import.meta.url), 'utf8');
const fieldSource = await readFile(new URL('../src/common/fileInfoFields.ts', import.meta.url), 'utf8');
const { defaultFileInfoFields } = await import(`data:text/javascript;base64,${Buffer.from(ts.transpile(fieldSource, { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext })).toString('base64')}`);
const storeSource = await readFile(new URL('../src/stores/configStore.js', import.meta.url), 'utf8');

async function fixture(customLoaders = {}) {
  const calls = [];
  const loaders = Object.fromEntries(['en', 'zh-CN', 'ja', 'de'].map(language => [
    `../locales/${language}.json`, async () => {
      calls.push(language);
      return { default: { title: language } };
    },
  ]));
  Object.assign(loaders, customLoaders);
  const context = vm.createContext({ console: { error() {} } });
  const synthetic = exports => new vm.SyntheticModule(Object.keys(exports), function () {
    for (const [key, value] of Object.entries(exports)) this.setExport(key, value);
  }, { context });
  const module = new vm.SourceTextModule(source, {
    context,
    initializeImportMeta(meta) { meta.glob = () => loaders; },
  });
  await module.link(() => synthetic({ createI18n }));
  await module.evaluate();
  return { calls, module, context, synthetic, ...module.namespace };
}

test('startup loads only English and the selected language, mapping zh to zh-CN', async () => {
  const { initializeLanguage, i18n, calls } = await fixture();
  assert.equal(await initializeLanguage('zh'), 'zh');
  assert.deepEqual(calls, ['en', 'zh-CN']);
  assert.equal(i18n.global.locale.value, 'zh');
  assert.equal(i18n.global.getLocaleMessage('zh').title, 'zh-CN');
  assert.equal(i18n.global.availableLocales.includes('ja'), false);
});

test('concurrent language requests share one load and subsequent requests use cache', async () => {
  const { loadLanguage, calls } = await fixture();
  await Promise.all([loadLanguage('ja'), loadLanguage('ja'), loadLanguage('ja')]);
  await loadLanguage('ja');
  assert.deepEqual(calls, ['ja']);
});

test('a failed load can be retried; failed startup language falls back to English', async () => {
  let attempt = 0;
  const { loadLanguage, initializeLanguage, i18n } = await fixture({
    '../locales/ja.json': async () => {
      if (++attempt === 1) throw new Error('read failed');
      return { default: { title: 'ja' } };
    },
  });
  await assert.rejects(loadLanguage('ja'), /read failed/);
  await loadLanguage('ja');
  assert.equal(attempt, 2);
  assert.equal(await initializeLanguage('unknown'), 'en');
  assert.equal(i18n.global.locale.value, 'en');
});

test('language configuration changes only after messages load; latest choice wins', async () => {
  let releaseJapanese;
  const pendingJapanese = new Promise(resolve => { releaseJapanese = resolve; });
  const f = await fixture({ '../locales/ja.json': () => pendingJapanese });
  await f.initializeLanguage('en');
  const storeModule = new vm.SourceTextModule(storeSource, { context: f.context });
  await storeModule.link(specifier => {
    if (specifier === 'pinia') return f.synthetic({ defineStore });
    if (specifier === '@/common/i18n') return f.module;
    if (specifier === '@/common/fileInfoFields') return f.synthetic({ defaultFileInfoFields });
    return f.synthetic({ SIDEBAR: { ALBUM: 1 }, MAP_MARKER_SIZES: [10, 20] });
  });
  await storeModule.evaluate();
  setActivePinia(createPinia());
  const config = storeModule.namespace.useConfigStore();
  const first = config.setLanguage('ja');
  assert.equal(config.settings.language, 'en');
  await config.setLanguage('de');
  releaseJapanese({ default: { title: 'ja' } });
  await first;
  assert.equal(config.settings.language, 'de');
  assert.equal(f.i18n.global.locale.value, 'de');
  assert.equal(f.i18n.global.getLocaleMessage('de').title, 'de');
});

test('first-content readiness waits for a paint and emits only once', async () => {
  const content = await readFile(new URL('../src/components/Content.vue', import.meta.url), 'utf8');
  const start = content.indexOf('const emit = defineEmits<{ startupReady: [] }>();');
  const code = ts.transpile(content.slice(start, content.indexOf('defineExpose({', start)), {
    target: ts.ScriptTarget.ES2022,
  });
  for (const welcome of [false, true]) {
    const contentReady = ref(false);
    const showWelcomeContent = ref(welcome);
    const frames = [];
    const emitted = [];
    new Function('watch', 'nextTick', 'contentReady', 'showWelcomeContent', 'requestAnimationFrame', 'defineEmits', code)(
      watch, nextTick, contentReady, showWelcomeContent,
      callback => frames.push(callback), () => event => emitted.push(event),
    );
    if (!welcome) {
      await nextTick();
      assert.equal(frames.length, 0);
      contentReady.value = true;
    }
    await nextTick();
    await nextTick();
    assert.equal(emitted.length, 0);
    frames.shift()();
    assert.equal(emitted.length, 0);
    frames.shift()();
    assert.deepEqual(emitted, ['startupReady']);
    contentReady.value = false;
    await nextTick();
    contentReady.value = true;
    await nextTick();
    assert.equal(frames.length, 0);
    assert.equal(emitted.length, 1);
  }
});

test('Home starts warmup and update checking once, including the corrupt-library view', async () => {
  const home = await readFile(new URL('../src/views/Home.vue', import.meta.url), 'utf8');
  const start = home.indexOf('let startupFinished = false;');
  const code = ts.transpile(home.slice(start, home.indexOf('onMounted(async () => {', start)), {
    target: ts.ScriptTarget.ES2022,
  });
  for (const autoCheckUpdates of [false, true]) {
    const calls = [];
    const frames = [];
    const databaseCorrupted = ref(false);
    const finish = new Function('watch', 'nextTick', 'databaseCorrupted', 'requestAnimationFrame', 'performance', 'invoke', 'config', 'checkForUpdates', 'uiStore',
      code + ';return finishStartup;')(
      watch, nextTick, databaseCorrupted, callback => frames.push(callback), { mark() {} },
      async (...args) => calls.push(args), { settings: { imageSearch: { model: 1 }, autoCheckUpdates } },
      () => calls.push(['update']), { startupReady: false },
    );
    databaseCorrupted.value = true;
    await nextTick();
    await nextTick();
    assert.equal(calls.length, 0);
    frames.shift()();
    frames.shift()();
    finish();
    assert.deepEqual(calls, autoCheckUpdates ? [['finish_startup', { model: 1 }], ['update']] : [['finish_startup', { model: 1 }]]);
  }
});
