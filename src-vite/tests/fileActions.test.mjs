import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import ts from "typescript";

const source = await readFile(
  new URL("../src/common/fileActions.ts", import.meta.url),
  "utf8",
);
const exports = {};
const stubs = {
  "@/common/api": {
    renameFile: async (id, path, name) => `${path}#${id}#${name}`,
    checkFileAccessibility: async (path) => path !== "/offline.jpg",
    copyImages: async (paths) => paths.length,
    revealPath: (path) => { globalThis.__revealedPath = path; },
  },
  "@/common/availability": {
    isOriginalStorageUnavailable: (file) => file?.cloud_only === true,
  },
  "@/common/utils": {
    combineFileName: (name, extension) => extension ? `${name}.${extension}` : name,
    getClipboardItemPaths: (file, groupRawPairs) => groupRawPairs && file.media_subtype === "raw_jpeg_pair"
      ? [file.file_path, file.live_photo_video_path]
      : [file.file_path],
    copyFilePaths: async (files) => files.length,
  },
};
new Function(
  "exports",
  "require",
  ts.transpile(source, {
    target: ts.ScriptTarget.ES2022,
    module: ts.ModuleKind.CommonJS,
  }),
)(exports, id => {
  if (!(id in stubs)) throw new Error(`Unexpected import: ${id}`);
  return stubs[id];
});
const {
  dispatchFileAction,
  getMediaKind,
  resolveExternalAppTarget,
  getClipboardFilePaths,
  renameSingleFile,
  buildFileDeleteTargets,
  shouldConfirmDelete,
  buildDeleteRetryTargets,
  countDeleteTargets,
  countRawPairTargets,
  countSingleTargets,
  rawPairCompanionFormat,
  copyFilesToClipboard,
  copyFilePathsForFiles,
  revealFile,
  isOriginalUnavailableActionError,
} = exports;

function harness() {
  const calls = [];
  const hooks = {
    calls,
    run: (handler) => calls.push(["run", handler]),
    onRating: (rating) => calls.push(["rating", rating]),
    onCulling: (flag) => calls.push(["culling", flag]),
    onExternalApp: (appId) => calls.push(["external", appId]),
    onManageExternalApps: () => calls.push(["manage-external"]),
  };
  return hooks;
}

test("rating actions are parsed without invoking a menu handler", () => {
  const hooks = harness();
  const handler = () => {};
  assert.equal(dispatchFileAction("rating-4", { "rating-4": handler }, hooks), true);
  assert.deepEqual(hooks.calls, [["rating", 4]]);
});

test("culling actions map to their flags", () => {
  const hooks = harness();
  dispatchFileAction("culling-pick", {}, hooks);
  dispatchFileAction("culling-reject", {}, hooks);
  dispatchFileAction("culling-unreviewed", {}, hooks);
  assert.deepEqual(hooks.calls, [["culling", 1], ["culling", 2], ["culling", 0]]);
});

test("external-app actions keep their app id", () => {
  const hooks = harness();
  dispatchFileAction("open-external-app:preview", {}, hooks);
  assert.deepEqual(hooks.calls, [["external", "preview"]]);
});

test("menu handlers go through the caller's unsaved-changes guard", () => {
  const hooks = harness();
  let ran = false;
  const handler = () => { ran = true; };
  assert.equal(dispatchFileAction("rename", { rename: handler }, hooks), true);
  assert.equal(ran, false, "handler must not run before the guard");
  assert.equal(hooks.calls.length, 1);
  assert.deepEqual(hooks.calls[0][0], "run");
  assert.equal(typeof hooks.calls[0][1], "function");
});

test("unknown actions are ignored", () => {
  const hooks = harness();
  assert.equal(dispatchFileAction("not-an-action", {}, hooks), false);
  assert.deepEqual(hooks.calls, []);
});

const lookup = {
  externalAppsFor: (kind) => kind === "image" ? [{ id: "preview", path: "/Preview" }] : [],
  defaultExternalApp: (kind) => kind === "image" ? { path: "/Default" } : null,
};

test("media kind distinguishes image, video and mixed selections", () => {
  assert.equal(getMediaKind([{ file_type: 1 }, { file_type: 3 }]), "image");
  assert.equal(getMediaKind([{ file_type: 2 }]), "video");
  assert.equal(getMediaKind([{ file_type: 1 }, { file_type: 2 }]), "mixed");
  assert.equal(getMediaKind([]), "empty");
});

test("external app resolves the requested app id", () => {
  assert.deepEqual(
    resolveExternalAppTarget([{ file_type: 1, file_path: "/a.jpg" }], "preview", lookup),
    { kind: "image", appPath: "/Preview", paths: ["/a.jpg"] },
  );
});

test("external app falls back to the default app", () => {
  assert.deepEqual(
    resolveExternalAppTarget([{ file_type: 1, file_path: "/a.jpg" }], undefined, lookup),
    { kind: "image", appPath: "/Default", paths: ["/a.jpg"] },
  );
});

test("missing app and mixed selections are reported without throwing", () => {
  assert.deepEqual(
    resolveExternalAppTarget([{ file_type: 2, file_path: "/a.mp4" }], undefined, lookup),
    { kind: "no-app" },
  );
  assert.deepEqual(
    resolveExternalAppTarget([{ file_type: 1 }, { file_type: 2 }], undefined, lookup),
    { kind: "mixed" },
  );
});

