import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';
import { computed, markRaw, reactive, ref } from 'vue';

function moduleBody(path) {
  const source = fs.readFileSync(new URL(path, import.meta.url), 'utf8');
  return ts.transpile(source.replace(/^import[\s\S]*?from ['"][^'"]+['"];?\n/gm, '').replace(/export /g, ''), { target: ts.ScriptTarget.ES2022 });
}
const libConfig = { _libraryId: 'offline-test', album: { id: 1 }, activePane: 'album' };
const availability = new Function('reactive', 'libConfig', moduleBody('../src/common/availability.ts') + '; return { isOriginalUnavailable, setAlbumAccessibility, setFileAccessibility, setFolderAccessibility, resetFileAccessibility, resetLibraryAccessibility, requiresOriginalAction };')(reactive, libConfig);
const { isOriginalUnavailable, setAlbumAccessibility, setFileAccessibility, setFolderAccessibility, resetFileAccessibility, resetLibraryAccessibility, requiresOriginalAction } = availability;

test('offline state does not remove catalog data and follows reconnection, individual files and library scope', () => {
  const file = { id: 42, album_id: 1, album_accessible: true, file_path: '/photos/sub/image.jpg', rating: 5, thumbnail: 'cached' };
  setAlbumAccessibility(1, false);
  assert.equal(isOriginalUnavailable(file), true);
  assert.equal(file.thumbnail, 'cached');
  setAlbumAccessibility(1, true);
  setFolderAccessibility('/photos/sub', false);
  assert.equal(isOriginalUnavailable(file), true);
  assert.equal(isOriginalUnavailable({ ...file, file_path: '/photos/sub-other/image.jpg' }), false);
  setFolderAccessibility('/photos/sub', true);
  assert.equal(isOriginalUnavailable(file), false);
  setFileAccessibility(file.file_path, false);
  assert.equal(isOriginalUnavailable(file), true);
  libConfig._libraryId = 'other-library';
  assert.equal(isOriginalUnavailable(file), false);
  libConfig._libraryId = 'offline-test';
  resetFileAccessibility();
  assert.equal(isOriginalUnavailable(file), false);
});

const source = fs.readFileSync(new URL('../src/common/fileMenu.ts', import.meta.url), 'utf8');
const iconNames = source.match(/import \{([^}]+)\} from '@\/common\/icons'/)[1].split(',').map(s => s.trim()).filter(Boolean);
const config = { main: { sidebarIndex: 1 }, settings: { face: { enabled: true } }, externalAppsFor: () => [{ id: 'editor', name: 'Editor' }], defaultExternalApp: () => null };
const useFileMenuItems = new Function('computed', 'markRaw', 'config', 'libConfig', 'SIDEBAR', 'DEFAULT_PLATFORM', 'getShortcutLabel', 'isOriginalUnavailable', 'requiresOriginalAction', 'isWin', ...iconNames, moduleBody('../src/common/fileMenu.ts') + '; return useFileMenuItems;')(
  computed, markRaw, config, libConfig, { ALBUM: 1 }, '', () => '', isOriginalUnavailable, requiresOriginalAction, true, ...iconNames.map(() => ({})),
);
const words = () => new Proxy({}, { get: (_, key) => String(key) });
const locale = ref({ menu: { file: words(), meta: words() }, rating: words(), culling: words() });
const allItems = items => items.flatMap(item => [item, ...allItems(item.children || [])]);
const action = (items, name) => allItems(items).find(item => item.action?.actionName === name);

