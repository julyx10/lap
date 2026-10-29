import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';

const source = fs.readFileSync(new URL('../src/common/nativeDrag.ts', import.meta.url), 'utf8');
function load(invoke = async () => {}, events = { listen: async () => () => {}, emit: async () => {} }) {
  const channels = [];
  class Channel { constructor() { channels.push(this); } }
  const js = ts.transpile(source.replace(/^import .*;$/gm, '').replace(/export /g, ''), { target: ts.ScriptTarget.ES2022 });
  const listeners = new Map();
  const window = { innerWidth: 1000, innerHeight: 700, addEventListener: (name, fn) => listeners.set(name, fn), removeEventListener: name => listeners.delete(name) };
  return { channels, listeners, ...new Function('invoke', 'Channel', 'window', 'listen', js + '; return { startNativeFileDrag, isNativeFileDragActive, isReturningNativeFileDrag, isWindowDragEdge };')(invoke, Channel, window, events.listen) };
}

async function waitForStart(api) {
  for (let i = 0; !api.channels.length && i < 10; i++) await Promise.resolve();
  assert.equal(api.channels.length, 1);
}

test('OS session remains active until drop or Escape cancellation, and rejects duplicate starts', async () => {
  for (const result of ['Dropped', 'Cancel']) {
    const calls = [];
    const api = load(async (...args) => { calls.push(args); });
    const paths = ['/photos/中文 #100%.jpg', '/photos/two.png'];
    const pending = api.startNativeFileDrag(paths, [1, 2]);
    await waitForStart(api);
    assert.equal(api.isNativeFileDragActive(), true);
    await api.startNativeFileDrag(['/other.jpg'], []);
    assert.equal(calls.length, 1);
    assert.equal(calls[0][0], 'start_file_drag');
    assert.deepEqual(calls[0][1].paths, paths);
    await waitForStart(api);
    api.channels[0].onmessage(result);
    await pending;
    assert.equal(api.isNativeFileDragActive(), false);
  }
});

test('native startup failure releases the session so another drag can start', async () => {
  const api = load(async () => { throw new Error('Original is unavailable'); });
  await assert.rejects(api.startNativeFileDrag(['/missing.jpg'], []), /unavailable/);
  assert.equal(api.isNativeFileDragActive(), false);
  await assert.rejects(api.startNativeFileDrag(['/missing.jpg'], []), /unavailable/);
  assert.equal(api.channels.length, 2);
});

test('release during backend preparation cancels even when the request ID arrives late', async () => {
  for (const releaseFirst of [false, true]) {
    const calls = [];
    const api = load(async (...args) => calls.push(args));
    const pending = api.startNativeFileDrag(['/slow-share/photo.jpg'], []);
    if (releaseFirst) api.listeners.get('pointerup')();
    await waitForStart(api);
    api.channels[0].onmessage('Preparing:request-123');
    assert.equal(api.isNativeFileDragActive(), true);
    if (!releaseFirst) api.listeners.get('pointerup')();
    assert.deepEqual(calls[1], ['cancel_file_drag', { requestId: 'request-123' }]);
    api.channels[0].onmessage('Cancel');
    await pending;
    assert.equal(api.listeners.size, 0);
    assert.equal(api.isNativeFileDragActive(), false);
  }
});

test('handoff region covers all window edges and excludes internal drop targets', () => {
  const api = load();
  for (const [clientX, clientY] of [[0, 350], [999, 350], [500, 0], [500, 699], [-2, 350]]) {
    assert.equal(api.isWindowDragEdge({ clientX, clientY }), true);
  }
  assert.equal(api.isWindowDragEdge({ clientX: 500, clientY: 350 }), false);
});