test("clipboard paths de-duplicate RAW/JPEG companions", () => {
  assert.deepEqual(
    getClipboardFilePaths([
      { file_path: "/a.RAW", live_photo_video_path: "/a.JPG", media_subtype: "raw_jpeg_pair" },
      { file_path: "/b.JPG", media_subtype: "single" },
    ], true, 10),
    { paths: ["/a.RAW", "/a.JPG", "/b.JPG"], filesOnly: true },
  );
});

test("clipboard paths honor the item limit", () => {
  assert.deepEqual(
    getClipboardFilePaths([
      { file_path: "/a.jpg", media_subtype: "single" },
      { file_path: "/b.jpg", media_subtype: "single" },
    ], false, 1),
    { paths: ["/a.jpg"], filesOnly: false },
  );
});

test("renameSingleFile combines the extension and returns the new path", async () => {
  assert.deepEqual(
    await renameSingleFile({ id: 7, name: "old.jpg", file_path: "/old.jpg" }, "new", "jpg"),
    { fileName: "new.jpg", filePath: "/old.jpg#7#new.jpg", unchanged: false },
  );
});

test("delete targets preserve RAW/JPEG companions", () => {
  const [target] = buildFileDeleteTargets([
    { id: 1, name: "a.RAW", file_path: "/a.RAW", media_subtype: "raw_jpeg_pair",
      live_photo_video_id: 2, live_photo_video_path: "/a.JPG" },
  ], true);
  assert.equal(target.rawPair, true);
  assert.deepEqual(target.companion, { id: 2, file_path: "/a.JPG" });
});

const plan = (files) => ({ libraryId: "lib", requestId: 1, files });

test("single unpaired files do not require confirmation", () => {
  assert.equal(shouldConfirmDelete(plan([{ id: 1, rawPair: false }]), false), false);
});

test("permanent deletion always requires confirmation", () => {
  assert.equal(shouldConfirmDelete(plan([{ id: 1, rawPair: false }]), true), true);
});

test("paired files require confirmation and count both components", () => {
  const paired = plan([{ id: 1, rawPair: true, companion: { id: 2, file_path: "/x.JPG" } }]);
  assert.equal(shouldConfirmDelete(paired, false), true);
  assert.equal(countDeleteTargets(paired, "both"), 2);
  assert.equal(countDeleteTargets(paired, "raw"), 1);
  assert.equal(countRawPairTargets(paired), 1);
  assert.equal(countSingleTargets(paired), 0);
  assert.equal(rawPairCompanionFormat(paired), "JPEG");
});

test("empty plans are safe", () => {
  assert.equal(countDeleteTargets(null, "both"), 0);
  assert.equal(countRawPairTargets(null), 0);
  assert.equal(countSingleTargets(null), 0);
  assert.equal(rawPairCompanionFormat(null), "");
});

test("copyFilesToClipboard copies available files", async () => {
  const result = await copyFilesToClipboard([{ file_path: "/a.jpg", media_subtype: "single" }], {
    groupRawJpegPairs: false,
  });
  assert.equal(result.copiedCount, 1);
  assert.equal(result.offline, false);
  assert.equal(result.cancelled, false);
});

test("copyFilesToClipboard reports unavailable originals", async () => {
  const result = await copyFilesToClipboard([{ file_path: "/a.jpg", cloud_only: true }], {
    groupRawJpegPairs: false,
  });
  assert.equal(result.offline, true);
  assert.equal(result.copiedCount, 0);
});

test("copyFilesToClipboard stops when a path is inaccessible", async () => {
  const result = await copyFilesToClipboard([{ file_path: "/offline.jpg", media_subtype: "single" }], {
    groupRawJpegPairs: false,
  });
  assert.equal(result.offline, true);
  assert.equal(result.copiedCount, 0);
});

test("copyFilePathsForFiles delegates to the clipboard path API", async () => {
  assert.equal(await copyFilePathsForFiles([{ file_path: "/a.jpg" }], false), 1);
});

test("revealFile reveals the file path", () => {
  assert.equal(revealFile({ file_path: "/a.jpg" }), true);
  assert.equal(globalThis.__revealedPath, "/a.jpg");
});

test("original_unavailable errors are recognized for offline UI", () => {
  assert.equal(isOriginalUnavailableActionError("original_unavailable: /a.jpg"), true);
  assert.equal(isOriginalUnavailableActionError("other error"), false);
});

test("delete retry only targets failed components and drops pair expansion", () => {
  const paired = {
    id: 1, name: "a.RAW", file_path: "/a.RAW", rawPair: true,
    companion: { id: 2, file_path: "/a.JPG" },
  };
  assert.deepEqual(buildDeleteRetryTargets([paired], [1]), [
    { ...paired, rawPair: false, companion: undefined },
  ]);
  assert.deepEqual(buildDeleteRetryTargets([paired], [2]), [
    { id: 2, name: "a.JPG", file_path: "/a.JPG", album_id: undefined, rawPair: false },
  ]);
});