test('single-file menu retains catalog organization and blocks original-file operations while offline', () => {
  const file = ref({ id: 42, album_id: 1, album_accessible: false, file_type: 1, has_embedding: true, file_path: '/photos/offline.jpg' });
  setAlbumAccessibility(1, false);
  const called = [];
  const menu = useFileMenuItems(file, locale, true, key => key, name => called.push(name));
  for (const name of ['edit', 'print', 'rename', 'copy', 'move-within-library', 'move-to-folder', 'copy-to-folder', 'trash', 'reveal', 'refresh-file-info', 'set-desktop-wallpaper', 'open-external-app:editor']) {
    assert.equal(action(menu.value, name).disabled, true, name);
    action(menu.value, name).action();
  }
  assert.deepEqual(called, []);
  for (const name of ['favorite', 'rating-5', 'tag', 'comment', 'add-to-collection', 'rotate', 'open', 'set-album-cover']) {
    assert.equal(action(menu.value, name).disabled, false, name);
    action(menu.value, name).action();
  }
  assert.ok(called.includes('tag'));
  setAlbumAccessibility(1, true);
  assert.equal(action(menu.value, 'rename').disabled, false);
  action(menu.value, 'rename').action();
  assert.equal(called.at(-1), 'rename');
});

test('mixed availability blocks the entire selection rather than silently skipping offline files', () => {
  const unavailable = ref(true);
  const menu = useFileMenuItems(ref(null), locale, true, key => key, () => {}, {
    selectMode: ref(true), selectionMediaKind: ref('image'), selectionCount: ref(3), selectionHasUnavailable: unavailable,
  });
  assert.equal(action(menu.value, 'refresh-file-info').disabled, true);
  assert.equal(action(menu.value, 'create-montage').disabled, true);
  assert.equal(menu.value[2].disabled, true);
  assert.equal(action(menu.value, 'compare-selected-images').disabled, false);
  unavailable.value = false;
  assert.equal(action(menu.value, 'refresh-file-info').disabled, false);
});

const imageSource = fs.readFileSync(new URL('../src/components/Image.vue', import.meta.url), 'utf8');
const fallbackBody = ts.transpile(imageSource.slice(imageSource.indexOf('async function showOfflinePreview'), imageSource.indexOf('// Watch file changes and the selected RAW preview source.')), { target: ts.ScriptTarget.ES2022 });
function previewFixture({ backend = false, thumbnail = true } = {}) {
  const deps = {
    currentLoadingId: ref(1), loadingTimeout: null, isLoading: ref(true), rawRequestPending: ref(true),
    imageFilePath: ref(['/old.jpg', '']), imageSrc: ref(['old-image', '']), activeImage: ref(0),
    imageNaturalSize: ref([{ width: 512, height: 256 }, { width: 0, height: 0 }]),
    imageSize: ref([{ width: 512, height: 256 }, { width: 0, height: 0 }]), scale: ref([1, 1]),
    maxScale: ref(10), offlinePreview: ref(false), preloadCache: new Map(), props: { fileId: 42, fileVersion: 1 },
    getPreviewUrl: () => 'preview://localhost/test/42?v=1',
    getEffectiveThumbnailSrc: async () => thumbnail ? 'thumbnail' : '',
    loadPlaceholderResource: async src => {
      if (!src || (src.startsWith('preview:') && !backend)) throw Error('No cache');
      return { src, naturalWidth: 512, naturalHeight: 256 };
    },
    position: ref([{ x: 0, y: 0 }, { x: 0, y: 0 }]), isZoomFit: ref(false), loadError: ref(false),
    setImageSlot: (slot, path, src) => { deps.imageFilePath.value[slot] = path; deps.imageSrc.value[slot] = src; },
    onImageReady: slot => { deps.activeImage.value = slot; },
  };
  const showOfflinePreview = new Function('deps', 'const {' + Object.keys(deps).join(',') + '} = deps; ' + fallbackBody + '; return showOfflinePreview;')(deps);
  return { deps, showOfflinePreview };
}

