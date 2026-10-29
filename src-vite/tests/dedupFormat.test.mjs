import test from 'node:test';
import assert from 'node:assert/strict';
import { dedupFormatLabel } from '../src/common/dedupFormat.js';

test('shows actual RAW companion format only when grouped', () => {
  const raw = { name: 'DSC_0028.NEF', file_type: 3, media_subtype: 'raw_jpeg_pair', live_photo_video_path: '/photos/DSC_0028.JPG' };
  assert.equal(dedupFormatLabel(raw), 'RAW+JPG');
  assert.equal(dedupFormatLabel({ ...raw, live_photo_video_path: '/photos/DSC_0028.hif' }), 'RAW+HIF');
  assert.equal(dedupFormatLabel(raw, false), 'RAW');
  assert.equal(dedupFormatLabel({ ...raw, live_photo_video_path: null }), 'RAW+…');
});

test('standalone files and Live Photos are not mislabeled as RAW pairs', () => {
  assert.equal(dedupFormatLabel({ name: 'a.NEF', file_type: 3 }), 'RAW');
  assert.equal(dedupFormatLabel({ name: 'a.jpg', file_type: 1 }), 'JPG');
  assert.equal(dedupFormatLabel({ name: 'a.HEIC', media_subtype: 'live_photo', live_photo_video_path: 'a.mov' }), 'HEIC');
  assert.equal(dedupFormatLabel(null), '');
});
