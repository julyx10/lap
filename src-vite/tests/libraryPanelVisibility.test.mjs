import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createApp, computed, markRaw, nextTick, reactive, ref } from 'vue';
import { createPinia, defineStore } from 'pinia';
import { createPersistedState } from 'pinia-plugin-persistedstate';
import ts from 'typescript';

const fieldsSource = readFileSync(
  new URL('../src/common/fileInfoFields.ts', import.meta.url),
  'utf8',
);
const { defaultFileInfoFields } = await import(
  `data:text/javascript;base64,${Buffer.from(ts.transpile(fieldsSource, { module: ts.ModuleKind.ESNext })).toString('base64')}`
);
const storeSource = readFileSync(new URL('../src/stores/configStore.js', import.meta.url), 'utf8');
const useConfig = new Function(
  'defineStore',
  'SIDEBAR',
  'MAP_MARKER_SIZES',
  'defaultFileInfoFields',
  storeSource
    .replace(/^import .*;\n/gm, '')
    .replace('export const useConfigStore', 'const useConfigStore') + ';return useConfigStore;',
)(defineStore, { ALBUM: 1 }, [], defaultFileInfoFields);
const source = readFileSync(new URL('../src/components/Library.vue', import.meta.url), 'utf8');
const menuSource = source.slice(
  source.indexOf('type SectionKey ='),
  source.indexOf('const libraryCounts ='),
);
const keys = ['today', 'recent', 'favorites', 'ratings', 'culling', 'subjects', 'collections'];

test('old settings default to all sections visible and each visibility choice survives reopening', async () => {
  const data = new Map([['configStore', JSON.stringify({ main: { sidebarIndex: 1 } })]]);
  const storage = {
    getItem: (key) => data.get(key) ?? null,
    setItem: (key, value) => data.set(key, value),
  };
  const open = () => {
    const pinia = createPinia();
    pinia.use(createPersistedState({ storage }));
    createApp({}).use(pinia);
    return useConfig(pinia);
  };
  const config = open();
  for (const key of keys) {
    assert.equal(config.libraryPanel[key], true);
    config.libraryPanel[key] = false;
  }
  await nextTick();
  const reopened = open();
  for (const key of keys) assert.equal(reopened.libraryPanel[key], false);
  reopened.libraryPanel.favorites = true;
  await nextTick();
  assert.equal(open().libraryPanel.favorites, true);
  assert.equal(open().libraryPanel.ratings, false);
});

test('hiding an active section restores All items and hiding Collections leaves its active view', () => {
  const LIB_ITEM = {
    ALL: 'all',
    TODAY: 'today',
    RECENT: 'recent',
    FAV: 'favorite',
    RATINGS: 'rating',
    CULLING: 'culling',
    SUBJECTS: 'subject',
  };
  const config = reactive({
    libraryPanel: Object.fromEntries(keys.map((key) => [key, true])),
    main: { sidebarIndex: 0 },
  });
  const libConfig = reactive({ activePane: 'main', library: { item: LIB_ITEM.ALL } });
  const words = new Proxy({}, { get: (_, key) => String(key) });
  const localeMsg = ref({
    library: words,
    favorite: words,
    rating: words,
    culling: words,
    subject: words,
    collection: words,
  });
  const selected = [];
  const menu = new Function(
    'computed',
    'markRaw',
    'config',
    'libConfig',
    'localeMsg',
    'LIB_ITEM',
    'SIDEBAR',
    'IconOk',
    'isActiveLibraryView',
    'selectItem',
    ts.transpile(menuSource) + ';return visibilityMenuItems;',
  )(
    computed,
    markRaw,
    config,
    libConfig,
    localeMsg,
    LIB_ITEM,
    { LIBRARY: 0 },
    {},
    () => libConfig.activePane === 'main',
    (item) => {
      libConfig.library.item = item;
      selected.push(item);
    },
  );
  for (const [index, item] of [
    LIB_ITEM.TODAY,
    LIB_ITEM.RECENT,
    LIB_ITEM.FAV,
    LIB_ITEM.RATINGS,
    LIB_ITEM.CULLING,
    LIB_ITEM.SUBJECTS,
  ].entries()) {
    libConfig.library.item = item;
    menu.value[index].action();
    assert.equal(config.libraryPanel[keys[index]], false);
    assert.equal(libConfig.library.item, LIB_ITEM.ALL);
    assert.equal(menu.value[index].icon, undefined);
  }
  libConfig.activePane = 'collection';
  menu.value[6].action();
  assert.equal(config.libraryPanel.collections, false);
  assert.equal(libConfig.activePane, 'main');
  assert.equal(selected.length, 7);
});