test('offline preview prefers existing preview cache, then falls back to a thumbnail', async () => {
  const preview = previewFixture({ backend: true });
  await preview.showOfflinePreview('/offline.jpg', 1);
  assert.equal(preview.deps.offlinePreview.value, true);
  assert.ok(preview.deps.imageSrc.value[1].includes('cachedOnly=true'));
  assert.equal(preview.deps.isLoading.value, false);
  const thumbnail = previewFixture();
  await thumbnail.showOfflinePreview('/offline.jpg', 1);
  assert.equal(thumbnail.deps.offlinePreview.value, true);
  assert.equal(thumbnail.deps.imageSrc.value[1], 'thumbnail');
});

test('offline preview keeps already displayed pixels and clears previous-file pixels when no cache exists', async () => {
  const current = previewFixture();
  await current.showOfflinePreview('/old.jpg', 1);
  assert.equal(current.deps.imageSrc.value[0], 'old-image');
  assert.equal(current.deps.offlinePreview.value, true);
  const missing = previewFixture({ thumbnail: false });
  await missing.showOfflinePreview('/offline.jpg', 1);
  assert.equal(missing.deps.offlinePreview.value, true);
  assert.deepEqual(missing.deps.imageSrc.value, ['', '']);
  assert.equal(missing.deps.loadError.value, true);
});

test('a delayed offline preview cannot replace a more recently selected file', async () => {
  const fixture = previewFixture();
  let resolve;
  fixture.deps.preloadCache.set('/offline.jpg', new Promise(done => { resolve = done; }));
  const loading = fixture.showOfflinePreview('/offline.jpg', 1);
  fixture.deps.currentLoadingId.value = 2;
  resolve({ src: 'cached' });
  await loading;
  assert.deepEqual(fixture.deps.imageSrc.value, ['old-image', '']);
  assert.equal(fixture.deps.offlinePreview.value, false);
});

const contentSource = fs.readFileSync(new URL('../src/components/Content.vue', import.meta.url), 'utf8');
const dragBody = ts.transpile(contentSource.slice(contentSource.indexOf('function markContentInternalDrag('), contentSource.indexOf('const isProcessing = ref(false);')), { target: ts.ScriptTarget.ES2022 });
function dragFixture(collection = false) {
  const files = [
    { id: 1, album_id: 10, file_path: '/online.jpg', isSelected: true },
    { id: 2, album_id: 20, file_path: '/offline.jpg', isSelected: true },
  ];
  setAlbumAccessibility(10, true);
  setAlbumAccessibility(20, false);
  const calls = { started: 0, previews: 0, hydrated: 0, preflight: [], added: [], transfers: 0 };
  const deps = {
    fileList: ref([files[0]]), selectedCount: ref(2),
    document: { getElementById: () => ({ querySelector: () => null }), addEventListener: () => {}, documentElement: { addEventListener: () => {}, removeEventListener: () => {} } },
    MAX_NATIVE_DRAG_FILES: 1000, createDragPreview: () => { calls.previews++; return []; }, onContentWindowLeave: () => {},
    isRealFileItem: () => true, isContentInternalDrag: ref(false),
    getActionableSelectedItems: () => deps.fileList.value,
    getActionableSelectedItemsForAction: async () => { calls.hydrated++; return files; },
    isOriginalUnavailable, config,
    draggedFileIds: ref(new Set()), createDragGhost: () => { calls.started++; },
    tauriEmit: async () => {}, updateContentDragPosition: () => {},
    updateDragGhostModifier: () => {}, isCopyDragModifier: () => false,
    removeDragGhost: () => {},
    requireOriginalFiles: async selected => { calls.preflight.push(selected.map(file => file.id)); return !selected.some(isOriginalUnavailable); },
    confirmLargeBatch: async () => { calls.transfers++; return true; },
    addFilesToCollection: async (id, selected) => { calls.added.push([id, selected]); return { added: selected.length, skipped: 0 }; },
    t: key => key, toast: { success: () => {}, info: () => {} },
    libConfig: { activePane: 'album' },
  };
  const result = new Function('deps', 'const {' + Object.keys(deps).join(',') + '} = deps; let pointerDragUsesSelection = false; let pointerDragFiles = null; let nativeDragFiles = null; let nativeDragPreview = []; const pointerDropTarget = ' + JSON.stringify({ dataset: collection ? { collectionDropId: '7' } : { fileDropPath: '/destination', fileDropAlbumId: '10' } }) + '; ' + dragBody + '; return { markContentInternalDrag, clearContentInternalDrag };')(deps);
  return { ...result, deps, calls, files };
}

