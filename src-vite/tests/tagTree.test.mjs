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

test('inline tag filters keep ten recent tags in usage order and support assigned-only filtering', async () => {
  const panel = await readFile(new URL('../src/components/AssignTagsPanel.vue', import.meta.url), 'utf8');
  const start = panel.indexOf('const rows = computed');
  const end = panel.indexOf('const ROW_HEIGHT', start);
  const deps = { computed, tagRows, ancestorIds,
    tags: ref(Array.from({ length: 12 }, (_, i) => ({ id: i + 1, name: `Tag ${i + 1}`, path: `Tag ${i + 1}`, parent_id: null }))),
    selected: ref(new Set([12])), partial: ref(new Set([11])),
    recent: ref([12,11,10,9,8,7,6,5,4,3,2,1]), recentOnly: ref(true),
    showSelectedOnly: ref(false), search: ref(''), collapsed: ref([]) };
  const rows = new Function(...Object.keys(deps), ts.transpile(panel.slice(start, end), { target: ts.ScriptTarget.ES2022 }) + ';return rows;')(...Object.values(deps));
  assert.deepEqual(rows.value.map(t => t.id), [12,11,10,9,8,7,6,5,4,3]);
  deps.showSelectedOnly.value = true;
  assert.deepEqual(rows.value.map(t => t.id), [12,11]);
  deps.search.value = 'Tag 12';
  assert.deepEqual(rows.value.map(t => t.id), [12]);
});

test('inline assignment keeps its original file target and does not reload after switching photos', async () => {
  const panel = await readFile(new URL('../src/components/AssignTagsPanel.vue', import.meta.url), 'utf8');
  const start = panel.indexOf('async function change(');
  const end = panel.indexOf('watch(() => libConfig._libraryId', start);
  let finish;
  const calls = [], updates = [], events = [];
  const props = { fileIds: [10] };
  const deps = { props, libConfig: { _libraryId: 'library', tag: {id: 1} }, subtreeIds, tags: ref([]), busy: ref(false), loading: ref(false), failed: ref(false),
    recent: ref([]), updateTagCount: (...args) => updates.push(args), emit: (...args) => events.push(args),
    source: 'test', tauriEmit: () => {}, applyTagsToFiles: (...args) => { calls.push(args); return new Promise(resolve => { finish = resolve; }); } };
  const handlers = new Function(...Object.keys(deps), 'let version=1, disposed=false;' + ts.transpile(panel.slice(start, end), { target: ts.ScriptTarget.ES2022 }) + ';return {change, switchFile:()=>version++};')(...Object.values(deps));
  const saving = handlers.change(3, true);
  props.fileIds = [20]; handlers.switchFile();
  finish([{file_id: 10, has_tags: true}]);
  await saving;
  assert.deepEqual(calls, [[[10], [3], []]]);
  assert.deepEqual(updates, []);
  assert.equal(events[0][1][0].file_id, 10);
  assert.equal(deps.busy.value, false);
});

test('inline Enter never assigns a tag outside the filtered list', async () => {
  const panel = await readFile(new URL('../src/components/AssignTagsPanel.vue', import.meta.url), 'utf8');
  const start = panel.indexOf('function searchKey('), end = panel.indexOf('function toggleBranch(', start);
  const calls = [];
  const deps = { search: ref('Hidden'), rows: ref([{id: 3, name: 'Visible'}]), selected: ref(new Set()),
    searchInput: ref(null), change: (...args) => calls.push(args) };
  const handler = new Function(...Object.keys(deps), ts.transpile(panel.slice(start, end), {target: ts.ScriptTarget.ES2022}) + ';return searchKey;')(...Object.values(deps));
  handler({key:'Enter',preventDefault(){}});
  assert.deepEqual(calls, []);
  deps.rows.value = [{id:1,name:'Travel',path:'Travel'}, {id:2,name:'Paris',path:'Travel/Paris'}];
  deps.search.value = 'Travel/Paris';
  handler({key:'Enter',preventDefault(){}});
  assert.deepEqual(calls, [[2, true]]);
});

test('same-file tag invalidation keeps badges visible until replacement data arrives', async () => {
  const info = await readFile(new URL('../src/components/FileInfo.vue', import.meta.url), 'utf8');
  const start = info.indexOf('watch(() => [props.fileInfo?.id, props.fileInfo?.tags, libConfig._libraryId]');
  const end = info.indexOf('onMounted(async () => { const stop = await listen', start);
  let callback;
  const props = {fileInfo: {id:10, tags:[{id:3,name:'Travel'}]}};
  const displayTags = ref([]);
  new Function('watch','props','libConfig','displayTags','refreshTags', ts.transpile(info.slice(start,end), {target:ts.ScriptTarget.ES2022}))(
    (_, handler) => { callback = handler; }, props, {_libraryId:'library'}, displayTags, () => {});
  callback([10,props.fileInfo.tags,'library'], undefined);
  props.fileInfo.tags = undefined;
  callback([10,undefined,'library'], [10,[], 'library']);
  assert.equal(displayTags.value[0].id, 3);
  props.fileInfo.tags = [];
  callback([10,[],'library'], [10,undefined,'library']);
  assert.equal(displayTags.value.length, 0);
  displayTags.value = [{id:3}]; props.fileInfo = {id:20};
  callback([20,undefined,'library'], [10,[], 'library']);
  assert.equal(displayTags.value.length, 0);
});