const content = fs.readFileSync(new URL('../src/components/Content.vue', import.meta.url), 'utf8');
const handoff = ts.transpile(content.slice(content.indexOf('async function handOffNativeDrag()'), content.indexOf('function updateContentDragPosition(')), { target: ts.ScriptTarget.ES2022 });
function loadHandoff(filesPromise, count = 2) {
  const listeners = new Map();
  const calls = [];
  const window = { addEventListener: (name, fn) => listeners.set(name, fn), removeEventListener: name => listeners.delete(name) };
  const start = new Function('pointerDragUsesSelection', 'selectedCount', 'MAX_NATIVE_DRAG_FILES', 'nativeDragFiles', 'nativeDragPreview', 'window', 'gridViewRef', 'clearContentInternalDrag', 'isOriginalUnavailable', 'toast', 't', 'startNativeFileDrag', handoff + '; return handOffNativeDrag;')(
    true, { value: count }, 1000, filesPromise, [42], window, { value: { cancelPointerDrag: () => calls.push('cancel-pointer') } },
    async (...args) => calls.push(['clear', args]), file => file.offline,
    { warning: message => calls.push(message === 'tooltip.drag_out.too_many' ? 'too-many' : 'offline'), error: () => calls.push('error') }, key => key,
    async paths => calls.push(['native', paths]),
  );
  return { start, calls, listeners };
}

test('handoff exports the complete hydrated selection and cleans internal drag without dropping', async () => {
  const api = loadHandoff(Promise.resolve([{ file_path: '/one.jpg' }, { file_path: '/offscreen.jpg' }]));
  await api.start();
  assert.deepEqual(api.calls, ['cancel-pointer', ['clear', []], ['native', ['/one.jpg', '/offscreen.jpg']]]);
  assert.equal(api.listeners.size, 0);
});

test('release during selection hydration cancels export and removes listeners', async () => {
  let finish;
  const api = loadHandoff(new Promise(resolve => { finish = resolve; }));
  const pending = api.start();
  api.listeners.get('pointerup')();
  finish([{ file_path: '/one.jpg' }]);
  await pending;
  assert.equal(api.calls.some(call => call[0] === 'native'), false);
  assert.equal(api.listeners.size, 0);
});

test('offline selection never starts a native export', async () => {
  const api = loadHandoff(Promise.resolve([{ file_path: '/offline.jpg', offline: true }]));
  await api.start();
  assert.equal(api.calls.includes('offline'), true);
  assert.equal(api.calls.some(call => call[0] === 'native'), false);
});

const imageSource = fs.readFileSync(new URL('../src/components/Image.vue', import.meta.url), 'utf8');
const imageHandlers = ts.transpile(
  imageSource.slice(imageSource.indexOf('let dragOutStart:'), imageSource.indexOf('const handleImageMouseMove ='))
  + imageSource.slice(imageSource.indexOf('const handleImageMouseUp ='), imageSource.indexOf('// mouse leave')),
  { target: ts.ScriptTarget.ES2022 },
);

test('viewer mouse drag exports the original, clears panning, and never navigates at handoff', async () => {
  const calls = [];
  const listeners = new Map();
  const document = { documentElement: { addEventListener() {}, removeEventListener() {} }, addEventListener: (name, fn) => listeners.set(name, fn), removeEventListener: name => listeners.delete(name) };
  const ref = value => ({ value });
  const isDraggingImage = ref(false);
  const api = new Function('document', 'props', 'isTouchActive', 'updatePosition', 'createDragPreview', 'isWindowDragEdge', 'startNativeFileDrag', 'toast', 't', 'emit', 'isDraggingImage', 'lastMousePosition', 'mouseDragNavDeltaX', 'mouseDragNavDeltaY', 'mouseDragNavTriggered', 'latestMouseEvent', 'navDirection', 'cancelAnimationFrame', 'MOUSE_DRAG_NAV_THRESHOLD', 'animationFrameId', 'updateDragPosition', imageHandlers + '; return { handleImageMouseDown, handleImageMouseUp };')(
    document, { filePath: '/original/raw.CR3', originalUnavailable: false, isSlideShow: false }, ref(false), () => {}, () => [42], event => event.clientX >= 984,
    async paths => calls.push(paths), { error: () => {} }, key => key, event => calls.push(event),
    isDraggingImage, ref({ x: 0, y: 0 }), ref(0), ref(0), ref(false), ref(null), ref(''), () => {}, 80, 0, () => { throw new Error('Native handoff must not flush swipe'); },
  );
  api.handleImageMouseDown({ button: 0, clientX: 300, clientY: 350, target: {}, preventDefault() {} });
  assert.equal(isDraggingImage.value, true);
  assert.equal(listeners.size, 2);
  listeners.get('mousemove')({ buttons: 1, clientX: 995, clientY: 350 });
  await Promise.resolve();
  assert.deepEqual(calls, [['/original/raw.CR3']]);
  assert.equal(isDraggingImage.value, false);
  assert.equal(listeners.size, 0);
});