test('folder drops preflight the complete selection before transferring any files', async () => {
  const fixture = dragFixture();
  fixture.markContentInternalDrag({ event: {}, index: 0, hotspotXRatio: 0, hotspotYRatio: 0 });
  await fixture.clearContentInternalDrag({ type: 'pointerup' });
  assert.deepEqual(fixture.calls.preflight, [[1, 2]]);
  assert.equal(fixture.calls.transfers, 0);
});

test('offline files can still be dragged into a collection without original-file checks', async () => {
  const fixture = dragFixture(true);
  fixture.deps.fileList.value = [fixture.files[1]];
  fixture.markContentInternalDrag({ event: {}, index: 0, hotspotXRatio: 0, hotspotYRatio: 0 });
  assert.equal(fixture.calls.started, 1);
  await fixture.clearContentInternalDrag({ type: 'pointerup' });
  assert.deepEqual(fixture.calls.added, [[7, [1, 2]]]);
  assert.deepEqual(fixture.calls.preflight, []);
});

const dedupSource = fs.readFileSync(new URL('../src/components/DedupPane.vue', import.meta.url), 'utf8');
function extractFunction(source, name, next) {
  return ts.transpile(source.slice(source.indexOf(name), source.indexOf(next, source.indexOf(name))), { target: ts.ScriptTarget.ES2022 });
}

test('dedup detects missing files, skips offline albums and ignores responses from an old library', async () => {
  const files = [
    { file_path: '/dedup-missing.jpg', album_accessible: true },
    { file_path: '/dedup-offline.jpg', album_accessible: false },
  ];
  const checked = [];
  const states = [];
  const refresh = new Function('libConfig', 'checkFileAccessibility', 'setFileAccessibility',
    extractFunction(dedupSource, 'async function refreshFileAccess(', 'async function hydrateSimilarThumbnails') + '; return refreshFileAccess;'
  )(libConfig, async path => { checked.push(path); return false; }, (...args) => states.push(args));
  await refresh(files);
  assert.deepEqual(checked, ['/dedup-missing.jpg']);
  assert.deepEqual(states, [['/dedup-missing.jpg', false]]);
  const library = { _libraryId: 'first' };
  const staleStates = [];
  const staleRefresh = new Function('libConfig', 'checkFileAccessibility', 'setFileAccessibility',
    extractFunction(dedupSource, 'async function refreshFileAccess(', 'async function hydrateSimilarThumbnails') + '; return refreshFileAccess;'
  )(library, async () => { library._libraryId = 'second'; return false; }, (...args) => staleStates.push(args));
  await staleRefresh(files);
  assert.deepEqual(staleStates, []);
});

test('dedup delete-all blocks unavailable items and selected deletion is independent of the preview file', async () => {
  const calls = [];
  const pending = ref(false);
  const unavailable = ref(true);
  const trashAll = new Function('totalDuplicateFileCount', 'totalReclaimableBytes', 'duplicateAccessPending', 'hasUnavailableDuplicates', 'emit', 'libConfig', 'refreshFileAccess', 'rawGroups',
    extractFunction(dedupSource, 'async function trashAllDuplicates(', 'function applyDeletedFiles') + '; return trashAllDuplicates;'
  )(ref(2), ref(100), pending, unavailable, (...args) => calls.push(args), libConfig, async () => {}, ref([]));
  await trashAll();
  assert.equal(calls.length, 0);
  unavailable.value = false;
  pending.value = true;
  await trashAll();
  assert.equal(calls.length, 0);
  pending.value = false;
  await trashAll();
  assert.equal(calls.length, 1);
  const deps = {
    currentOriginalsUnavailable: () => true, selectMode: ref(false), selectedCount: ref(0),
    dedupReclaimBytes: ref(0), dedupTrashGroupKey: ref(''), dedupDeleteFileIds: ref([]),
    deletePermanently: ref(false), permanentDeleteChecked: ref(false), showTrashMsgbox: ref(false),
  };
  const open = new Function(...Object.keys(deps),
    extractFunction(contentSource, 'const openTrashMsgbox =', 'const openAllDuplicatesTrashMsgbox') + '; return openTrashMsgbox;'
  )(...Object.values(deps));
  open();
  assert.equal(deps.showTrashMsgbox.value, false);
  open(100, 'group', [42]);
  assert.equal(deps.showTrashMsgbox.value, true);
  assert.deepEqual(deps.dedupDeleteFileIds.value, [42]);
});

