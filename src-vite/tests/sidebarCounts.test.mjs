import ts from 'typescript';
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
const source = readFileSync(new URL('../src/components/Content.vue', import.meta.url), 'utf8');
const body = source.slice(source.indexOf('const getSidebarQueryCountAndSum ='), source.indexOf('async function getSearchSidebarCountAndSum'));
test('sidebar query and collection totals ignore browser file type without changing the content query', async () => {
  for (const kind of ['query', 'collection']) {
    for (const mask of [0, 2, 4, 5]) {
      const params = { searchFileType: mask, searchFolder: '/photos', tagId: 3 };
      let received;
      const deps = { currentQuerySource: { value: kind }, currentQueryParams: { value: params },
        currentCollectionId: { value: 42 }, getCurrentQueryCountAndSum: () => {},
        getCollectionCountAndSum: (id, p) => { assert.equal(id, 42); received = p; },
        getQueryCountAndSum: p => { received = p; } };
      const fn = new Function(...Object.keys(deps), body + ';return getSidebarQueryCountAndSum;')(...Object.values(deps));
      await fn();
      assert.deepEqual(received, { ...params, searchFileType: 0 });
      assert.equal(params.searchFileType, mask);
    }
  }
});

test('search sidebar count queries All and deduplicates text and visual matches', async () => {
  const start = source.indexOf('async function getSearchSidebarCountAndSum');
  const body = source.slice(start, source.indexOf('async function refreshRequestedSidebarCount', start));
  const text = { searchFileType: 2, searchFileName: 'cat' };
  const image = { fileType: 2, searchText: 'cat' };
  const deps = { currentQueryParams: {value: text}, currentImageSearchParams: {value: image},
    getMediaItemSize: file => Number(file?.size ?? 0) + Number(file?.companion_size ?? 0),
    imageSearchLanguageUnsupported: {value: false}, passesAlbumFilters: f => f.id !== 4,
    getQueryFiles: async p => { assert.equal(p.searchFileType, 0); return [{id:1,size:10},{id:2,size:20}]; },
    searchSimilarImages: async p => { assert.equal(p.fileType, 0); return [{id:2,size:20},{id:3,size:30},{id:4,size:40}]; } };
  const fn = new Function(...Object.keys(deps), ts.transpile(body, {target: ts.ScriptTarget.ES2022}) + ';return getSearchSidebarCountAndSum;')(...Object.values(deps));
  assert.deepEqual(await fn(), [3, 60]);
  assert.equal(text.searchFileType, 2);
  assert.equal(image.fileType, 2);
});
