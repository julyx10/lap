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