test('switching photos reuses the tag catalog and only refreshes membership', async () => {
  const panel = await readFile(new URL('../src/components/AssignTagsPanel.vue', import.meta.url), 'utf8');
  const start = panel.indexOf('const catalogReady ='), end = panel.indexOf('async function change(', start);
  let reads = 0;
  const deps = {ref, props:{fileIds:[10]},libConfig:{_libraryId:'library'},loading:ref(false),failed:ref(false),tags:ref([]),selected:ref(new Set()),partial:ref(new Set()),locate:()=>{},
    getAllTags:async()=>{reads++;return [{id:3,name:'Travel'}];},
    getTagSelectionCounts:async(ids)=>ids[0]===10?[{tag_id:3,count:1}]:[]};
  const load = new Function(...Object.keys(deps), 'let version=0, disposed=false, pendingFocusTagId=0;' + ts.transpile(panel.slice(start,end),{target:ts.ScriptTarget.ES2022}) + ';return load;')(...Object.values(deps));
  await load(false);
  const catalog = deps.tags.value;
  assert.equal(deps.selected.value.has(3), true);
  deps.props.fileIds = [20];
  await load(false);
  assert.equal(reads, 1);
  assert.equal(deps.tags.value, catalog);
  assert.equal(deps.selected.value.size, 0);
});

test('inline tag additions refresh the active query and removals reuse its patch path', async () => {
  const source = await readFile(new URL('../src/components/Content.vue', import.meta.url), 'utf8');
  const start = source.indexOf('async function handleInlineTagsChanged('), end = source.indexOf('async function updateFileHasTags(', start);
  const notifications = [], removals = [];
  const context = {library:'library',source:'inline',tagId:7,branch:new Set([7]),add:true};
  const deps = {libConfig:{_libraryId:'library'},syncTagStates:async()=>{},isTagRemovalView:()=>true,
    tauriEmit:(...args)=>notifications.push(args),updateFileHasTags:async(...args)=>removals.push(args)};
  const handler = new Function(...Object.keys(deps),ts.transpile(source.slice(start,end),{target:ts.ScriptTarget.ES2022})+';return handleInlineTagsChanged;')(...Object.values(deps));
  await handler([],context);
  assert.deepEqual(notifications,[['tags-changed',{source:'inline',library:'library'}]]);
  await handler([],{...context,add:false});
  assert.equal(removals.length,1);
  assert.equal(removals[0][2],'inline');
  assert.equal(notifications.length,1);
});

test('a refresh overlapping a tag save reconciles membership after the write commits', async () => {
  const panel = await readFile(new URL('../src/components/AssignTagsPanel.vue', import.meta.url), 'utf8');
  const start = panel.indexOf('async function change('), end = panel.indexOf('watch(() => libConfig._libraryId',start);
  let finish;
  const refreshes = [], updates = [];
  const deps = {props:{fileIds:[10]},libConfig:{_libraryId:'library',tag:{id:1}},subtreeIds,tags:ref([]),busy:ref(false),loading:ref(false),failed:ref(false),recent:ref([]),source:'inline',
    updateTagCount:(...args)=>updates.push(args),emit:()=>{},load:async(catalog)=>refreshes.push(catalog),
    applyTagsToFiles:()=>new Promise(resolve=>{finish=resolve;})};
  const handlers = new Function(...Object.keys(deps),'let version=1,disposed=false;'+ts.transpile(panel.slice(start,end),{target:ts.ScriptTarget.ES2022})+';return {change,refresh:()=>version++};')(...Object.values(deps));
  const saving = handlers.change(3,true);
  handlers.refresh();
  finish([{file_id:10,has_tags:true}]);
  await saving;
  assert.deepEqual(updates,[]);
  assert.deepEqual(refreshes,[false]);
  assert.equal(deps.busy.value,false);
});

test('shared tag menu action respects selection mode and single-file mode', async () => {
  const content = await readFile(new URL('../src/components/Content.vue', import.meta.url), 'utf8');
  const start = content.indexOf('const clickTag = async'), end = content.indexOf('const clickAddToCollection', start);
  const action = content.match(/'tag': (.+),/)[1];
  const deps = { selectMode:ref(true), selectedCount:ref(3), selectedItemIndex:ref(0),
    fileList:ref([{id:101},{id:102},{id:103}]), fileIdsToTag:ref([]), showTaggingDialog:ref(false),
    getActionableSelectedItemsForAction:async()=>[{id:101},{id:102},{id:103}], confirmLargeBatch:async()=>true };
  const dispatch = new Function(...Object.keys(deps),ts.transpile(content.slice(start,end),{target:ts.ScriptTarget.ES2022})+`;return ${action};`)(...Object.values(deps));
  await dispatch();
  assert.deepEqual(deps.fileIdsToTag.value,[101,102,103]);
  assert.equal(deps.showTaggingDialog.value,true);
  deps.selectMode.value=false; deps.selectedItemIndex.value=1;
  await dispatch();
  assert.deepEqual(deps.fileIdsToTag.value,[102]);
  deps.selectMode.value=true; deps.selectedCount.value=0;
  deps.fileIdsToTag.value=[];deps.showTaggingDialog.value=false;
  await dispatch();
  assert.equal(deps.showTaggingDialog.value,false);
});

test('File Info tag navigation opens a temporary view with a tag filter', async () => {
  const content=await readFile(new URL('../src/components/Content.vue',import.meta.url),'utf8');
  const start=content.indexOf('function handleNavigateMetadata('),end=content.indexOf('async function removeFileFromCurrentCollection',start);
  const calls=[];
  const handler=new Function('enterMetadataTempView',ts.transpile(content.slice(start,end),{target:ts.ScriptTarget.ES2022})+';return handleNavigateMetadata;')((...args)=>calls.push(args));
  handler({type:'tag',tagId:7,name:'Travel/Paris'});
  assert.deepEqual(calls,[['tag','Travel/Paris',{tagId:7,searchFileType:0}]]);
  handler({type:'tag',tagId:0});
  assert.equal(calls.length,1);
});