test('independent preview updates all visible panes on album recovery and ignores another library', async () => {
  const viewer = fs.readFileSync(new URL('../src/views/ImageViewer.vue', import.meta.url), 'utf8');
  const start = viewer.indexOf("async (event: any) => {", viewer.indexOf("listen('album-accessibility-changed'"));
  const handler = viewer.slice(start, viewer.indexOf('\n  });', start));
  const left = ref({ id: 1, album_id: 1, album_accessible: false });
  const right = ref({ id: 2, album_id: 1, album_accessible: false });
  const extras = { bottomLeft: { fileInfo: { id: 3, album_id: 1 } }, bottomRight: { fileInfo: { id: 4, album_id: 1 } } };
  const calls = [];
  const deps = {
    libConfig: { _libraryId: 'preview-library' },
    setAlbumAccessibility: (...args) => calls.push(args), resetFileAccessibility: () => {},
    getAvailablePanes: () => ['left', 'right', 'bottomLeft', 'bottomRight'],
    getFileInfoByPane: pane => pane === 'left' ? left.value : pane === 'right' ? right.value : extras[pane].fileInfo,
    getFileInfo: async id => ({ id, album_accessible: true }),
    fileInfo: left, rightFileInfo: right, extraPaneState: extras,
  };
  const sync = new Function(...Object.keys(deps), ts.transpile('const sync = ' + handler, { target: ts.ScriptTarget.ES2022 }) + '; return sync;')(...Object.values(deps));
  await sync({ payload: { albumId: 1, available: true, libraryId: 'other-library' } });
  assert.deepEqual(calls, []);
  assert.equal(left.value.album_accessible, false);
  await sync({ payload: { albumId: 1, available: true, libraryId: 'preview-library' } });
  assert.deepEqual(calls, [[1, true]]);
  for (const file of [left.value, right.value, extras.bottomLeft.fileInfo, extras.bottomRight.fileInfo]) {
    assert.equal(file.album_accessible, true);
  }
});

test('library reload clears stale album, folder and file availability when returning to a library', () => {
  const file = { album_id: 123, album_accessible: true, file_path: '/returning/sub/file.jpg' };
  setAlbumAccessibility(123, false);
  setFileAccessibility(file.file_path, false);
  setFolderAccessibility('/returning/sub', false);
  assert.equal(isOriginalUnavailable(file), true);
  const store = fs.readFileSync(new URL('../src/stores/libraryStore.js', import.meta.url), 'utf8');
  const start = store.indexOf('async reload() {');
  const reload = new Function('resetLibraryAccessibility', 'return ({' +
    store.slice(start, store.indexOf('\n    },', start) + 6) + '}).reload;')(resetLibraryAccessibility);
  return reload.call({ $reset() {}, async init() {} }).then(() => {
    assert.equal(isOriginalUnavailable(file), false);
  });
});