test('quick viewer swipe flushes its final move and cannot mutate state after release', () => {
const handlers = ts.transpile(imageSource.slice(imageSource.indexOf('let dragOutStart:'), imageSource.indexOf('// Simple reset - clear all swipe state')), { target: ts.ScriptTarget.ES2022 });
const ref = value => ({ value });
let queuedFrame;
const emits = [];
const deltaX = ref(0), triggered = ref(false);
const context = {
  document: {documentElement:{addEventListener(){},removeEventListener(){}},addEventListener(){},removeEventListener(){}},
  props: {filePath:'/photo.jpg',originalUnavailable:false,isSlideShow:false},
  isTouchActive:ref(false),updatePosition(){},createDragPreview(){return [];},isWindowDragEdge(){return false;},startNativeFileDrag:async()=>{},toast:{error(){}},t:key=>key,
  emit:(...args)=>emits.push(args),isDraggingImage:ref(false),lastMousePosition:ref({x:0,y:0}),mouseDragNavDeltaX:deltaX,mouseDragNavDeltaY:ref(0),mouseDragNavTriggered:triggered,
  latestMouseEvent:ref(null),navDirection:ref(''),cancelAnimationFrame(){},requestAnimationFrame(fn){queuedFrame=fn;return 1;},MOUSE_DRAG_NAV_THRESHOLD:80,animationFrameId:0,
  mousePosition:ref({x:0,y:0}),containerSize:ref({width:1000,height:700}),activeImage:ref(0),scale:ref([1]),imageSizeRotated:ref([{width:600,height:400}]),isZoomFit:ref(true),position:ref([{x:0,y:0}]),clampPosition(){},
};
const api = new Function(...Object.keys(context), handlers + '; return { handleImageMouseDown, handleImageMouseMove, handleImageMouseUp };')(...Object.values(context));
api.handleImageMouseDown({button:0,clientX:300,clientY:350,target:{},preventDefault(){}});
api.handleImageMouseMove({buttons:1,clientX:450,clientY:350});
api.handleImageMouseUp();
queuedFrame();
assert.equal(emits.length, 1);
assert.equal(triggered.value, false);
assert.equal(deltaX.value, 0);
assert.equal(context.latestMouseEvent.value, null);
});

test('returning native drag stays rejected when native completion precedes DOM drop', async () => {
  for (const result of ['Dropped', 'Cancel']) {
    const api = load();
    const pending = api.startNativeFileDrag(['/photos/one.jpg'], []);
    assert.equal(api.isReturningNativeFileDrag({ type: 'dragenter' }), true);
    assert.equal(api.isReturningNativeFileDrag({ type: 'dragover' }), true);
    await waitForStart(api);
    api.channels[0].onmessage(result);
    await pending;
    assert.equal(api.isNativeFileDragActive(), false);
    assert.equal(api.isReturningNativeFileDrag({ type: 'drop' }), true);
    assert.equal(api.isReturningNativeFileDrag({ type: 'dragenter' }), false);
    assert.equal(api.isReturningNativeFileDrag({ type: 'drop' }), false);
  }
});

