import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import ts from 'typescript';

const read = path => readFileSync(new URL(path, import.meta.url), 'utf8');
const registry = {};
new Function('exports', ts.transpile(read('../src/common/shortcuts.ts'), {
  target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS,
}))(registry);
const { matchesShortcut, getShortcutLabel, getShortcutConflicts } = registry;
class HTMLElementStub {
  constructor(properties = {}) {
    this.tagName = 'DIV';
    this.isContentEditable = false;
    Object.assign(this, properties);
  }
}
const event = (extra = {}) => ({ key: 'a', code: 'KeyA', repeat: false,
  ctrlKey: false, metaKey: false, altKey: false, shiftKey: false, defaultPrevented: false,
  target: new HTMLElementStub(),
  preventDefault() { this.defaultPrevented = true; }, ...extra });

function hostHandler(viewer, mode, toggle, overrides = {}) {
  const source = read(viewer ? '../src/views/ImageViewer.vue' : '../src/components/Content.vue');
  const start = source.indexOf(viewer ? 'function handleKeyDown(event:' : 'function handleLocalKeyDown(event:');
  const end = source.indexOf(viewer ? "  if (matchesShortcut('meta.favorite'" : "  if (getActivePreviewMode() !== 'none' && matchesShortcut('view.zoomFit'", start);
  const deps = { HTMLElement: HTMLElementStub, matchesShortcut, shortcutPlatform: 'mac', uiStore: { inputStack: [], activePane: 'content' },
    isUpdatingRawGrouping: { value: false }, isSlideShow: { value: false }, splitCount: { value: 2 }, isMapVisible: { value: false },
    selectedItemIndex: { value: 0 }, fileList: { value: [{ file_type: 1 }] }, selectMode: { value: false },
    showQuickView: { value: mode === 'quick-view' }, getActivePreviewMode: () => mode,
    getActivePreviewMediaRef: () => ({ toggleAfAreas: toggle }),
    getActivePane: () => 'right', getViewerRef: pane => { assert.equal(pane, 'right'); return { toggleAfAreas: toggle }; },
    getMatchedViewBackground: () => null, getMatchedRating: () => null, getMatchedCulling: () => null,
    ...overrides };
  return new Function(...Object.keys(deps), ts.transpile(source.slice(start, end) + '\n}',
    { target: ts.ScriptTarget.ES2022 }) + `;return ${viewer ? 'handleKeyDown' : 'handleLocalKeyDown'};`)(...Object.values(deps));
}

test('A is available across preview contexts without conflicting with Select All or keyboard layouts', () => {
  for (const platform of ['mac', 'windows', 'linux']) {
    assert.equal(matchesShortcut('view.toggleAfAreas', event(), platform), true);
    assert.equal(getShortcutLabel('view.toggleAfAreas', platform), 'A');
    assert.equal(matchesShortcut('view.toggleAfAreas', event({ key: 'a', code: 'KeyQ' }), platform), true);
    assert.equal(matchesShortcut('view.toggleAfAreas', event({ key: 'q', code: 'KeyA' }), platform), false);
    for (const modifier of ['ctrlKey', 'metaKey', 'altKey', 'shiftKey']) {
      assert.equal(matchesShortcut('view.toggleAfAreas', event({ [modifier]: true }), platform), false);
    }
    for (const context of ['content', 'media-viewer', 'image-viewer']) {
      assert.equal(getShortcutConflicts(context, platform).some(c => c.actions.includes('view.toggleAfAreas')), false);
    }
  }
});

test('quick preview, Filmstrip, and independent viewer invoke the button action exactly once per key press', () => {
  for (const [viewer, mode] of [[false, 'quick-view'], [false, 'filmstrip'], [true, 'viewer']]) {
    let calls = 0;
    const handle = hostHandler(viewer, mode, () => { calls++; });
    handle(event());
    handle(event({ repeat: true }));
    assert.equal(calls, 1);
    for (const target of [{ tagName: 'INPUT' }, { tagName: 'TEXTAREA' }, { tagName: 'SELECT' }, { isContentEditable: true }]) {
      handle(event({ target: new HTMLElementStub(target) }));
    }
    assert.equal(calls, 1);
    handle(event({ defaultPrevented: true }));
    assert.equal(calls, 1);
  }
});

test('grid-only browsing, dialogs, and slideshow never toggle AF through A', () => {
  let calls = 0;
  const toggle = () => { calls++; };
  hostHandler(false, 'none', toggle)(event());
  for (const viewer of [false, true]) {
    hostHandler(viewer, 'quick-view', toggle, { uiStore: { inputStack: ['dialog'] } })(event());
    hostHandler(viewer, 'quick-view', toggle, { isSlideShow: { value: true } })(event());
  }
  assert.equal(calls, 0);
});

test('exposed button action respects media availability and arms feedback for the current image', () => {
  const source = read('../src/components/MediaViewer.vue');
  const start = source.indexOf('function toggleAfAreas()');
  const end = source.indexOf('const isHoverLeft =', start);
  const props = { fileIndex: 0, canInteract: true, file: { file_type: 1, file_path: '/a.jpg', modified_at: 10 } };
  const state = { value: false }, feedback = { value: null };
  const toggle = new Function('props', 'showAfAreas', 'afAreaFeedbackRequest', ts.transpile(source.slice(start, end),
    { target: ts.ScriptTarget.ES2022 }) + ';return toggleAfAreas;')(props, state, feedback);
  toggle();
  assert.equal(state.value, true);
  assert.deepEqual(feedback.value, { filePath: '/a.jpg', fileVersion: 10 });
  toggle();
  assert.equal(state.value, false);
  assert.equal(feedback.value, null);
  props.file.file_type = 2; toggle(); assert.equal(state.value, false);
  props.file.file_type = 3; props.canInteract = false; toggle(); assert.equal(state.value, false);
  props.canInteract = true; props.fileIndex = -1; toggle(); assert.equal(state.value, false);
});

test('the shared toolbar delegates AF toggling to the active image pane', () => {
  const source = read('../src/components/MediaViewer.vue');
  const start = source.indexOf('function toggleAfAreas()');
  const end = source.indexOf('const isHoverLeft =', start);
  const events = [];
  const state = { value: false }, feedback = { value: null };
  const toggle = new Function('props', 'showAfAreas', 'afAreaFeedbackRequest', 'emit', ts.transpile(source.slice(start, end),
    { target: ts.ScriptTarget.ES2022 }) + ';return toggleAfAreas;')(
    { toolbarOnly: true, fileIndex: 0, canInteract: true, file: { file_type: 1 } }, state, feedback, name => events.push(name));
  toggle();
  assert.deepEqual(events, ['toggle-af-areas']);
  assert.equal(state.value, false);
  assert.equal(feedback.value, null);
  const viewer = read('../src/views/ImageViewer.vue');
  const handler = viewer.match(/@toggle-af-areas="([^"]+)"/)[1];
  const calls = [];
  new Function('getViewerRef', 'activePane', handler)(pane => ({ toggleAfAreas: () => calls.push(pane) }), 'right');
  assert.deepEqual(calls, ['right']);
});
