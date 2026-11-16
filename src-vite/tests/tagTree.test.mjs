import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import ts from "typescript";
import { computed, ref } from "vue";

const treeSource = await readFile(
  new URL("../src/common/tagTree.ts", import.meta.url),
  "utf8",
);
const exports = {};
new Function(
  "exports",
  ts.transpile(treeSource, {
    target: ts.ScriptTarget.ES2022,
    module: ts.ModuleKind.CommonJS,
  }),
)(exports);
const { tagRows, ancestorIds, subtreeIds } = exports;
test("tag names reject delimiters and allow asterisks", () => {
  for (const name of ["", " ", "\t ", "a,b", "a;b", "a|b", "bad\0name", "a".repeat(256)])
    assert.equal(exports.isValidTagName(name), false, JSON.stringify(name));
  for (const name of [" tag", "tag ", "\ttag", "tag\t", "a/b", "a\\b", "two words", "a*b", "*tag", "tag*", "*", "旅行", "a".repeat(255)])
    assert.equal(exports.isValidTagName(name), true, JSON.stringify(name));
});
const tags = [
  { id: 1, name: "Places", parent_id: null, path: "Places", count: 2 },
  { id: 2, name: "France", parent_id: 1, path: "Places|France", count: 2 },
  { id: 3, name: "Paris", parent_id: 2, path: "Places|France|Paris", count: 1 },
  { id: 4, name: "Clients", parent_id: null, path: "Clients", count: 1 },
  { id: 5, name: "Paris", parent_id: 4, path: "Clients|Paris", count: 1 },
];

test("tree rows preserve depth, distinct paths and collapse boundaries", () => {
  assert.deepEqual(
    tagRows(tags).map((t) => [t.id, t.depth]),
    [
      [4, 0],
      [5, 1],
      [1, 0],
      [2, 1],
      [3, 2],
    ],
  );
  assert.deepEqual(
    tagRows(tags, "", [1]).map((t) => t.id),
    [4, 5, 1],
  );
  assert.deepEqual(ancestorIds(tags, 3), [2, 1]);
  assert.deepEqual([...subtreeIds(tags, 1)].sort(), [1, 2, 3]);
});

test("search reveals the matching paths and their ancestors without opening unrelated branches", () => {
  assert.deepEqual(
    tagRows(tags, "paris", [1, 4]).map((t) => t.id),
    [4, 5, 1, 2, 3],
  );
  assert.deepEqual(
    tagRows(tags, "france", [1]).map((t) => t.id),
    [1, 2, 3],
  );
  assert.deepEqual(tagRows(tags, "missing"), []);
  assert.deepEqual(
    tagRows(tags, "", [], 3)
      .filter((t) => t.depth === 0)
      .map((t) => t.id),
    [1, 4],
  );
});

test("new tag drafts stay last among siblings until saved", () => {
  for (const sort of [0, 1, 2, 3]) {
    for (const parent_id of [null, 1]) {
      const draft = { id: -1, name: "A draft", path: "A draft", parent_id, count: 999 };
      const rows = tagRows([...tags, draft], "", [], sort, -1);
      assert.equal(rows.filter(tag => tag.parent_id === parent_id).at(-1).id, -1);
    }
  }
  const saved = { id: 6, name: "A saved", path: "A saved", parent_id: null };
  assert.equal(tagRows([...tags, saved], "", [], 0, -1)[0].id, 6);
});

test("deep tag trees do not depend on the JavaScript recursion limit", () => {
  const deep = Array.from({ length: 2000 }, (_, index) => ({
    id: index + 1,
    parent_id: index || null,
    name: `node-${index}`,
    path: `node-${index}`,
  }));
  const rows = tagRows(deep);
  assert.equal(rows.length, 2000);
  assert.equal(rows.at(-1).depth, 1999);
  assert.equal(subtreeIds(deep, 1).size, 2000);
});

