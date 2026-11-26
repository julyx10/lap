import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import ts from 'typescript';

const api = readFileSync(new URL('../src/common/api.js', import.meta.url), 'utf8');
const content = readFileSync(new URL('../src/components/Content.vue', import.meta.url), 'utf8');

test('shared count APIs leave sidebar predicates unchanged', async () => {
  for (const name of ['getQueryCountAndSum', 'getSmartQueryCountAndSum', 'getCollectionCountAndSum']) {
    const start = api.indexOf(`export async function ${name}(`);
    const end = api.indexOf('\nexport ', start + 1);
    const source = api.slice(start, end).replace('export ', '');
    let passed;
    const call = new Function('invoke', source + `; return ${name};`)(async (_command, payload) => { passed = payload.params; return [10, 100]; });
    const params = { searchFolder: '/photos' };
    await (name.startsWith('getCollection') ? call(1, params) : call(params));
    assert.deepEqual(passed, params);
    assert.equal('browserFilters' in passed, false);
  }
});

test('browser query snapshots filters while explicit empty filters bypass them', () => {
  const start = content.indexOf('function browserQuery(');
  const end = content.indexOf('\nconst ', start);
  const code = ts.transpile(content.slice(start, end), { target: ts.ScriptTarget.ES2022 });
  const filters = { rating: 5, tags: 0 };
  const query = new Function('browserFilters', 'config', 'SIDEBAR', 'isScanStreamingMode', code + '; return browserQuery;')(filters, { main: { sidebarIndex: 0 } }, { SMART_ALBUM: 9 }, { value: false });
  const snapshot = query({ searchFolder: '/photos' });
  filters.tags = 1;
  assert.equal(snapshot.browserFilters.tags, 0);
  assert.equal(snapshot.searchFolder, '/photos');
  assert.deepEqual(query({ browserFilters: {} }).browserFilters, {});
  const smart = new Function('browserFilters', 'config', 'SIDEBAR', 'isScanStreamingMode', code + '; return browserQuery;')(filters, { main: { sidebarIndex: 9 } }, { SMART_ALBUM: 9 }, { value: false });
  assert.deepEqual(smart({ searchFolder: '/photos' }).browserFilters, {});
  const scanning = new Function('browserFilters', 'config', 'SIDEBAR', 'isScanStreamingMode', code + '; return browserQuery;')(filters, { main: { sidebarIndex: 0 } }, { SMART_ALBUM: 9 }, { value: true });
  assert.deepEqual(scanning({ searchFolder: '/photos' }).browserFilters, {});
});

test('local browser predicates keep unified search text and visual results consistent', async () => {
  const filterSource = readFileSync(new URL('../src/common/browserFilters.ts', import.meta.url), 'utf8');
  const start = filterSource.indexOf('export function passesBrowserFilters(');
  const end = filterSource.indexOf('\n}', start) + 2;
  const source = filterSource.slice(start, end).replace('export ', '');
  const passes = new Function('browserFilters', ts.transpile(source, { target: ts.ScriptTarget.ES2022 }) + '; return passesBrowserFilters;')({});
  const filters = (overrides = {}) => ({ favorite: -1, rating: -1, culling: -1, tags: -1, ratingOp: 'gte', ...overrides });
  assert.equal(passes({ is_favorite: 1, rating: 4, culling_flag: 1, has_tags: true }, filters()), true);
  assert.equal(passes({ is_favorite: 1, rating: 4, culling_flag: 1, has_tags: true }, filters({ rating: 5 })), false);
  assert.equal(passes({ rating: 0, has_tags: false }, filters({ rating: 0 })), true);
  assert.equal(passes({ rating: 3 }, filters({ rating: 4, ratingOp: 'lte' })), true);
  assert.equal(passes({ has_tags: false }, filters({ tags: 1 })), false);
  assert.equal(passes({ has_tags: true }, filters({ tags: 0 })), false);
});

test('unified search filters locally and smart albums bypass browser predicates', () => {
  const content = readFileSync(new URL('../src/components/Content.vue', import.meta.url), 'utf8');
  assert.match(content, /browserFilters: \{\},/);
  assert.match(content, /allMatches\.length/);
  assert.match(content, /passesBrowserFilters\(file\)/);
  assert.match(content, /config\.main\.sidebarIndex === SIDEBAR\.SMART_ALBUM/);
});

