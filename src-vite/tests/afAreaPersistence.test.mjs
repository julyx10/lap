import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createApp, computed, nextTick } from 'vue';
import { createPinia, defineStore } from 'pinia';
import { createPersistedState } from 'pinia-plugin-persistedstate';
import ts from 'typescript';

const fieldSource = readFileSync(new URL('../src/common/fileInfoFields.ts', import.meta.url), 'utf8');
const { defaultFileInfoFields } = await import(`data:text/javascript;base64,${Buffer.from(ts.transpile(fieldSource, { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext })).toString('base64')}`);
const storeSource = readFileSync(new URL('../src/stores/configStore.js', import.meta.url), 'utf8');
const useConfig = new Function('defineStore', 'SIDEBAR', 'MAP_MARKER_SIZES', 'defaultFileInfoFields',
  storeSource.replace(/^import .*;\n/gm, '').replace('export const useConfigStore', 'const useConfigStore')
  + ';return useConfigStore;')(defineStore, { ALBUM: 1 }, [], defaultFileInfoFields);
const mainSource = readFileSync(new URL('../src/main.js', import.meta.url), 'utf8');
const listenerStart = mainSource.indexOf("listen('af-area-display-changed'");
const listener = mainSource.slice(listenerStart, mainSource.indexOf('\n})', listenerStart) + 3);
const viewerSource = readFileSync(new URL('../src/components/MediaViewer.vue', import.meta.url), 'utf8');
const preference = viewerSource.slice(viewerSource.indexOf('const showAfAreas = computed('), viewerSource.indexOf('const afAreaFeedbackRequest ='));

function fixture(legacy = false) {
  const data = new Map(legacy ? [['configStore', JSON.stringify({ mediaViewer: { isPinned: false } })]] : []);
  const storage = { getItem: key => data.get(key) ?? null, setItem: (key, value) => data.set(key, value) };
  const listeners = [];
  const events = [];
  function windowStore() {
    const pinia = createPinia();
    pinia.use(createPersistedState({ storage }));
    createApp({}).use(pinia);
    const config = useConfig(pinia);
    new Function('listen', 'config', listener)((name, callback) => listeners.push(callback), config);
    const toggle = new Function('computed', 'config', 'tauriEmit', ts.transpile(preference, {target: ts.ScriptTarget.ES2022})
      + ';return showAfAreas;')(computed, config, async (name, payload) => {
        events.push({ name, payload });
        for (const callback of listeners) callback({ payload });
      });
    return { config, toggle };
  }
  return { windowStore, events };
}

test('AF display defaults to off for existing preferences and persists both toggle states', async () => {
  const f = fixture(true);
  const first = f.windowStore();
  assert.equal(first.toggle.value, false);
  assert.equal(first.config.mediaViewer.isPinned, false);
  first.toggle.value = true;
  await nextTick();
  const reopened = f.windowStore();
  assert.equal(reopened.toggle.value, true);
  reopened.toggle.value = false;
  await nextTick();
  assert.equal(f.windowStore().toggle.value, false);
});

test('all viewer panes and windows retain the preference across unrelated config writes without event loops', async () => {
  const f = fixture();
  const main = f.windowStore();
  const preview = f.windowStore();
  const settings = f.windowStore();
  preview.toggle.value = true;
  await nextTick();
  assert.equal(main.toggle.value, true);
  assert.equal(settings.toggle.value, true);
  assert.equal(f.events.length, 1);
  main.config.mediaViewer.isPinned = false;
  await nextTick();
  settings.config.settings.language = 'zh-CN';
  await nextTick();
  assert.equal(f.windowStore().toggle.value, true);
  main.toggle.value = false;
  await nextTick();
  assert.equal(preview.toggle.value, false);
  assert.equal(settings.toggle.value, false);
  assert.equal(f.events.length, 2);
  assert.equal(f.windowStore().toggle.value, false);
});