const dialog = await readFile(
  new URL("../src/components/TaggingDialog.vue", import.meta.url),
  "utf8",
);
const beginMarker = "// TAG_SELECTION_TEST_BEGIN";
const endMarker = "// TAG_SELECTION_TEST_END";
assert.equal(dialog.split(beginMarker).length, 2);
assert.equal(dialog.split(endMarker).length, 2);
const begin = dialog.indexOf(beginMarker);
const end = dialog.indexOf(endMarker);
assert.ok(begin >= 0 && end > begin);
const selection = ts.transpile(dialog.slice(begin, end), {
  target: ts.ScriptTarget.ES2022,
});
function fixture(initial = [], partial = []) {
  const ref = (value) => ({ value });
  const deps = {
    canEditTags: ref(true),
    editor: ref(null),
    selectedTags: ref(new Set(initial)),
    intermediateTags: ref(new Set(partial)),
    initialSelectedTags: ref(new Set(initial)),
    initialIntermediateTags: ref(new Set(partial)),
    tagChanges: ref(new Map()),
  };
  const toggle = new Function(
    ...Object.keys(deps),
    selection + "; return toggleTag;",
  )(...Object.values(deps));
  return { ...deps, toggle };
}

test("tag selection keeps tri-state batch semantics and never implicitly assigns ancestors", () => {
  const empty = fixture();
  empty.toggle(3);
  assert.deepEqual([...empty.selectedTags.value], [3]);
  assert.deepEqual([...empty.tagChanges.value], [[3, "add"]]);
  empty.toggle(3);
  assert.equal(empty.tagChanges.value.size, 0);
  const mixed = fixture([1], [3]);
  mixed.toggle(3);
  assert.equal(mixed.tagChanges.value.get(3), "add");
  mixed.toggle(3);
  assert.equal(mixed.tagChanges.value.get(3), "remove");
  mixed.toggle(3);
  assert.equal(mixed.tagChanges.value.size, 0);
  assert.ok(mixed.intermediateTags.value.has(3));
  mixed.toggle(1);
  assert.equal(mixed.tagChanges.value.get(1), "remove");
  mixed.toggle(1);
  assert.equal(mixed.tagChanges.value.size, 0);
});


test("virtual tag rows reveal offscreen targets and keep the inline editor at normal row height", () => {
  const start = dialog.indexOf('const TAG_ROW_HEIGHT =');
  const end = dialog.indexOf('let tagListObserver:', start);
  assert.ok(start >= 0 && end > start);
  const editor = ref(null);
  const filteredTags = ref(Array.from({ length: 4000 }, (_, i) => ({ id: i + 1 })));
  const code = ts.transpile(dialog.slice(start, end), { target: ts.ScriptTarget.ES2022 });
  const { tagGeometry, tagContentHeight, tagScroller, scrollToTag, editorTagIndex } = new Function(
    'ref', 'computed', 'editor', 'filteredTags', code + ';return { tagGeometry, tagContentHeight, tagScroller, scrollToTag, editorTagIndex };',
  )(ref, computed, editor, filteredTags);
  const viewport = { scrollTop: 0, clientHeight: 400 };
  const scrolls = [];
  tagScroller.value = { $el: viewport, scrollToPosition: value => { scrolls.push(value); viewport.scrollTop = value; } };
  scrollToTag(3999);
  assert.equal(viewport.scrollTop + viewport.clientHeight, tagContentHeight.value);
  scrollToTag(0);
  assert.equal(viewport.scrollTop, 0);
  scrollToTag(1);
  assert.equal(scrolls.length, 2, 'visible targets must not scroll');
  editor.value = { id: 4000 };
  scrollToTag(editorTagIndex.value);
  assert.equal(tagGeometry.value.at(-1).height, 40);
  assert.equal(viewport.scrollTop + viewport.clientHeight, tagContentHeight.value);
  filteredTags.value = [];
  scrollToTag(editorTagIndex.value);
  assert.equal(tagContentHeight.value, 0);
});


