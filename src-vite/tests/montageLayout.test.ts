// Run with: node --test tests/montageLayout.test.ts
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { computeMontageLayout, type MontageMode, type MontageItem } from '../src/common/montageLayout.ts';

const EPS = 1e-6;
const ratios = [1.5, 0.67, 1, 1.78, 0.75, 1.33, 0.56];
const photosOf = (n: number) => Array.from({ length: n }, (_, i) => ({ fileId: i + 1, ratio: ratios[i % ratios.length] }));
const style = { spacing: 0.01, border: 0.008 };
const pageRatios = [297 / 210, 210 / 297, 16 / 9, 1];

function overlaps(a: MontageItem, b: MontageItem) {
  return a.x + a.w > b.x + EPS && b.x + b.w > a.x + EPS && a.y + a.h > b.y + EPS && b.y + b.h > a.y + EPS;
}

for (const mode of ['grid', 'mosaic', 'pile'] as MontageMode[]) {
  test(`${mode}: every photo is placed once, inside the page`, () => {
    for (const n of [2, 3, 7, 50]) {
      for (const pageRatio of pageRatios) {
        const items = computeMontageLayout(photosOf(n), pageRatio, mode, style, 7);
        assert.equal(items.length, n);
        assert.deepEqual(items.map(i => i.fileId).sort((a, b) => a - b), photosOf(n).map(p => p.fileId));
        for (const item of items) {
          assert.ok(item.w > 0 && item.h > 0);
          assert.ok(item.x >= -EPS && item.y >= -EPS, `${mode} n=${n}: ${JSON.stringify(item)}`);
          assert.ok(item.x + item.w <= 1 + EPS && item.y + item.h <= 1 + EPS, `${mode} n=${n}: ${JSON.stringify(item)}`);
        }
      }
    }
  });

  test(`${mode}: same seed gives the same layout`, () => {
    const a = computeMontageLayout(photosOf(7), 1.5, mode, style, 42);
    assert.deepEqual(computeMontageLayout(photosOf(7), 1.5, mode, style, 42), a);
    assert.notDeepEqual(computeMontageLayout(photosOf(7), 1.5, mode, style, 43).map(i => i.fileId), a.map(i => i.fileId));
  });
}

test('seed 0 keeps the selection order', () => {
  assert.deepEqual(computeMontageLayout(photosOf(5), 1, 'grid', style, 0).map(i => i.fileId), [1, 2, 3, 4, 5]);
});

for (const mode of ['grid', 'mosaic'] as MontageMode[]) {
  test(`${mode}: photos never overlap and are not rotated`, () => {
    for (const n of [2, 3, 7, 50]) {
      for (const pageRatio of pageRatios) {
        const items = computeMontageLayout(photosOf(n), pageRatio, mode, style, 1);
        items.forEach((a, i) => {
          assert.equal(a.rotation, 0);
          items.slice(i + 1).forEach(b => assert.ok(!overlaps(a, b), `${mode} n=${n} ratio=${pageRatio}`));
        });
      }
    }
  });
}

test('mosaic covers the page except the gaps', () => {
  for (const n of [2, 3, 7, 50]) {
    for (const pageRatio of pageRatios) {
      const noGap = { spacing: 0, border: 0 };
      const items = computeMontageLayout(photosOf(n), pageRatio, 'mosaic', noGap, 1);
      const area = items.reduce((sum, i) => sum + i.w * i.h, 0);
      assert.ok(Math.abs(area - 1) < 0.01, `n=${n} ratio=${pageRatio}: area ${area}`);
    }
  }
});

test('pile photos keep their aspect ratio and are rotated within ±15°', () => {
  const pageRatio = 1.5;
  const noBorder = { spacing: 0, border: 0 };
  const photos = photosOf(7);
  const items = computeMontageLayout(photos, pageRatio, 'pile', noBorder, 0);
  items.forEach((item, i) => {
    assert.ok(Math.abs(item.rotation) <= 15);
    const ratio = (item.w * pageRatio) / item.h; // back to absolute units
    assert.ok(Math.abs(ratio - photos[i].ratio) < 1e-6);
  });
});
