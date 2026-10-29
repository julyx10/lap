import test from 'node:test';
import assert from 'node:assert/strict';
import { mapVisibleRawPairSelection } from '../src/common/rawPairSelection.js';

test('maps hidden JPEGs to visible RAWs, deduplicates and preserves unmapped visible IDs', () => {
  const available = new Set([10, 30, 40]);
  const { selected, mapId } = mapVisibleRawPairSelection(new Set([10, 20, 30, 50]), available, { 20: 10, 50: 99 });
  assert.deepEqual([...selected], [10, 30]);
  assert.equal(mapId(20), 10);
  assert.equal(mapId(30), 30);
  assert.equal(available.has(mapId(50)), false);
});

test('a visible companion is never replaced by an unrelated primary', () => {
  const { selected } = mapVisibleRawPairSelection(new Set([20]), new Set([20, 10]), { 20: 10 });
  assert.deepEqual([...selected], [20]);
});