test("tag list measures width after the modal slot mounts and ignores a closed dialog", () => {
  const start = dialog.indexOf('  tagListObserver = new ResizeObserver(');
  const end = dialog.indexOf('  // Catalog and selection reads', start);
  assert.ok(start >= 0 && end > start);
  for (const closed of [false, true]) {
    const tagListContainer = { value: null };
    const tagListWidth = { value: 0 };
    const ticks = [], observed = [];
    new Function('ResizeObserver', 'nextTick', 'disposed', 'tagListContainer', 'tagListWidth',
      'let tagListObserver; ' + dialog.slice(start, end))(
      class { observe(element) { observed.push(element); } },
      callback => ticks.push(callback), closed, tagListContainer, tagListWidth,
    );
    assert.equal(observed.length, 0);
    tagListContainer.value = { clientWidth: 480 };
    ticks.forEach(callback => callback());
    assert.equal(tagListWidth.value, closed ? 0 : 464);
    assert.equal(observed.length, closed ? 0 : 1);
  }
});


test("virtual scroll keeps an offscreen editor mounted without forcing its scroll position", async () => {
  const source = await readFile(new URL('../src/components/VirtualScroll.vue', import.meta.url), 'utf8');
  const start = source.indexOf('const visibleItems = computed(');
  const end = source.indexOf('function getItemKey', start);
  const props = { items: Array.from({ length: 400 }, (_, id) => ({ id })), gridItems: 1, pinnedIndex: 399 };
  const range = ref({ start: 0, end: 12 });
  const rows = new Function('computed', 'props', 'visibleRange', 'totalCount',
    source.slice(start, end) + ';return visibleItems;')(computed, props, range, { value: 400 });
  assert.equal(rows.value.length, 13);
  assert.equal(rows.value.at(-1).index, 399);
  range.value = { start: 390, end: 400 };
  assert.equal(rows.value.length, 10, 'visible editor must not be duplicated');
  assert.equal(rows.value.filter(row => row.index === 399).length, 1);
  assert.doesNotMatch(dialog, /@scroll="keepEditorVisible"/);
});


test("selected-only tags retain unselected ancestors and combine with search", () => {
  const start = dialog.indexOf('const filteredTags = computed(');
  const end = dialog.indexOf('const editor = ref<', start);
  assert.ok(start >= 0 && end > start);
  const allTags = ref([
    { id: 1, name: 'Parent', parent_id: null, path: 'Parent' },
    { id: 2, name: 'Child', parent_id: 1, path: 'Parent|Child' },
    { id: 3, name: 'Other', parent_id: null, path: 'Other' },
  ]);
  const selectedTags = ref(new Set([2]));
  const intermediateTags = ref(new Set([3]));
  const showSelectedOnly = ref(true), tagSearch = ref(''), editor = ref(null);
  const deps = { computed, tagRows, allTags, selectedTags, intermediateTags, showSelectedOnly, tagSearch, editor, collapsed: ref([]) };
  const rows = new Function(...Object.keys(deps), ts.transpile(dialog.slice(start, end), { target: ts.ScriptTarget.ES2022 }) + ';return filteredTags;')(...Object.values(deps));
  assert.deepEqual(rows.value.map(tag => tag.id), [3, 1, 2]);
  assert.equal(rows.value.at(-1).depth, 1);
  assert.equal(selectedTags.value.has(1), false, 'ancestor is not automatically selected');
  assert.equal(rows.value.find(tag => tag.id === 1).hasChildren, true);
  tagSearch.value = 'Child';
  assert.deepEqual(rows.value.map(tag => tag.id), [1, 2]);
  selectedTags.value = new Set();
  assert.equal(rows.value.length, 0);
  editor.value = { id: 2, name: 'Child', parent: 1 };
  assert.deepEqual(rows.value.map(tag => tag.id), [1, 2], 'keep the editor mounted if its selection changes');
});

const content = await readFile(new URL('../src/components/Content.vue', import.meta.url), 'utf8');
const tagPanel = await readFile(new URL('../src/components/Tag.vue', import.meta.url), 'utf8');
const imageViewer = await readFile(new URL('../src/views/ImageViewer.vue', import.meta.url), 'utf8');

