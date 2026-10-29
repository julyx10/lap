import test from 'node:test';
import assert from 'node:assert/strict';
import { createRawPairBackfill } from '../src/common/rawPairBackfill.js';

const tick = () => new Promise(resolve => setImmediate(resolve));
test('busy backfill retries on index completion and coalesces in-flight notifications', async () => {
  let resolveFirst;
  let calls = 0;
  let successes = 0;
  const task = createRawPairBackfill({
    run: () => ++calls === 1 ? new Promise(resolve => { resolveFirst = resolve; }) : Promise.resolve(true),
    libraryId: () => 'a', onSuccess: () => { successes++; }, onError: assert.fail,
  });
  const first = task.request();
  task.indexFinished();
  task.indexFinished();
  resolveFirst(false);
  await first;
  assert.equal(calls, 2);
  assert.equal(successes, 1);
  task.indexFinished();
  await tick();
  assert.equal(calls, 2);
});

test('failed backfill can be retried and stale library completion is ignored', async () => {
  let id = 'a';
  let resolveFirst;
  let calls = 0;
  let errors = 0;
  let successes = 0;
  const task = createRawPairBackfill({
    run: () => ++calls === 1 ? new Promise(resolve => { resolveFirst = resolve; }) : calls === 2 ? Promise.reject(new Error('failed')) : Promise.resolve(true),
    libraryId: () => id, onSuccess: () => { successes++; }, onError: () => { errors++; },
  });
  const first = task.request();
  id = 'b';
  task.request();
  resolveFirst(true);
  await first;
  assert.equal(successes, 0);
  assert.equal(errors, 1);
  await task.request();
  assert.equal(successes, 1);
});