test('opening dedup checks only the active group rather than all duplicate files', async () => {
  const checked = [];
  const groups = Array.from({ length: 100 }, (_, index) => ({
    id: index + 1, items: [1, 2].map(id => ({ file: { id: index * 2 + id, file_path: '/file.jpg', thumbnail: 'cached' } })),
  }));
  const deps = {
    libConfig,
    refreshFileAccess: async files => checked.push(files.map(file => file.id)),
    loadedDuplicateGroupCount: ref(20), thumbnailPlaceholder: '', getFileThumb: () => { throw new Error('Cached thumbnail should be reused'); },
  };
  const hydrate = new Function(...Object.keys(deps),
    extractFunction(dedupSource, 'async function hydrateGroupThumbnails(', 'async function refreshOverview(') + '; return hydrateGroupThumbnails;'
  )(...Object.values(deps));
  await hydrate(groups, 50);
  assert.deepEqual(checked, [[99, 100]]);
});

test('album recovery preserves similar pagination and selection while refreshing the active list', async () => {
  const start = dedupSource.indexOf('async (event: any) => {', dedupSource.indexOf("listen('album-accessibility-changed'"));
  const handler = dedupSource.slice(start, dedupSource.indexOf('\n  });', start));
  const groups = ref([{ id: 50, items: [{ file: { id: 99, album_id: 1, album_accessible: false } }] }]);
  const selected = ref(50);
  const calls = [];
  const deps = {
    libConfig, rawGroups: ref([]), similarGroups: groups, activeTab: ref('similar'),
    selectedSimilarGroupId: selected, selectedGroupId: ref(1),
    hydrateSimilarThumbnails: async (list, id) => calls.push([list, id]),
    hydrateGroupThumbnails: () => { throw new Error('Inactive panel must not be reloaded'); },
  };
  const sync = new Function(...Object.keys(deps), ts.transpile('const sync = ' + handler, { target: ts.ScriptTarget.ES2022 }) + '; return sync;')(...Object.values(deps));
  const original = groups.value;
  await sync({ payload: { libraryId: libConfig._libraryId, albumId: 1, available: true } });
  assert.equal(groups.value, original);
  assert.equal(selected.value, 50);
  assert.equal(groups.value[0].items[0].file.album_accessible, true);
  assert.equal(calls[0][1], 50);
});

test('rechecking one album preserves unavailable state in unrelated albums', () => {
  const a = { album_id: 777, album_accessible: true, file_path: '/scope/a/image.jpg' };
  const b = { album_id: 778, album_accessible: true, file_path: '/scope/ab/image.jpg' };
  setFileAccessibility(a.file_path, false);
  setFolderAccessibility('/scope/ab', false);
  assert.equal(resetFileAccessibility('/scope/a/'), true);
  assert.equal(isOriginalUnavailable(a), false);
  assert.equal(isOriginalUnavailable(b), true);
  resetFileAccessibility();
});

test('offline preview keeps a finite zoom limit before image layout finishes', async () => {
  const fixture = previewFixture();
  fixture.deps.imageNaturalSize.value[0].width = 0;
  fixture.deps.imageSize.value[0].width = 0;
  await fixture.showOfflinePreview('/old.jpg', 1);
  assert.equal(Number.isFinite(fixture.deps.maxScale.value), true);
  assert.equal(fixture.deps.maxScale.value, 1);
});