test('scoped search preserves category count predicates after replacing the active query', async () => {
  const start = content.indexOf('  // Keep sidebar totals tied');
  const end = content.indexOf('  const textParams =', start);
  const capture = ts.transpile(content.slice(start, end), { target: ts.ScriptTarget.ES2022 });
  for (const source of ['query', 'collection', 'smart', 'search']) {
    const query = { value: { searchFolder: '/photos', rating: 4 } };
    const smart = { value: { rules: [{ field: 'rating', value: 4 }] } };
    let called;
    const count = (kind) => async (...args) => { called = [kind, ...args]; return [100, 1000]; };
    const resolver = new Function('scopeFileIds', 'currentQuerySource', 'currentCollectionId',
      'currentQueryParams', 'currentSmartQueryParams', 'currentImageSearchParams',
      'getQueryCountAndSum', 'getCollectionCountAndSum', 'getSmartQueryCountAndSum',
      'searchSimilarImages', 'passesAlbumFilters', 'getMediaItemSize',
      `let searchScopeCountAndSum; ${capture}; return searchScopeCountAndSum;`)(
      [1, 2], { value: source }, { value: 7 }, query, smart,
      { value: { searchText: 'subject prompt', fileType: 4 } },
      count('query'), count('collection'), count('smart'),
      async (params) => { called = ['image', params]; return [{ size: 20 }, { size: 30, album_visible: false }]; },
      file => file.album_visible !== false, file => file.size,
    );
    query.value = { searchFileName: 'keyword' };
    smart.value = null;
    assert.deepEqual(await resolver(), source === 'search' ? [1, 20] : [100, 1000]);
    const params = called.at(-1);
    if (source === 'query' || source === 'collection') {
      assert.equal(params.searchFolder, '/photos');
      assert.equal(params.rating, 4);
      assert.equal(params.searchFileName, undefined);
      if (source === 'collection') assert.equal(called[1], 7);
    } else if (source === 'smart') assert.equal(params.rules[0].value, 4);
    else assert.equal(params.searchText, 'subject prompt');
    assert.equal(source === 'search' ? params.fileType : params.searchFileType, 0);
  }
});

test('smart album search ignores disabled browser filters and media type', () => {
  const start = content.indexOf('    const matchesFilters =');
  const end = content.indexOf('    const textMatches =', start);
  const create = new Function('isBrowserFilterDisabled', 'config', 'passesBrowserFilters',
    `${ts.transpile(content.slice(start, end))}; return matchesFilters;`);
  const config = { search: { fileType: 4 } };
  const rejectedByRating = () => false;
  assert.equal(create({ value: true }, config, rejectedByRating)({ file_type: 1 }), true);
  assert.equal(create({ value: false }, config, rejectedByRating)({ file_type: 3 }), false);
  assert.equal(create({ value: false }, config, () => true)({ file_type: 1 }), false);
  assert.equal(create({ value: false }, config, () => true)({ file_type: 3 }), true);
});

test('removing a collection search result rebuilds flat and grouped search results', async () => {
  const start = content.indexOf('async function removeFileFromCurrentCollection(');
  const end = content.indexOf('\nasync function removeSelectedFromCollection(', start);
  for (const grouped of [false, true]) {
    const calls = [];
    const remove = new Function('fileList', 'currentCollectionId', 'currentQuerySource',
      'groupedModeActive', 'removeFilesFromCollection', 'updateContent', 'tauriEmit', 'toast', 't',
      `${ts.transpile(content.slice(start, end))}; return removeFileFromCurrentCollection;`)(
      { value: [{ id: 42 }] }, { value: 7 }, { value: 'search' }, { value: grouped },
      async (id, files) => calls.push(['remove', id, files]),
      async force => calls.push(['refresh', force]),
      (_event, payload) => calls.push(['event', payload.collectionId]),
      { success() {} }, () => '',
    );
    await remove(0);
    assert.deepEqual(calls, [['remove', 7, [42]], ['refresh', true], ['event', 7]]);
  }
});

test('disabled toolbar search rejects shortcut focus and submission without side effects', async () => {
  const source = readFileSync(new URL('../src/components/ToolbarSearch.vue', import.meta.url), 'utf8');
  const props = { disabled: true };
  const expanded = { value: false };
  const focusCode = source.slice(source.indexOf('async function focus()'), source.indexOf('let historyRequest'));
  const focus = new Function('props', 'expanded', `${ts.transpile(focusCode)}; return focus;`)(props, expanded);
  await focus();
  assert.equal(expanded.value, false);
  const submitCode = source.slice(source.indexOf('function submit('), source.indexOf('function clearHistory('));
  const submit = new Function('props', `${ts.transpile(submitCode)}; return submit;`)(props);
  submit('keyword');
  submit('');
  assert.match(content, /<ToolbarSearch[^>]*:disabled="isScanStreamingMode"/);
});

test('toolbar search exposes combobox semantics and mounts its outside listener', () => {
  const source = readFileSync(new URL('../src/components/ToolbarSearch.vue', import.meta.url), 'utf8');
  assert.match(source, /role="combobox"/);
  assert.match(source, /aria-controls="toolbar-search-history"/);
  assert.match(source, /role="listbox"/);
  assert.match(source, /onMounted\(\(\) => \{\s*document\.addEventListener\("pointerdown", outside\)/);
});

test('active toolbar search is deferred while the selected album streams, then reapplied', () => {
  const source = readFileSync(new URL('../src/components/Content.vue', import.meta.url), 'utf8');
  assert.match(source, /const hasActiveToolbarSearch = computed/);
  assert.match(source, /hasActiveToolbarSearch\.value/);
  assert.match(source, /hasActiveBrowserFilters \|\| hasActiveToolbarSearch\.value/);
});

test('toolbar search guard is shared by album, collection, smart, and visual search paths', () => {
  const source = readFileSync(new URL('../src/components/Content.vue', import.meta.url), 'utf8');
  assert.match(source, /const hasActiveToolbarSearch = computed/);
  assert.match(source, /hasActiveToolbarSearch\.value/);
  assert.equal((source.match(/hasActiveToolbarSearch\.value/g) || []).length >= 4, true);
});
