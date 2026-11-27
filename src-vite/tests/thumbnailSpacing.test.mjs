import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import ts from 'typescript';

const compile = (path, exports) => new Function(ts.transpile(
  readFileSync(new URL(path, import.meta.url), 'utf8').replace(/export /g, ''),
  { target: ts.ScriptTarget.ES2022 },
) + `; return { ${exports} };`)();
const { thumbnailSpacing, cardItemMetrics } = compile('../src/common/thumbnailSpacing.ts', 'thumbnailSpacing, cardItemMetrics');
const layout = compile('../src/common/layout.ts', 'calculateJustifiedLayout, calculateMasonryLayout, calculateLinearRowLayout, calculateLinearColumnLayout');
const photos = Array.from({ length: 12 }, (_, id) => ({ id, width: 400, height: 300 }));

test('card slots contain the rem-scaled frame and label lines at every font size', () => {
  for (const scale of [.8, .9, 1, 1.1, 1.2]) {
    for (const primary of [false, true]) for (const secondary of [false, true]) {
      const metrics = cardItemMetrics(scale, primary, secondary);
      const renderedFrame = 8 * scale + 4;
      const renderedLabels = (primary ? 20 * scale : 0) + (secondary ? 16 * scale : 0);
      assert(Math.abs(metrics.labelHeight - renderedLabels) < 1e-6);
      assert(metrics.frame + metrics.labelHeight >= renderedFrame + renderedLabels);
      assert(Math.abs(metrics.frame - renderedFrame - 8 * scale) < 1e-6);
    }
  }
});

test('compact adds no space and rem presets follow the app font scale', () => {
  assert.equal(thumbnailSpacing(0, 1.2).pixels, 0);
  for (const mode of [1, 2]) {
    const normal = thumbnailSpacing(mode, 1);
    assert.equal(thumbnailSpacing(mode, 1.2).pixels, normal.pixels * 1.2);
    assert(normal.x.startsWith('pr-') && normal.y.startsWith('pb-'));
  }
  assert.equal(thumbnailSpacing(undefined).pixels, 8);
  assert.equal(thumbnailSpacing(2).pixels, 20);
});

test('geometry grids keep the configured gaps without overlapping photos', () => {
  for (const mode of [0, 1, 2]) {
    const gap = thumbnailSpacing(mode).pixels;
    for (const calculate of [layout.calculateJustifiedLayout, layout.calculateMasonryLayout]) {
      const result = calculate(photos, 900, calculate === layout.calculateMasonryLayout ? 160 + gap : 160, gap);
      assert.equal(result.boxes.length, photos.length);
      for (const [i, a] of result.boxes.entries()) {
        assert(a.width > 0 && a.height > 0);
        if (calculate === layout.calculateMasonryLayout) assert(a.width >= 160);
        for (const b of result.boxes.slice(i + 1)) {
          assert(a.x + a.width + gap <= b.x + .01 || b.x + b.width + gap <= a.x + .01
            || a.y + a.height + gap <= b.y + .01 || b.y + b.height + gap <= a.y + .01);
        }
      }
    }
  }
});

test('filmstrip spacing changes only the scrolling axis, preserving thumbnail size', () => {
  for (const mode of [0, 1, 2]) {
    const gap = thumbnailSpacing(mode).pixels;
    const row = layout.calculateLinearRowLayout(photos, 160, gap).boxes;
    const column = layout.calculateLinearColumnLayout(photos, 160, gap).boxes;
    assert.equal(row[0].height, 160);
    assert.equal(column[0].width, 160);
    assert.equal(row[1].x - row[0].x - row[0].width, gap);
    assert.equal(column[1].y - column[0].y - column[0].height, gap);
  }
});
