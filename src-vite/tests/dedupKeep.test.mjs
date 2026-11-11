import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';
import { ref } from 'vue';

const source = fs.readFileSync(new URL('../src/components/DedupPane.vue', import.meta.url), 'utf8');
const start = source.indexOf('async function setKeep(');
const end = source.indexOf('async function setSimilarKeep(', start);
assert.ok(start >= 0 && end > start);
const code = ts.transpile(source.slice(start, end), { target: ts.ScriptTarget.ES2022 });

function keepFixture(detail) {
  const calls = [];
  const selected = new Set();
  const rawGroups = ref([{
    id: 7,
    items: [
      { file_id: 1, is_keep: 1, can_delete: false, file: { id: 1, name: 'keep.raw' } },
      { file_id: 2, is_keep: 0, can_delete: true, file: { id: 2, name: 'next.raw' } },
    ],
  }]);
  const setKeep = new Function(
    'dedupSetKeep', 'dedupGetGroupItems', 'rawGroups', 'getDupSelectedSet',
    'emitDedupStatuses', 'refreshOverview', 'emit',
    `${code}; return setKeep;`,
  )(
    async (groupId, fileId) => calls.push(['setKeep', groupId, fileId]),
    async () => detail,
    rawGroups,
    () => selected,
    () => calls.push(['statuses']),
    () => calls.push(['overview']),
    (...args) => calls.push(['emit', ...args]),
  );
  return { calls, selected, rawGroups, setKeep };
}

test('setKeep refreshes can_delete from the backend group state', async () => {
  const detail = [
    { file_id: 1, is_keep: 0, can_delete: true, file: { id: 1, name: 'keep.raw' } },
    { file_id: 2, is_keep: 1, can_delete: false, file: { id: 2, name: 'next.raw' } },
  ];
  const f = keepFixture(detail);
  await f.setKeep(7, 2);

  assert.deepEqual(f.calls[0], ['setKeep', 7, 2]);
  assert.deepEqual(f.calls[1], ['statuses']);
  assert.deepEqual(f.calls[2], ['overview']);
  assert.deepEqual(f.calls[3], ['emit', 'select-file', 2]);
  assert.equal(f.selected.has(1), true);
  assert.equal(f.selected.has(2), false);
  const items = f.rawGroups.value[0].items;
  assert.equal(items.find(item => item.file_id === 1).can_delete, true);
  assert.equal(items.find(item => item.file_id === 2).can_delete, false);
  assert.equal(items.find(item => item.file_id === 1).file.name, 'keep.raw');
});
