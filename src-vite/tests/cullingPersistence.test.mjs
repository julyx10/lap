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

const syncSource = fs.readFileSync(new URL('../src/common/metadataSync.ts', import.meta.url), 'utf8');
const syncJs = ts.transpile(
  syncSource.replace(/export /g, ''),
  { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext },
);
const {
  createMetadataSyncTracker,
  getMatchingPaneFileInfo,
} = new Function(`${syncJs}; return { createMetadataSyncTracker, getMatchingPaneFileInfo };`)();

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
  const sync = createMetadataSyncTracker();
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
    leftFileInfo = sync.applyPendingMetadata(loaded, req);
  }

  async function setCurrentFlag(flag, persistFn) {
    const currentId = activeFileId;
    const matching = getMatchingPaneFileInfo(leftFileInfo, currentId);
    const knownConfirmed = matching ? Number(matching.culling_flag ?? 0) : null;
    await sync.enqueueMetadataUpdate({
      fileId: currentId,
      value: flag,
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
  const sync = createMetadataSyncTracker();
  const fileId = 200;
  const info = { id: fileId, culling_flag: 0 };
  const synced = [];
  const toasts = [];
  const calls = [];
  const gates = [];

  function triggerFlag(flag) {
    const matching = getMatchingPaneFileInfo(info, fileId);
    const knownConfirmed = matching ? Number(matching.culling_flag ?? 0) : null;
    return sync.enqueueMetadataUpdate({
      fileId,
      value: flag,
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
  const sync = createMetadataSyncTracker();
  const fileId = 201;
  const info = { id: fileId, culling_flag: 0 };
  const toasts = [];
  const calls = [];
  const gates = [];

  function triggerFlag(flag) {
    const matching = getMatchingPaneFileInfo(info, fileId);
    const knownConfirmed = matching ? Number(matching.culling_flag ?? 0) : null;
    return sync.enqueueMetadataUpdate({
      fileId,
      value: flag,
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


test('same-value updates use the latest auto-advance intent and pane', async () => {
  for (const [firstAdvance, latestAdvance] of [[false, true], [true, false]]) {
    const sync = createMetadataSyncTracker();
    const commits = [];
    let finish;
    let writes = 0;
    const options = {
      fileId: 300, value: 2, knownConfirmed: 0,
      pane: 'left', advance: firstAdvance,
      applyOptimistic: () => {},
      persist: () => { writes++; return new Promise(resolve => { finish = resolve; }); },
      onCommit: (_id, _flag, settled, advanceInfo) => commits.push({ settled, advanceInfo }),
      onError: () => assert.fail('write should succeed'),
    };
    const first = sync.enqueueMetadataUpdate(options);
    const latest = sync.enqueueMetadataUpdate({ ...options, pane: 'right', advance: latestAdvance });
    assert.equal(writes, 1);
    finish(1);
    await Promise.all([first, latest]);
    assert.equal(writes, 1);
    assert.deepEqual(commits, [{ settled: true, advanceInfo: { pane: 'right', advance: latestAdvance } }]);
  }
});


for (const [field, firstValue, latestValue] of [['rating', 3, 5], ['is_favorite', 1, 0]]) {
  test(`${field}: serial writes, failed rollback and stale read protection`, async () => {
    const sync = createMetadataSyncTracker(field);
    const load = sync.beginFileInfoLoad('left');
    const info = { id: 500, [field]: field === 'is_favorite' ? false : 0 };
    const gates = [];
    const calls = [];
    const options = {
      fileId: 500, knownConfirmed: 0, pane: 'left', advance: false,
      applyOptimistic: (_id, value) => { info[field] = value; },
      persist: (_id, value) => { calls.push(value); return new Promise(resolve => gates.push(resolve)); },
      onCommit: () => {},
      onError: (_id, value) => { info[field] = value; },
    };
    const first = sync.enqueueMetadataUpdate({ ...options, value: firstValue });
    const second = sync.enqueueMetadataUpdate({ ...options, value: latestValue });
    assert.deepEqual(calls, [firstValue]);
    gates[0](1);
    await Promise.resolve(); await Promise.resolve(); await Promise.resolve();
    assert.deepEqual(calls, [firstValue, latestValue]);
    gates[1](null);
    await Promise.all([first, second]);
    assert.equal(info[field], firstValue);
    const loaded = sync.applyPendingMetadata({ id: 500, [field]: 0 }, load);
    assert.equal(loaded[field], field === 'is_favorite' ? Boolean(firstValue) : firstValue);
  });
}

test('rejected persistence rolls back and allows a later retry', async () => {
  const sync = createMetadataSyncTracker('rating');
  const errors = [];
  const options = { fileId: 600, value: 5, knownConfirmed: 2, pane: 'left', advance: false,
    applyOptimistic: () => {}, persist: async () => { throw Error('write failed'); },
    onCommit: () => {}, onError: (_id, value) => errors.push(value) };
  await sync.enqueueMetadataUpdate(options);
  assert.deepEqual(errors, [2]);
  let committed;
  await sync.enqueueMetadataUpdate({ ...options, persist: async () => 1, onCommit: (_id, value) => { committed = value; } });
  assert.equal(committed, 5);
});


for (const name of ['setFileRating', 'setFileFavorite']) {
  test(`${name} rejects invalid IDs and zero-row writes`, async () => {
    const start = apiSource.indexOf(`export async function ${name}`);
    const end = apiSource.indexOf('\n}\n', start) + 3;
    const snippet = apiSource.slice(start, end).replace('export async function ', 'async function ');
    const calls = [];
    let result = 0;
    const api = new Function('invoke', `${snippet}; return ${name};`)(async (...args) => { calls.push(args); return result; });
    assert.equal(await api(0, 1), null);
    assert.equal(await api('5', 1), null);
    assert.equal(calls.length, 0);
    assert.equal(await api(5, 1), null);
    result = 1;
    assert.equal(await api(5, 1), 1);
  });
}


test('grid rollback updates the current file object after list replacement', async () => {
  const source = fs.readFileSync(new URL('../src/components/Content.vue', import.meta.url), 'utf8');
  const start = source.indexOf('async function setSelectedMetadata(');
  const end = source.indexOf('\nconst toggleFavorite', start);
  const snippet = ts.transpile(source.slice(start, end), { target: ts.ScriptTarget.ES2022 });
  const fileList = { value: [{ id: 42, rating: 0 }] };
  const libConfig = { _libraryId: 'library-a' };
  const gates = [];
  const metadataWriters = { rating: { tracker: createMetadataSyncTracker('rating'),
    persist: () => new Promise(resolve => gates.push(resolve)) } };
  const set = new Function('metadataWriters', 'fileList', 'libConfig', 'selectedItemIndex', 'isRealFileItem',
    'currentContentRequestId', 'syncFileMetaToImageViewer', 'tauriEmit', 'advanceAfterMetadataShortcut', 'toast', 't',
    `${snippet}; return setSelectedMetadata;`)(metadataWriters, fileList, libConfig, { value: 0 }, file => file?.id > 0,
      1, () => {}, () => {}, () => {}, { error: () => {} }, key => key);
  const first = set('rating', 3, false);
  const old = fileList.value[0];
  fileList.value = [{ id: 42, rating: 0 }];
  const second = set('rating', 5, false);
  assert.equal(fileList.value[0].rating, 5);
  gates[0](1);
  await Promise.resolve(); await Promise.resolve(); await Promise.resolve();
  gates[1](null);
  await Promise.all([first, second]);
  assert.equal(fileList.value[0].rating, 3);
  assert.notEqual(old, fileList.value[0]);
});


test('auto-advance uses the latest list request after refresh', async () => {
  const sync = createMetadataSyncTracker('rating');
  let finish;
  const intents = [];
  const options = { fileId: 700, value: 3, knownConfirmed: 0, pane: 'left', advance: false, requestId: 10,
    applyOptimistic: () => {}, persist: () => new Promise(resolve => { finish = resolve; }),
    onCommit: (_id, _value, _settled, intent) => intents.push(intent), onError: () => assert.fail('unexpected failure') };
  const first = sync.enqueueMetadataUpdate(options);
  const latest = sync.enqueueMetadataUpdate({ ...options, advance: true, requestId: 11 });
  finish(1);
  await Promise.all([first, latest]);
  assert.deepEqual(intents, [{ pane: 'left', advance: true, requestId: 11 }]);
});