function removalFixture(getTagsForFile) {
  const original = [1, 2, 3].map(id => ({ id, size: 10 }));
  const events = [];
  const state = {
    isTagRemovalView: () => true, groupedModeActive: ref(false), isMapView: ref(false),
    isLoading: ref(false), tempViewMode: ref('none'), fileList: ref(original),
    isRealFileItem: file => !!file, getTagsForFile, getMediaItemSize: file => file.size,
    selectedItemIndex: ref(1), countedFolderRequestId: ref(0), totalFileCount: ref(3),
    totalFileSize: ref(30), selectMode: ref(false), refreshFileGeometryLayout() {},
    updateSelectedImage() {}, syncSelectionToImageViewer() {}, isPendingFileItem: () => false,
    markDedupSourceUpdated() {}, getCurrentQueryTimeLine: async () => [], timelineData: ref([]),
    showTaggingDialog: ref(true), syncTagStates: async () => {},
    tauriEmit: (...args) => events.push(args), gridViewRef: ref({ getScrollTop: () => 128 }),
  };
  const start = content.indexOf('async function updateFileHasTags');
  const end = content.indexOf('async function syncTagStates', start);
  const code = ts.transpile(content.slice(start, end), { target: ts.ScriptTarget.ES2022 });
  const run = new Function(...Object.keys(state), `
    let currentContentRequestId = 1, tagRemovalRestoreVersion = 0, visibleRangeSeqId = 0;
    let pendingTagRemovalAnchor = { index: 999 }, pendingScopeSelectionChange = false, contentQueryRefreshPending = false;
    let lastVisibleRange = {start: -1, end: -1}, suppressImageViewerSync = false;
    ${code}
    return { updateFileHasTags, anchor: () => pendingTagRemovalAnchor,
      beginRefresh: () => { contentQueryRefreshPending = true; } };
  `)(...Object.values(state));
  return { ...state, ...run, events };
}

const removal = { library: 'test', tagId: 10, branch: new Set([10, 11]) };
test('tag removal patches the current list and focuses the next photo', async () => {
  const f = removalFixture(async () => [{ id: 20 }]);
  await f.updateFileHasTags([{ file_id: 2, has_tags: true }], removal);
  assert.deepEqual(f.fileList.value.map(file => file.id), [1, 3]);
  assert.equal(f.fileList.value[f.selectedItemIndex.value].id, 3);
  assert.equal(f.totalFileCount.value, 2);
  assert.equal(f.anchor(), null);
  assert.equal(f.events[0][1].patchedTagId, 10);
});

test('removing another tag or retaining a descendant keeps the photo in the list', async () => {
  for (const id of [10, 11]) {
    const f = removalFixture(async () => [{ id }]);
    await f.updateFileHasTags([{ file_id: 2, has_tags: true }], removal);
    assert.deepEqual(f.fileList.value.map(file => file.id), [1, 2, 3]);
    assert.equal(f.events[0][1].patchedTagId, 10);
  }
});

test('missing rows or large batches fall back with a fresh anchor and no per-file reads', async () => {
  const f = removalFixture(async () => assert.fail('unexpected membership lookup'));
  await f.updateFileHasTags([{ file_id: 99, has_tags: true }], removal);
  assert.equal(f.events[0][1], undefined);
  assert.deepEqual(f.anchor().ids, [2, 3, 1]);
  assert.equal(f.anchor().scrollTop, 128);
  f.fileList.value = Array.from({ length: 25 }, (_, index) => ({ id: index + 1 }));
  await f.updateFileHasTags(f.fileList.value.map(file => ({ file_id: file.id, has_tags: true })), removal);
  assert.equal(f.events[1][1], undefined);
  assert.ok(f.anchor());
  await f.updateFileHasTags([], null);
  assert.equal(f.anchor(), null);
});

test('a query refresh starting during membership lookup blocks a stale patch and anchor', async () => {
  let release;
  const f = removalFixture(() => new Promise(resolve => { release = resolve; }));
  const pending = f.updateFileHasTags([{ file_id: 2, has_tags: true }], removal);
  await Promise.resolve();
  f.beginRefresh();
  release([]);
  await pending;
  assert.equal(f.fileList.value.length, 3);
  assert.equal(f.anchor(), null);
  assert.equal(f.events[0][1], undefined);
});

