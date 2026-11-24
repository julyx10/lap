import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const read = (path) => readFile(new URL(path, import.meta.url), "utf8");

test("MediaViewer forwards image context menus to ImageViewer in viewer mode", async () => {
  const source = await read("../src/components/MediaViewer.vue");
  assert.match(source, /'context-menu'/);
  assert.match(source, /emit\('context-menu', e\)/);
  assert.match(source, /props\.mode === 2/);
});

test("ImageViewer listens for context menus from every MediaViewer pane", async () => {
  const source = await read("../src/views/ImageViewer.vue");
  const listeners = [...source.matchAll(/@context-menu="openPaneContextMenu\(/g)];
  assert.ok(listeners.length >= 2, "normal and split MediaViewer must both forward context menus");
});

test("viewer dialogs persist their positions with stable keys", async () => {
  const viewer = await read("../src/views/ImageViewer.vue");
  const rename = await read("../src/components/RenameFileDialog.vue");
  const remove = await read("../src/components/FileDeleteDialog.vue");
  assert.match(viewer, /position-key="edit-tag"/);
  assert.match(viewer, /position-key="edit-collection"/);
  assert.match(rename, /position-key="rename-file"/);
  assert.match(remove, /position-key="file-delete"/);
});

test("delete dialog scope updates are wired by the same event name", async () => {
  const remove = await read("../src/components/FileDeleteDialog.vue");
  const viewer = await read("../src/views/ImageViewer.vue");
  assert.match(remove, /update:rawPairScope/);
  assert.match(viewer, /@update:rawPairScope="rawPairScope = \$event"/);
});

test("viewer registers image-editor save-as context with the main window", async () => {
  const viewer = await read("../src/views/ImageViewer.vue");
  const content = await read("../src/components/Content.vue");
  assert.match(viewer, /register-image-editor-context/);
  assert.match(content, /case 'register-image-editor-context'/);
});

test("viewer move/copy uses the shared conflict dialog", async () => {
  const viewer = await read("../src/views/ImageViewer.vue");
  assert.match(viewer, /<FileConflictDialog/);
  assert.match(viewer, /requestFileConflict\(/);
});

test("viewer hides View in new window and exposes the toolbar more button", async () => {
  const menu = await read("../src/common/fileMenu.ts");
  const media = await read("../src/components/MediaViewer.vue");
  const viewer = await read("../src/views/ImageViewer.vue");
  assert.match(menu, /hideViewInNewWindow/);
  assert.match(media, /viewer-more/);
  assert.match(viewer, /hideViewInNewWindow: true/);
  assert.match(viewer, /@viewer-more="openViewerMoreMenu"/);
});

test("viewer handles Print and Refresh File Info locally", async () => {
  const viewer = await read("../src/views/ImageViewer.vue");
  assert.match(viewer, /printActiveImage/);
  assert.match(viewer, /refreshActiveFileInfo/);
});

test("MoveTo can be dismissed with Escape from the viewer window", async () => {
  const viewer = await read("../src/views/ImageViewer.vue");
  assert.match(viewer, /isInputActive\('MoveTo'\) && event\.key === 'Escape'/);
});

test("More buttons use the localized More... tooltip", async () => {
  const context = await read("../src/components/ContextMenu.vue");
  const media = await read("../src/components/MediaViewer.vue");
  assert.match(context, /tooltip: \{/);
  assert.match(media, /info_panel\.more/);
});

test("all locales define the More tooltip", async () => {
  const fs = await import("node:fs/promises");
  const dir = new URL("../src/locales/", import.meta.url);
  const files = (await fs.readdir(dir)).filter(name => name.endsWith(".json"));
  assert.ok(files.length > 0);
  for (const file of files) {
    const data = JSON.parse(await fs.readFile(new URL(file, dir), "utf8"));
    assert.equal(typeof data.info_panel?.more, "string", `${file} missing info_panel.more`);
  }
});

test("viewer handles album cover and wallpaper actions locally", async () => {
  const viewer = await read("../src/views/ImageViewer.vue");
  assert.match(viewer, /setActiveAlbumCover/);
  assert.match(viewer, /setActiveDesktopWallpaper/);
});

test("MoveTo Enter and Escape are handled in the viewer window", async () => {
  const viewer = await read("../src/views/ImageViewer.vue");
  const moveTo = await read("../src/components/MoveTo.vue");
  assert.match(viewer, /moveToRef\.value\?\.clickOk/);
  assert.match(moveTo, /defineExpose\(\{ clickOk, clickCancel \}\)/);
});

test("viewer refresh syncs the refreshed file back to the main window", async () => {
  const viewer = await read("../src/views/ImageViewer.vue");
  const content = await read("../src/components/Content.vue");
  assert.match(viewer, /message: 'file-refreshed'/);
  assert.match(content, /case 'file-refreshed'/);
  assert.match(content, /Object\.assign\(file, refreshedFile\)/);
});

test("both viewer move entries use the shared ownership sync", async () => {
  const viewer = await read("../src/views/ImageViewer.vue");
  const content = await read("../src/components/Content.vue");
  assert.match(viewer, /async function applyMovedFile/);
  const calls = [...viewer.matchAll(/await applyMovedFile\(file, movedPath/g)];
  assert.equal(calls.length, 2, "library and system-folder moves must share sync");
  assert.match(viewer, /message: 'file-moved'/);
  assert.match(viewer, /file: updatedFile/);
  assert.match(viewer, /file\?\.album_id \|\| libConfig\.album\.id/);
  assert.match(content, /const sourceAlbumId = Number\(file\?\.album_id/);
});

test("moving outside the library drops the deleted viewer identity", async () => {
  const viewer = await read("../src/views/ImageViewer.vue");
  assert.match(viewer, /move_file_outside_library removes the database row/);
  assert.match(viewer, /if \(!libraryDestination\)/);
  assert.match(viewer, /fileIds: \[file\.id\]/);
  assert.match(viewer, /albumId: Number\(file\.album_id/);
  assert.match(viewer, /:file="null"/);
  assert.match(viewer, /:fileCount="0"/);
  assert.doesNotMatch(viewer, /class="absolute top-0 left-0 right-0 z-80"/);
  assert.match(viewer, /keep the full MediaViewer\/image container mounted/);
});

test("trash-failure retry is built from failed components only", async () => {
  const viewer = await read("../src/views/ImageViewer.vue");
  assert.match(viewer, /buildDeleteRetryTargets\(plan\.files, failedIds\)/);
});

test("normal deletion navigation waits for Content's settled file count", async () => {
  const viewer = await read("../src/views/ImageViewer.vue");
  const content = await read("../src/components/Content.vue");
  assert.match(viewer, /source === 'image-viewer' && !isCompareModeSession\.value/);
  assert.match(content, /fileCount: viewerFileCount/);
  assert.match(content, /source: 'content'/);
});

test("deleting one pair member refreshes the surviving member", async () => {
  const content = await read("../src/components/Content.vue");
  const viewer = await read("../src/views/ImageViewer.vue");
  assert.match(content, /survivorIds/);
  assert.match(content, /updatedFiles/);
  assert.match(content, /live_photo_video_id/);
  assert.match(viewer, /applyFileMetaToPanes\(Number\(updatedFile\.id\), updatedFile\)/);
});

test("delete failedCount is reported instead of silently closing", async () => {
  const viewer = await read("../src/views/ImageViewer.vue");
  assert.match(viewer, /const failedCount = Math\.max\(0, Number\(result\.failedCount/);
  assert.match(viewer, /msgbox\.raw_pair_delete\.failed/);
});

test("deleting a RAW also re-queries to surface the surviving companion", async () => {
  const content = await read("../src/components/Content.vue");
  assert.match(content, /surviveCompanionByDeletedPrimary/);
  assert.match(content, /surviveCompanionByDeletedPrimary\.size > 0/);
  assert.match(content, /await updateContent\(true\)/);
  assert.match(content, /imageViewerSession\.value = \{ \.\.\.session, files: nextFiles \}/);
});

test("pair repair collects known files from both list and compare snapshot", async () => {
  const content = await read("../src/components/Content.vue");
  assert.match(content, /const knownFiles = new Map<number, any>\(\)/);
  assert.match(content, /imageViewerSession\.value\.mode === 'compare'/);
  assert.match(content, /knownFiles\.set\(Number\(file\.id\), file\)/);
  assert.match(content, /for \(const file of knownFiles\.values\(\)\)/);
});

test("rename refreshes companion paths in the viewer and main list", async () => {
  const viewer = await read("../src/views/ImageViewer.vue");
  const content = await read("../src/components/Content.vue");
  assert.match(viewer, /rename_file also renames the paired companion/);
  assert.match(viewer, /file: updatedFile/);
  assert.match(content, /if \(renamedFile\) Object\.assign\(file, renamedFile\)/);
  assert.match(content, /Object\.assign\(sessionFile, renamedFile\)/);
});

test("surviving panes shift their indices after an earlier deletion", async () => {
  const content = await read("../src/components/Content.vue");
  const viewer = await read("../src/views/ImageViewer.vue");
  assert.match(content, /deletedIndices/);
  assert.match(viewer, /const shiftForIndex =/);
  assert.match(viewer, /fileIndex\.value - shiftForIndex\(fileIndex\.value\)/);
  assert.match(viewer, /rightFileIndex\.value - shiftForIndex\(rightFileIndex\.value\)/);
});

test("viewer delete dialog loads thumbnails as backend data URLs", async () => {
  const viewer = await read("../src/views/ImageViewer.vue");
  assert.match(viewer, /getFileThumbById/);
  assert.match(viewer, /getThumbnailDataUrl/);
  assert.match(viewer, /function loadDeleteThumbnails/);
  assert.match(viewer, /deletePlan = shallowRef<FileDeletePlan \| null>/);
  assert.match(viewer, /deletePlan\.value !== plan/);
  assert.doesNotMatch(viewer, /getThumbUrl\(target\.id/);
});

test("image editor overwrite reloads the viewer even when the source left the list", async () => {
  const viewer = await read("../src/views/ImageViewer.vue");
  const content = await read("../src/components/Content.vue");
  assert.match(viewer, /case 'file-saved'/);
  assert.match(viewer, /uiStore\.updateFileVersion\(savedFile\.file_path\)/);
  assert.match(content, /message: 'file-saved'/);
  assert.match(content, /getFileInfo\(sourceId\)/);
});

test("viewer deletion reports its snapshot count when the main list moved away", async () => {
  const viewer = await read("../src/views/ImageViewer.vue");
  assert.match(viewer, /fileCount: isCompareModeSession\.value/);
  assert.match(viewer, /fileCount\.value - deletedIds\.length/);
});
