import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import ts from "typescript";

const treeSource = await readFile(
  new URL("../src/common/tagTree.ts", import.meta.url),
  "utf8",
);
const exports = {};
new Function(
  "exports",
  ts.transpile(treeSource, {
    target: ts.ScriptTarget.ES2022,
    module: ts.ModuleKind.CommonJS,
  }),
)(exports);
const { tagRows, ancestorIds, subtreeIds } = exports;
test("tag names reject delimiters and allow asterisks", () => {
  for (const name of ["", " ", "\t ", "a,b", "a;b", "a|b", "bad\0name", "a".repeat(256)])
    assert.equal(exports.isValidTagName(name), false, JSON.stringify(name));
  for (const name of [" tag", "tag ", "\ttag", "tag\t", "a/b", "a\\b", "two words", "a*b", "*tag", "tag*", "*", "旅行", "a".repeat(255)])
    assert.equal(exports.isValidTagName(name), true, JSON.stringify(name));
});
const tags = [
  { id: 1, name: "Places", parent_id: null, path: "Places", count: 2 },
  { id: 2, name: "France", parent_id: 1, path: "Places|France", count: 2 },
  { id: 3, name: "Paris", parent_id: 2, path: "Places|France|Paris", count: 1 },
  { id: 4, name: "Clients", parent_id: null, path: "Clients", count: 1 },
  { id: 5, name: "Paris", parent_id: 4, path: "Clients|Paris", count: 1 },
];

test("tree rows preserve depth, distinct paths and collapse boundaries", () => {
  assert.deepEqual(
    tagRows(tags).map((t) => [t.id, t.depth]),
    [
      [4, 0],
      [5, 1],
      [1, 0],
      [2, 1],
      [3, 2],
    ],
  );
  assert.deepEqual(
    tagRows(tags, "", [1]).map((t) => t.id),
    [4, 5, 1],
  );
  assert.deepEqual(ancestorIds(tags, 3), [2, 1]);
  assert.deepEqual([...subtreeIds(tags, 1)].sort(), [1, 2, 3]);
});

test("search reveals the matching paths and their ancestors without opening unrelated branches", () => {
  assert.deepEqual(
    tagRows(tags, "paris", [1, 4]).map((t) => t.id),
    [4, 5, 1, 2, 3],
  );
  assert.deepEqual(
    tagRows(tags, "france", [1]).map((t) => t.id),
    [1, 2, 3],
  );
  assert.deepEqual(tagRows(tags, "missing"), []);
  assert.deepEqual(
    tagRows(tags, "", [], 3)
      .filter((t) => t.depth === 0)
      .map((t) => t.id),
    [1, 4],
  );
});

test("new tag drafts stay last among siblings until saved", () => {
  for (const sort of [0, 1, 2, 3]) {
    for (const parent_id of [null, 1]) {
      const draft = { id: -1, name: "A draft", path: "A draft", parent_id, count: 999 };
      const rows = tagRows([...tags, draft], "", [], sort, -1);
      assert.equal(rows.filter(tag => tag.parent_id === parent_id).at(-1).id, -1);
    }
  }
  const saved = { id: 6, name: "A saved", path: "A saved", parent_id: null };
  assert.equal(tagRows([...tags, saved], "", [], 0, -1)[0].id, 6);
});

test("deep tag trees do not depend on the JavaScript recursion limit", () => {
  const deep = Array.from({ length: 2000 }, (_, index) => ({
    id: index + 1,
    parent_id: index || null,
    name: `node-${index}`,
    path: `node-${index}`,
  }));
  const rows = tagRows(deep);
  assert.equal(rows.length, 2000);
  assert.equal(rows.at(-1).depth, 1999);
  assert.equal(subtreeIds(deep, 1).size, 2000);
});

const dialog = await readFile(
  new URL("../src/components/TaggingDialog.vue", import.meta.url),
  "utf8",
);
const beginMarker = "// TAG_SELECTION_TEST_BEGIN";
const endMarker = "// TAG_SELECTION_TEST_END";
assert.equal(dialog.split(beginMarker).length, 2);
assert.equal(dialog.split(endMarker).length, 2);
const begin = dialog.indexOf(beginMarker);
const end = dialog.indexOf(endMarker);
assert.ok(begin >= 0 && end > begin);
const selection = ts.transpile(dialog.slice(begin, end), {
  target: ts.ScriptTarget.ES2022,
});
function fixture(initial = [], partial = []) {
  const ref = (value) => ({ value });
  const deps = {
    canEditTags: ref(true),
    editor: ref(null),
    selectedTags: ref(new Set(initial)),
    intermediateTags: ref(new Set(partial)),
    initialSelectedTags: ref(new Set(initial)),
    initialIntermediateTags: ref(new Set(partial)),
    tagChanges: ref(new Map()),
  };
  const toggle = new Function(
    ...Object.keys(deps),
    selection + "; return toggleTag;",
  )(...Object.values(deps));
  return { ...deps, toggle };
}

test("tag selection keeps tri-state batch semantics and never implicitly assigns ancestors", () => {
  const empty = fixture();
  empty.toggle(3);
  assert.deepEqual([...empty.selectedTags.value], [3]);
  assert.deepEqual([...empty.tagChanges.value], [[3, "add"]]);
  empty.toggle(3);
  assert.equal(empty.tagChanges.value.size, 0);
  const mixed = fixture([1], [3]);
  mixed.toggle(3);
  assert.equal(mixed.tagChanges.value.get(3), "add");
  mixed.toggle(3);
  assert.equal(mixed.tagChanges.value.get(3), "remove");
  mixed.toggle(3);
  assert.equal(mixed.tagChanges.value.size, 0);
  assert.ok(mixed.intermediateTags.value.has(3));
  mixed.toggle(1);
  assert.equal(mixed.tagChanges.value.get(1), "remove");
  mixed.toggle(1);
  assert.equal(mixed.tagChanges.value.size, 0);
});