test('album selection refresh is not duplicated by the accessibility event', () => {
  const start = contentSource.indexOf('(event: any) => {', contentSource.indexOf("listen('album-accessibility-changed'"));
  const handler = contentSource.slice(start, contentSource.indexOf('\n  });', start));
  let refreshes = 0;
  const deps = { libConfig, config, SIDEBAR: { ALBUM: 1 }, selectedItemIndex: ref(7),
    gridViewRef: ref({ getScrollTop: () => 320 }), pendingRestoreScrollTop: ref(null),
    updateContent: () => refreshes++ };
  const onAccess = new Function(...Object.keys(deps),
    'let pendingInitialSelectedIndex = -1; let hasRestoredInitialSelection = true; ' +
    ts.transpile('const handler = ' + handler, { target: ts.ScriptTarget.ES2022 }) + '; return handler;')(...Object.values(deps));
  onAccess({ payload: { libraryId: libConfig._libraryId, selectionChanged: true } });
  assert.equal(refreshes, 0);
  onAccess({ payload: { libraryId: libConfig._libraryId, selectionChanged: false } });
  assert.equal(refreshes, 1);
  onAccess({ payload: { libraryId: 'other-library', selectionChanged: false } });
  assert.equal(refreshes, 1);
  assert.equal(deps.pendingRestoreScrollTop.value, 320);
  onAccess({ payload: { libraryId: libConfig._libraryId, albumId: 2, selectionChanged: false } });
  assert.equal(refreshes, 1, 'recovering another album must not refresh the selected album');
});

test('oversized drag selection skips native preview and hydration while keeping internal dragging', () => {
  const fixture = dragFixture();
  fixture.deps.selectedCount.value = 1001;
  fixture.markContentInternalDrag({ event: {}, index: 0, hotspotXRatio: 0, hotspotYRatio: 0 });
  assert.equal(fixture.calls.previews, 0);
  assert.equal(fixture.calls.hydrated, 0);
  assert.equal(fixture.calls.started, 1);
  assert.equal(fixture.deps.isContentInternalDrag.value, true);
});


test('cloud indexing menu falls back to English when the locale has no new label', () => {
  const menu = useFileMenuItems(ref(null), ref({ menu: { file: {}, meta: {} } }), true, key => key, () => {}, {
    selectMode: ref(true), selectionCount: ref(1), selectionHasCloudOnly: ref(true),
  });
  assert.equal(action(menu.value, 'index-cloud-files').label, 'Download, index, then free up space');
  assert.equal(action(menu.value, 'index-cloud-files').disabled, false);
});

const warmupBody = ts.transpile(imageSource.slice(imageSource.indexOf('let warmImageRequest ='), imageSource.indexOf('function clearStalePreloadEntries')), { target: ts.ScriptTarget.ES2022 });
function warmupFixture(checkCloud) {
  const props = { filePath: '/current.jpg', nextFilePath: '/next.jpg', fileType: 1 };
  const scheduled = new Map();
  const reads = [];
  const fixture = new Function('props', 'isCloudOnlyFile', 'loadImageResource', 'shouldUseBackendPreview', 'setTimeout', 'clearTimeout', 'queueMicrotask', `
    let warmImageTimeout = null;
    let warmImageIdleId = null;
    ${warmupBody}
    return { warmImage, cancelWarmImageScheduling };
  `)(props, checkCloud, async path => reads.push(path), () => false,
    callback => { scheduled.set(1, callback); return 1; }, id => scheduled.delete(id), callback => callback());
  return { ...fixture, props, reads, flush: async () => {
    const callbacks = [...scheduled.values()]; scheduled.clear();
    for (const callback of callbacks) callback();
    await new Promise(resolve => setImmediate(resolve));
  } };
}

test('neighbor warmup reads local files but never reads cloud-only originals', async () => {
  for (const cloud of [true, false]) {
    const fixture = warmupFixture(async () => cloud);
    fixture.warmImage('/next.jpg');
    await fixture.flush();
    assert.deepEqual(fixture.reads, cloud ? [] : ['/next.jpg']);
  }
});

test('cancelled or superseded warmup cannot read an original after the attribute check resolves', async () => {
  for (const cancel of [true, false]) {
    let resolveCheck;
    const fixture = warmupFixture(() => new Promise(resolve => { resolveCheck = resolve; }));
    fixture.warmImage('/next.jpg');
    await fixture.flush();
    if (cancel) fixture.cancelWarmImageScheduling();
    else fixture.props.nextFilePath = '/another.jpg';
    resolveCheck(false);
    await fixture.flush();
    assert.deepEqual(fixture.reads, []);
  }
});