test('a later external drag is accepted after a native drag ends elsewhere', async () => {
  const api = load();
  const pending = api.startNativeFileDrag(['/photos/one.jpg'], []);
  api.isReturningNativeFileDrag({ type: 'dragenter' });
  await waitForStart(api);
  api.channels[0].onmessage('Dropped');
  await pending;
  assert.equal(api.isReturningNativeFileDrag({ type: 'dragenter' }), false);
  assert.equal(api.isReturningNativeFileDrag({ type: 'drop' }), false);
});

test('return drop prevents default and never reaches the importer', async () => {
  const start = content.indexOf('  domDrop = async (e: DragEvent) => {');
  const end = content.indexOf('  domDragEnd =', start);
  const handler = ts.transpile(content.slice(start, end), { target: ts.ScriptTarget.ES2022 });
  const calls = [];
  const drop = new Function('isReturningNativeFileDrag', 'clearDropOverlay', 'finishExternalDrag', 'let domDrop; ' + handler + '; return domDrop;')(
    () => true, () => calls.push('clear'), () => calls.push('finish-external-drag'),
  );
  await drop({ type: 'drop', preventDefault: () => calls.push('prevent-default') });
  assert.deepEqual(calls, ['finish-external-drag', 'prevent-default', 'clear']);
});

test('viewer drag is rejected by another WebView even when completion precedes its drop', async () => {
  for (const result of ['Dropped', 'Cancel']) {
    const listeners = new Set();
    const events = {
      listen: async (_name, callback) => { listeners.add(callback); return () => listeners.delete(callback); },
      emit: async (_name, payload) => { for (const listener of listeners) listener({ payload }); },
    };
    const viewer = load(async command => {
      if (command === 'start_file_drag') await events.emit('', true);
    }, events);
    const main = load(undefined, events);
    const pending = viewer.startNativeFileDrag(['/photos/one.jpg'], []);
    await waitForStart(viewer);
    assert.equal(main.isNativeFileDragActive(), true);
    assert.equal(main.isReturningNativeFileDrag({ type: 'dragenter' }), true);
    assert.equal(main.isReturningNativeFileDrag({ type: 'dragover' }), true);
    // Reject another start from a different window during the same native session.
    await main.startNativeFileDrag(['/photos/two.jpg'], []);
    assert.equal(main.channels.length, 0);
    await events.emit('', false); // Native completion broadcasts independently of the source channel.
    viewer.channels[0].onmessage(result);
    await pending;
    assert.equal(main.isNativeFileDragActive(), false);
    assert.equal(main.isReturningNativeFileDrag({ type: 'drop' }), true);
    assert.equal(main.isReturningNativeFileDrag({ type: 'dragenter' }), false);
    assert.equal(main.isReturningNativeFileDrag({ type: 'drop' }), false);
  }
});

test('failed native startup releases the shared state in all windows', async () => {
  const listeners = new Set();
  const events = {
    listen: async (_name, callback) => { listeners.add(callback); return () => listeners.delete(callback); },
    emit: async (_name, payload) => { for (const listener of listeners) listener({ payload }); },
  };
  const main = load(undefined, events);
  const viewer = load(async () => {
    await events.emit('', true);
    await events.emit('', false);
    throw new Error('Startup failed');
  }, events);
  await assert.rejects(viewer.startNativeFileDrag(['/missing.jpg'], []), /Startup failed/);
  assert.equal(main.isNativeFileDragActive(), false);
  assert.equal(main.isReturningNativeFileDrag({ type: 'dragenter' }), false);
});

