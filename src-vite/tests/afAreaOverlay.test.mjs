import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import ts from 'typescript';
import * as Vue from 'vue';
import { compileTemplate } from '@vue/compiler-sfc';
import { reactive, ref, watch, nextTick, effectScope } from 'vue';

const source = readFileSync(new URL('../src/components/Image.vue', import.meta.url), 'utf8');
const code = ts.transpile(source.slice(source.indexOf('type AfArea ='), source.indexOf('const uiStore = useUIStore();')),
  { target: ts.ScriptTarget.ES2022 });
function fixture(enabled = true) {
  const props = reactive({ showAfAreas: enabled, filePath: '/a.nef', fileVersion: 0, originalUnavailable: false, afAreaFeedbackRequest: null });
  let timer, cleanup;
  const requests = [];
  const notices = [];
  const scope = effectScope();
  const state = scope.run(() => new Function('props', 'ref', 'watch', 'setTimeout', 'clearTimeout', 'onBeforeUnmount', 'invoke', 'toast', 't',
    code + ';return {afAreas};')(props, ref, watch,
    fn => { timer = fn; return 1; }, () => { timer = null; }, fn => { cleanup = fn; },
    (_command, payload) => new Promise(resolve => requests.push({ ...payload, resolve })),
    { info: message => notices.push(message) }, key => key));
  return { ...state, props, requests, notices,
    fire() { const fn = timer; timer = null; fn?.(); },
    stop() { cleanup(); scope.stop(); },
  };
}
const settle = async () => { await Promise.resolve(); await nextTick(); };

test('AF parsing stays opt-in and debounces rapid image navigation', async () => {
  const f = fixture(false);
  f.fire(); assert.equal(f.requests.length, 0);
  f.props.showAfAreas = true;
  f.props.filePath = '/b.nef';
  await nextTick();
  f.props.filePath = '/c.nef';
  await nextTick();
  f.fire();
  assert.deepEqual(f.requests.map(r => r.filePath), ['/c.nef']);
  f.stop();
});

test('a slow response cannot overwrite the next image, and only the latest image is queued', async () => {
  const f = fixture(); f.fire();
  f.props.filePath = '/b.nef'; await nextTick(); f.fire();
  f.props.filePath = '/c.nef'; await nextTick(); f.fire();
  assert.equal(f.requests.length, 1);
  f.requests[0].resolve([{ focused: true }]); await settle();
  assert.deepEqual(f.afAreas.value, []);
  assert.deepEqual(f.requests.map(r => r.filePath), ['/a.nef', '/c.nef']);
  f.requests[1].resolve([{ focused: null }]); await settle();
  assert.deepEqual(f.afAreas.value, [{ focused: null }]);
  f.stop();
});

test('disabling, losing original access, or unmounting discards pending AF results', async () => {
  for (const action of ['disable', 'offline', 'unmount']) {
    const f = fixture(); f.fire();
    f.props.filePath = '/b.nef'; await nextTick(); f.fire();
    if (action === 'disable') f.props.showAfAreas = false;
    if (action === 'offline') f.props.originalUnavailable = true;
    if (action === 'unmount') f.stop();
    await nextTick();
    f.requests[0].resolve([{ focused: true }]); await settle();
    assert.deepEqual(f.afAreas.value, []);
    assert.equal(f.requests.length, 1);
    if (action !== 'unmount') f.stop();
  }
});

const imageTemplate = source.slice(source.indexOf('    <TransitionGroup'), source.indexOf('    <!-- Faces Overlay -->'));
const compiledTemplate = compileTemplate({ source: imageTemplate, filename: 'Image.vue', id: 'af-overlay-test',
  compilerOptions: { mode: 'function', prefixIdentifiers: true, cacheHandlers: false, hoistStatic: false } });
assert.deepEqual(compiledTemplate.errors, []);
const renderImages = new Function('Vue', compiledTemplate.code)({ ...Vue,
  withCtx: fn => fn,
  withDirectives: vnode => vnode, // No DOM is needed to inspect the actual rendered image/overlay hierarchy.
});
function descendants(vnode) {
  if (!vnode || typeof vnode !== 'object') return [];
  return [vnode, ...(Array.isArray(vnode.children) ? vnode.children.flatMap(descendants) : [])];
}
function renderBuffers(overrides = {}) {
  const ctx = { imageSrc: ['a.jpg', 'b.jpg'], imageFilePath: ['/a.nef', '/b.nef'], activeImage: 0,
    filePath: '/a.nef', showAfAreas: true, loadError: false, scale: [2, 4],
    isGrabbing: false, isDraggingImage: false,
    afAreas: [{ x: 0.1, y: 0.2, width: 0.3, height: 0.4, focused: null }],
    getImageStyle: index => ({ transform: `translate(${index * 100}px, 0) scale(${index ? 4 : 2}) rotate(90deg)` }),
    ...overrides };
  const transition = renderImages(ctx, []);
  const wrappers = transition.children.default().flatMap(descendants)
    .filter(node => node.props?.class?.includes('slide-wrapper'));
  return { transition, wrappers };
}

