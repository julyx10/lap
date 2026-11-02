import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { computed, reactive, ref } from 'vue';
import ts from 'typescript';

const source = readFileSync(new URL('../src/components/Content.vue', import.meta.url), 'utf8');
const between = (start, end) => source.slice(source.indexOf(start), source.indexOf(end, source.indexOf(start)));
const functions = between('const FILE_TYPE_IMAGE =', 'const emptyFilesMessage =')
  + between('const activeFileTypeMask =', 'const handleSortTypeSelect =');
function fixture(smart = false) {
  const config = reactive({ search: { fileType: 0, favoritesOnly: false } });
  const album = { query: { rules: [{ field: 'rating', value: 5 }] } };
  const deps = { config, computed, isFixedAiResultView: ref(false), isSmartAlbumView: ref(smart),
    smartAlbumFileTypeMask: ref(0), tempViewMode: ref('none'), isScanStreamingMode: ref(false),
    rememberFocusedFileForPresentationRefresh() {}, getActiveCustomSmartAlbum: () => album,
    fileTypeOptions: ref([{value:0,label:'All'},{value:1,label:'Photos'},{value:4,label:'RAW'},{value:2,label:'Video'}]) };
  const api = new Function(...Object.keys(deps), ts.transpile(functions, {target: ts.ScriptTarget.ES2022})
    + ';return {toggleFavoritesFilter, handleFileTypeSelect, activeFavoritesOnly, fileTypeSummaryLabel};')(...Object.values(deps));
  return { ...deps, ...api, album };
}

test('Favorites only composes with media types, and All does not clear it', () => {
  const f = fixture();
  f.handleFileTypeSelect([1, 4]);
  f.toggleFavoritesFilter();
  assert.equal(f.config.search.fileType, 5);
  assert.equal(f.activeFavoritesOnly.value, true);
  assert.equal(f.fileTypeSummaryLabel.value, 'Photos + RAW');
  f.handleFileTypeSelect([0]);
  assert.equal(f.config.search.fileType, 0);
  assert.equal(f.activeFavoritesOnly.value, true);
  f.toggleFavoritesFilter();
  assert.equal(f.activeFavoritesOnly.value, false);
});

test('favorite filtering never edits saved smart album rules', () => {
  const f = fixture(true);
  const before = structuredClone(f.album);
  f.toggleFavoritesFilter();
  assert.deepEqual(f.album, before);
  assert.equal(f.activeFavoritesOnly.value, true);
});

test('fixed AI ordering allows Favorite filtering, while temporary fixed views remain disabled', () => {
  const f = fixture();
  f.isFixedAiResultView.value = true;
  f.toggleFavoritesFilter();
  assert.equal(f.activeFavoritesOnly.value, true);
  for (const mode of ['similar', 'person', 'album']) {
    f.tempViewMode.value = mode;
    f.toggleFavoritesFilter();
    assert.equal(f.config.search.favoritesOnly, true);
    assert.equal(f.activeFavoritesOnly.value, false);
  }
  f.tempViewMode.value = 'none';
  assert.equal(f.activeFavoritesOnly.value, true);
});

test('ordinary and collection query snapshots include the independent Favorite condition', async () => {
  const config = { search: { fileType: 5, favoritesOnly: true, sortType: 0, sortOrder: 0 }, settings: {} };
  const deps = { config, activeFavoritesOnly: ref(true), currentQueryParams: ref({}),
    currentQuerySource: ref('query'), currentCollectionId: ref(null), currentSmartQueryParams: ref(null),
    currentSearchFileIds: ref([]), createRandomSeed: () => 1, categorySortForGroup: () => 0,
    effectiveGroupBy: ref(0) };
  const queryStart = between('async function getFileList(', "  if (sourceContext?.source === 'smart') {");
  const collectionStart = between('async function getCollectionFileList(', '  isLoading.value = true;');
  const api = new Function(...Object.keys(deps), ts.transpile(queryStart + '}\n' + collectionStart + '}\n',
    { target: ts.ScriptTarget.ES2022 }) + ';return {getFileList, getCollectionFileList};')(...Object.values(deps));
  await api.getFileList({ searchFolder: '/photos' }, 1);
  assert.equal(deps.currentQueryParams.value.isFavorite, true);
  assert.equal(deps.currentQueryParams.value.searchFileType, 5);
  assert.equal(deps.currentQueryParams.value.searchFolder, '/photos');
  await api.getCollectionFileList(42, 1);
  assert.equal(deps.currentQuerySource.value, 'collection');
  assert.equal(deps.currentQueryParams.value.isFavorite, true);
  assert.equal(deps.currentQueryParams.value.searchFileType, 5);
});