test('tag sidebar reloads counts but skips query activation for a matching patched view', async () => {
  const start = tagPanel.indexOf('      if (event.payload?.source === tagsChangeSource)');
  const end = tagPanel.indexOf('\n    },', start);
  const config = { main: { sidebarIndex: 7 } };
  const libConfig = { _libraryId: 'test', tag: { id: 10, activateTick: 0 } };
  let loads = 0;
  const run = new Function('config', 'libConfig', 'load', `
    const tagsChangeSource = 'sidebar', disposed = false, SIDEBAR = { TAG: 7 };
    return async event => { ${tagPanel.slice(start, end)} };
  `)(config, libConfig, async () => { loads++; });
  await run({ payload: { library: 'test', patchedTagId: 10 } });
  assert.equal(loads, 1);
  assert.equal(libConfig.tag.activateTick, 0);
  await run({ payload: { library: 'test', patchedTagId: 11 } });
  assert.equal(libConfig.tag.activateTick, 1);
});

test('ImageViewer tag edits still notify the main view to refresh', async () => {
  const start = imageViewer.indexOf('async function updateFileHasTags');
  const end = imageViewer.indexOf('async function syncTagStates', start);
  const events = [];
  const visible = ref(true);
  const handler = new Function('syncTagStates', 'showTaggingDialog', 'emit',
    ts.transpile(imageViewer.slice(start, end)) + ';return updateFileHasTags;'
  )(async () => {}, visible, event => events.push(event));
  await handler([]);
  assert.equal(visible.value, false);
  assert.deepEqual(events, ['tags-changed']);
});

test('fallback restoration skips removed photos and selects the next surviving candidate', async () => {
  const start = content.indexOf('async function restoreTagRemovalAnchor');
  const end = content.indexOf('watch([fileList, contentReady]', start);
  const selected = [];
  const deps = {
    contentCountIsAuthoritative: ref(true),
    config: { main: { sidebarIndex: 7 } }, SIDEBAR: { TAG: 7 },
    libConfig: { _libraryId: 'test', tag: { id: 10 } },
    currentQueryParams: ref({ tagId: 10 }), fileList: ref([{}, {}]),
    groupedModeActive: ref(false),
    resolveRefreshedFileIndex: async id => id === 3 ? 1 : -1,
    restoreRefreshedFileSelection: async (index, current, scrollTop) => {
      if (current()) selected.push([index, scrollTop]);
    },
    fetchDataRange() {},
  };
  const restore = new Function(...Object.keys(deps), `
    let pendingTagRemovalAnchor = { request: 1, library: 'test', tagId: 10, index: 0, ids: [2, 3, 1], scrollTop: 128 };
    const currentContentRequestId = 2, tagRemovalRestoreVersion = 1;
    const pendingScopeSelectionChange = false, contentQueryRefreshPending = false;
    ${content.slice(start, end)}
    return restoreTagRemovalAnchor;
  `)(...Object.values(deps));
  await restore();
  assert.deepEqual(selected, [[1, 128]]);
});

test('restoring the same index refreshes the photo and syncs ImageViewer', async () => {
  const start = content.indexOf('async function restoreRefreshedFileSelection');
  const end = content.indexOf('async function restoreFocusedFileAfterListRefresh', start);
  const selectedItemIndex = ref(1);
  const calls = [];
  const deps = {
    selectedItemIndex, nextTick: async () => {}, groupedModeActive: ref(false),
    gridViewRef: ref(null), suppressImageViewerSync: false,
    updateSelectedImage: index => calls.push(['photo', index]),
    syncSelectionToImageViewer: index => calls.push(['viewer', index]),
  };
  const restore = new Function(...Object.keys(deps),
    ts.transpile(content.slice(start, end)) + ';return restoreRefreshedFileSelection;'
  )(...Object.values(deps));
  await restore(0, () => false);
  assert.equal(selectedItemIndex.value, 1);
  assert.deepEqual(calls, []);
  await restore(1, () => true);
  assert.deepEqual(calls, [['photo', 1], ['viewer', 1]]);
});