test('AF overlay inherits slide and fade transitions from its own image buffer', () => {
  for (const transitionName of ['slide-next', 'slide-prev', 'slideshow-fade']) {
    for (const activeImage of [0, 1]) {
      const { transition, wrappers } = renderBuffers({ transitionName, activeImage, filePath: activeImage ? '/b.nef' : '/a.nef' });
      assert.equal(transition.props.name, transitionName);
      assert.equal(wrappers.length, 2);
      for (const [index, wrapper] of wrappers.entries()) {
        const overlays = descendants(wrapper).filter(node => node.props?.['aria-hidden'] === 'true');
        assert.equal(overlays.length, index === activeImage ? 1 : 0);
        if (overlays.length) {
          const nodes = descendants(overlays[0]);
          const layout = nodes.find(node => node.props?.style?.transform);
          const image = wrapper.children.find(node => node.type === 'img');
          assert.equal(layout.props.style.transform, image.props.style.transform);
          const box = nodes.find(node => node.props?.class === 'absolute box-border');
          assert.equal(box.props.style.border, `${2 / (index ? 4 : 2)}px solid #94a3b8`);
        }
      }
    }
  }
});

test('an image buffer cannot display AF areas for a different file or after the toggle is disabled', () => {
  for (const overrides of [{ filePath: '/c.nef' }, { showAfAreas: false }, { loadError: true }]) {
    const { wrappers } = renderBuffers(overrides);
    assert.equal(wrappers.flatMap(descendants).filter(node => node.props?.['aria-hidden'] === 'true').length, 0);
  }
});

test('an explicit enable reports empty AF data once, while image navigation stays silent', async () => {
  const f = fixture(false);
  f.props.afAreaFeedbackRequest = { filePath: '/a.nef', fileVersion: 0 };
  f.props.showAfAreas = true;
  await nextTick(); f.fire();
  f.requests[0].resolve([]); await settle();
  assert.deepEqual(f.notices, ['image_viewer.af_unavailable']);
  f.props.filePath = '/b.nef'; await nextTick(); f.fire();
  f.requests[1].resolve([]); await settle();
  f.props.filePath = '/a.nef'; await nextTick(); f.fire();
  f.requests[2].resolve([]); await settle();
  assert.equal(f.notices.length, 1);
  f.stop();
});

test('persisted state and other panes never show the no-data toast', async () => {
  const f = fixture(true); f.fire();
  f.requests[0].resolve([]); await settle();
  assert.deepEqual(f.notices, []);
  f.stop();
});

test('switching files or disabling AF during an explicit check cancels its toast', async () => {
  for (const action of ['navigate', 'disable', 'unmount']) {
    const f = fixture(false);
    f.props.afAreaFeedbackRequest = { filePath: '/a.nef', fileVersion: 0 };
    f.props.showAfAreas = true;
    await nextTick(); f.fire();
    if (action === 'navigate') f.props.filePath = '/b.nef';
    if (action === 'disable') f.props.showAfAreas = false;
    if (action === 'unmount') f.stop();
    await nextTick();
    f.requests[0].resolve([]); await settle();
    assert.deepEqual(f.notices, []);
    if (action !== 'unmount') f.stop();
  }
});

test('navigation before the debounce or a file version change does not notify for the replacement image', async () => {
  for (const action of ['navigate', 'version']) {
    const f = fixture(false);
    f.props.afAreaFeedbackRequest = { filePath: '/a.nef', fileVersion: 0 };
    f.props.showAfAreas = true;
    if (action === 'navigate') f.props.filePath = '/b.nef';
    else f.props.fileVersion = 1;
    await nextTick(); f.fire();
    f.requests[0].resolve([]); await settle();
    assert.deepEqual(f.notices, []);
    f.stop();
  }
});

test('a fresh explicit request can notify again after toggling off and on in one update', async () => {
  const f = fixture(false);
  f.props.afAreaFeedbackRequest = { filePath: '/a.nef', fileVersion: 0 };
  f.props.showAfAreas = true;
  await nextTick(); f.fire();
  f.requests[0].resolve([{ focused: true }]); await settle();
  assert.equal(f.notices.length, 0);
  f.props.showAfAreas = false;
  f.props.afAreaFeedbackRequest = null;
  f.props.showAfAreas = true;
  f.props.afAreaFeedbackRequest = { filePath: '/a.nef', fileVersion: 0 };
  await nextTick(); f.fire();
  f.requests[1].resolve([]); await settle();
  assert.equal(f.notices.length, 1);
  f.stop();
});
