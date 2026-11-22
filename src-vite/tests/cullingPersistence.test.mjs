import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';

const apiSource = fs.readFileSync(new URL('../src/common/api.js', import.meta.url), 'utf8');
const fnStart = apiSource.indexOf('export async function setFileCullingFlag');
const fnEnd = apiSource.indexOf('\n}\n', fnStart) + 3;
const apiSnippet = apiSource.slice(fnStart, fnEnd).replace('export async function ', 'async function ');

function createApiSetCullingFlag(invokeImpl) {
  return new Function('invoke', `${apiSnippet}; return setFileCullingFlag;`)(invokeImpl);
}

const syncSource = fs.readFileSync(new URL('../src/common/cullingSync.ts', import.meta.url), 'utf8');
const syncJs = ts.transpile(
  syncSource.replace(/export /g, ''),
  { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext },
);
const {
  createCullingSyncTracker,
  getMatchingPaneFileInfo,
} = new Function(`${syncJs}; return { createCullingSyncTracker, getMatchingPaneFileInfo };`)();

test('setFileCullingFlag rejects invalid/placeholder IDs and treats 0 updated rows as failure', async () => {
  const invoked = [];
  let nextResult = 1;
  const setFileCullingFlag = createApiSetCullingFlag(async (cmd, args) => {
    invoked.push([cmd, args]);
    if (nextResult instanceof Error) throw nextResult;
    return nextResult;
  });

  assert.equal(await setFileCullingFlag('ph-group-1', 2), null);
  assert.equal(await setFileCullingFlag(0, 2), null);
  assert.equal(await setFileCullingFlag(-3, 2), null);
  assert.equal(invoked.length, 0);

  nextResult = 0;
  assert.equal(await setFileCullingFlag(10, 2), null);

  nextResult = new Error('db error');
  assert.equal(await setFileCullingFlag(10, 2), null);

  nextResult = 1;
  assert.equal(await setFileCullingFlag(10, 2), 1);
  assert.equal(invoked.length, 3);
});

test('cullingSync preserves reject pressed before getFileInfo resolves and ignores stale loads', async () => {
  const sync = createCullingSyncTracker();
  let activeFileId = 100;
  let leftFileInfo = { id: 100, culling_flag: 0, rotate: 0 };
  const synced = [];
  const toasts = [];
  const pendingGets = new Map();

  const getFileInfo = (id) => new Promise((resolve) => {
    pendingGets.set(id, resolve);
  });

  async function loadLeft(targetId) {
    const req = sync.beginFileInfoLoad('left');
    if (leftFileInfo && Number(leftFileInfo.id) !== targetId) {
      leftFileInfo = null;
    }
    const loaded = await getFileInfo(targetId);
    if (!sync.isCurrentFileInfoLoad('left', req, activeFileId, targetId)) {
      return;
    }
    leftFileInfo = sync.applyPendingCulling(loaded, req);
  }

  async function rejectCurrent(persistFn) {
    const currentId = activeFileId;
    const matching = getMatchingPaneFileInfo(leftFileInfo, currentId);
    const knownPrev = matching ? Number(matching.culling_flag ?? 0) : null;
    const { seq, normalized } = sync.beginCullingUpdate(currentId, 2, knownPrev);
    if (matching) matching.culling_flag = normalized;

    const result = await persistFn(currentId, normalized);
    const finished = sync.finishCullingUpdate(currentId, seq, result !== null);
    if (!finished.isLatest) return;
    if (result === null) {
      const curr = getMatchingPaneFileInfo(leftFileInfo, currentId);
      if (curr && finished.previous !== null) curr.culling_flag = finished.previous;
      toasts.push('error');
      return;
    }
    const curr = getMatchingPaneFileInfo(leftFileInfo, currentId);
    if (curr) curr.culling_flag = normalized;
    synced.push([currentId, normalized]);
  }

  // Navigate to 100 (slow), then immediately to 101
  const load100 = loadLeft(100);
  activeFileId = 101;
  const load101 = loadLeft(101);

  // User presses Reject on 101 BEFORE getFileInfo(101) resolves, and persistence completes first!
  const write101 = rejectCurrent(async () => 1);
  await write101;
  assert.deepEqual(synced, [[101, 2]]);

  // Now delayed getFileInfo(100) resolves, then delayed getFileInfo(101) resolves with stale DB snapshot 0
  pendingGets.get(100)({ id: 100, culling_flag: 0 });
  await load100;
  assert.equal(leftFileInfo, null, 'stale file 100 load must not populate active pane');

  pendingGets.get(101)({ id: 101, culling_flag: 0 });
  await load101;
  assert.equal(leftFileInfo.id, 101);
  assert.equal(leftFileInfo.culling_flag, 2, 'committed culling flag must survive delayed stale getFileInfo');
  assert.equal(toasts.length, 0);
});

test('cullingSync rolls back on failed persistence and avoids clobbering newer updates', async () => {
  const sync = createCullingSyncTracker();
  const activeFileId = 200;
  const info = { id: 200, culling_flag: 0 };
  const toasts = [];
  const gates = [];

  async function setFlag(flag) {
    const matching = getMatchingPaneFileInfo(info, activeFileId);
    const knownPrev = matching ? Number(matching.culling_flag ?? 0) : null;
    const { seq, normalized } = sync.beginCullingUpdate(activeFileId, flag, knownPrev);
    if (matching) matching.culling_flag = normalized;

    const result = await new Promise((resolve) => gates.push(resolve));
    const finished = sync.finishCullingUpdate(activeFileId, seq, result !== null);
    if (!finished.isLatest) return;
    if (result === null) {
      if (finished.previous !== null) matching.culling_flag = finished.previous;
      toasts.push('error');
    }
  }

  const first = setFlag(2);
  const second = setFlag(1);
  assert.equal(info.culling_flag, 1);

  // Older write fails after newer write was queued; must NOT revert newer flag 1 or toast
  gates[0](null);
  await first;
  assert.equal(info.culling_flag, 1);
  assert.equal(toasts.length, 0);

  // Newer write fails; rolls back to original DB value 0 and shows error toast
  gates[1](null);
  await second;
  assert.equal(info.culling_flag, 0);
  assert.equal(toasts.length, 1);
});