test('source WebView can disappear before its channel completes without leaving other windows blocked', async () => {
  const listeners = new Set();
  const events = {
    listen: async (_name, callback) => { listeners.add(callback); return () => listeners.delete(callback); },
    emit: async (_name, payload) => { for (const listener of listeners) listener({ payload }); },
  };
  const main = load(undefined, events);
  const viewer = load(async () => { await events.emit('', true); }, events);
  void viewer.startNativeFileDrag(['/slow-share/photo.jpg'], []);
  await waitForStart(viewer);
  assert.equal(main.isNativeFileDragActive(), true);
  // Native window destruction clears the broadcast state; no source channel or finally executes.
  await events.emit('', false);
  assert.equal(main.isNativeFileDragActive(), false);
  assert.equal(main.isReturningNativeFileDrag({ type: 'dragenter' }), false);
});

test('native export accepts 1000 files and rejects 1001 before invoking the backend', async () => {
  const calls = [];
  const api = load(async (...args) => calls.push(args));
  await assert.rejects(api.startNativeFileDrag(Array(1001).fill('/photo.jpg'), []), /1000/);
  assert.equal(calls.length, 0);
  assert.equal(api.listeners.size, 0);
  assert.equal(api.isNativeFileDragActive(), false);
  const pending = api.startNativeFileDrag(Array(1000).fill('/photo.jpg'), []);
  await waitForStart(api);
  assert.equal(calls[0][1].paths.length, 1000);
  api.channels[0].onmessage('Dropped');
  await pending;
});

test('oversized selection is rejected as a whole without awaiting hydration', async () => {
  const api = loadHandoff(new Promise(() => {}), 1001);
  await api.start();
  assert.deepEqual(api.calls, ['cancel-pointer', ['clear', []], 'too-many']);
  assert.equal(api.listeners.size, 0);
});

test('starting at the window edge leaves no internal listeners after native handoff', async () => {
  const mark = ts.transpile(content.slice(content.indexOf('function markContentInternalDrag('), content.indexOf('async function clearContentInternalDrag(')), { target: ts.ScriptTarget.ES2022 });
  const listeners = new Map();
  const surface = {
    addEventListener: (name, callback) => listeners.set(name, callback),
    removeEventListener: name => listeners.delete(name),
  };
  const api = loadHandoff(Promise.resolve([{ file_path: '/one.jpg' }]));
  let pending;
  const deps = {
    fileList: { value: [{ id: 1, file_path: '/one.jpg', isSelected: true }] },
    selectedCount: { value: 1 }, MAX_NATIVE_DRAG_FILES: 1000,
    document: { ...surface, documentElement: surface, getElementById: () => ({ querySelector: () => null }) },
    isRealFileItem: () => true, isContentInternalDrag: { value: false },
    getActionableSelectedItems: () => [{ id: 1, file_path: '/one.jpg' }],
    getActionableSelectedItemsForAction: async () => [{ id: 1, file_path: '/one.jpg' }],
    draggedFileIds: { value: new Set() }, createDragPreview: () => [], createDragGhost() {}, tauriEmit: async () => {},
    onContentWindowLeave() {}, updateDragGhostModifier() {},
    updateContentDragPosition: event => {
      assert.equal(event.clientX, 0);
      // Native handoff synchronously cleans the internal listeners before awaiting files.
      for (const name of ['pointermove', 'mouseleave', 'keydown', 'keyup']) surface.removeEventListener(name);
      pending = api.start();
    },
  };
  const start = new Function('deps', 'const {' + Object.keys(deps).join(',') + '} = deps; let pointerDragUsesSelection = false; let pointerDragFiles = null; let nativeDragFiles = null; let nativeDragPreview = []; ' + mark + '; return markContentInternalDrag;')(deps);
  start({ event: { clientX: 0, buttons: 1 }, index: 0, hotspotXRatio: 0, hotspotYRatio: 0 });
  await pending;
  assert.equal(listeners.size, 0);
  assert.equal(api.listeners.size, 0);
  assert.deepEqual(api.calls, ['cancel-pointer', ['clear', []], ['native', ['/one.jpg']]]);
});
