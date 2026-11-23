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

  async function setCurrentFlag(flag, persistFn) {
    const currentId = activeFileId;
    const matching = getMatchingPaneFileInfo(leftFileInfo, currentId);
    const knownConfirmed = matching ? Number(matching.culling_flag ?? 0) : null;
    await sync.enqueueCullingUpdate({
      fileId: currentId,
      cullingFlag: flag,
      knownConfirmed,
      pane: 'left',
      advance: false,
      applyOptimistic: (id, nextFlag) => {
        const curr = getMatchingPaneFileInfo(leftFileInfo, id);
        if (curr) curr.culling_flag = nextFlag;
      },
      persist: persistFn,
      onCommit: (id, nextFlag, settled) => {
        if (settled) {
          const curr = getMatchingPaneFileInfo(leftFileInfo, id);
          if (curr) curr.culling_flag = nextFlag;
        }
        synced.push([id, nextFlag]);
      },
      onError: (id, rollbackFlag) => {
        const curr = getMatchingPaneFileInfo(leftFileInfo, id);
        if (curr && rollbackFlag !== null) curr.culling_flag = rollbackFlag;
        toasts.push('error');
      },
    });
  }

  // Navigate to 100 (slow), then immediately to 101
  const load100 = loadLeft(100);
  activeFileId = 101;
  const load101 = loadLeft(101);

  // User presses Reject on 101 BEFORE getFileInfo(101) resolves, and persistence completes first
  await setCurrentFlag(2, async () => 1);
  assert.deepEqual(synced, [[101, 2]]);

  // Delayed getFileInfo(100) resolves, then delayed getFileInfo(101) resolves with stale DB snapshot 0
  pendingGets.get(100)({ id: 100, culling_flag: 0 });
  await load100;
  assert.equal(leftFileInfo, null, 'stale file 100 load must not populate active pane');

  pendingGets.get(101)({ id: 101, culling_flag: 0 });
  await load101;
  assert.equal(leftFileInfo.id, 101);
  assert.equal(leftFileInfo.culling_flag, 2, 'committed culling flag must survive delayed stale getFileInfo');
  assert.equal(toasts.length, 0);
});

test('consecutive updates serialize per file and roll back to last confirmed Reject when subsequent Pick fails', async () => {
  const sync = createCullingSyncTracker();
  const fileId = 200;
  const info = { id: fileId, culling_flag: 0 };
  const synced = [];
  const toasts = [];
  const calls = [];
  const gates = [];

  function triggerFlag(flag) {
    const matching = getMatchingPaneFileInfo(info, fileId);
    const knownConfirmed = matching ? Number(matching.culling_flag ?? 0) : null;
    return sync.enqueueCullingUpdate({
      fileId,
      cullingFlag: flag,
      knownConfirmed,
      pane: 'left',
      advance: false,
      applyOptimistic: (_id, nextFlag) => {
        info.culling_flag = nextFlag;
      },
      persist: (_id, nextFlag) => {
        calls.push(nextFlag);
        return new Promise((resolve) => gates.push(resolve));
      },
      onCommit: (id, nextFlag, settled) => {
        if (settled) info.culling_flag = nextFlag;
        synced.push([id, nextFlag]);
      },
      onError: (_id, rollbackFlag) => {
        if (rollbackFlag !== null) info.culling_flag = rollbackFlag;
        toasts.push(rollbackFlag);
      },
    });
  }

  // Start from Unreviewed (0): trigger Reject (2), then Pick (1) while Reject is still in flight
  const first = triggerFlag(2);
  const second = triggerFlag(1);

  assert.equal(info.culling_flag, 1, 'UI reflects latest user intent immediately');
  assert.deepEqual(calls, [2], 'Pick must wait until Reject finishes (serialized per file)');

  // Reject (2) succeeds; Pick (1) should now be dispatched next
  gates[0](1);
  await Promise.resolve();
  await Promise.resolve();
  assert.deepEqual(synced, [[fileId, 2]]);
  assert.deepEqual(calls, [2, 1]);
  assert.equal(info.culling_flag, 1, 'UI still reflects pending Pick while Pick is in flight');

  // Subsequent Pick (1) fails: UI must roll back to confirmed Reject (2), NOT Unreviewed (0)
  gates[1](null);
  await Promise.all([first, second]);

  assert.equal(info.culling_flag, 2, 'rollback restores last confirmed Reject state');
  assert.deepEqual(toasts, [2]);
});

test('failed earlier write does not flash error or revert when a newer queued intent succeeds', async () => {
  const sync = createCullingSyncTracker();
  const fileId = 201;
  const info = { id: fileId, culling_flag: 0 };
  const toasts = [];
  const calls = [];
  const gates = [];

  function triggerFlag(flag) {
    const matching = getMatchingPaneFileInfo(info, fileId);
    const knownConfirmed = matching ? Number(matching.culling_flag ?? 0) : null;
    return sync.enqueueCullingUpdate({
      fileId,
      cullingFlag: flag,
      knownConfirmed,
      pane: 'left',
      advance: false,
      applyOptimistic: (_id, nextFlag) => {
        info.culling_flag = nextFlag;
      },
      persist: (_id, nextFlag) => {
        calls.push(nextFlag);
        return new Promise((resolve) => gates.push(resolve));
      },
      onCommit: (_id, nextFlag, settled) => {
        if (settled) info.culling_flag = nextFlag;
      },
      onError: (_id, rollbackFlag) => {
        if (rollbackFlag !== null) info.culling_flag = rollbackFlag;
        toasts.push(rollbackFlag);
      },
    });
  }

  const first = triggerFlag(2);
  const second = triggerFlag(1);
  assert.deepEqual(calls, [2]);

  // First write (2) fails while newer intent (1) is queued
  gates[0](null);
  await Promise.resolve();
  await Promise.resolve();
  assert.deepEqual(calls, [2, 1]);
  assert.equal(toasts.length, 0);
  assert.equal(info.culling_flag, 1);

  // Second write (1) succeeds
  gates[1](1);
  await Promise.all([first, second]);
  assert.equal(info.culling_flag, 1);
  assert.equal(toasts.length, 0);
});
