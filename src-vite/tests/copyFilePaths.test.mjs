import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import ts from 'typescript';
const source = readFileSync(new URL('../src/common/utils.ts', import.meta.url), 'utf8')
  .split('export function getClipboardItemPaths')[1].split('/// get full path')[0];
const functionSource = ('function getClipboardItemPaths' + source).replace('export async function', 'async function');
test('copy paths keeps offline and Windows paths intact, deduplicates and skips missing paths',async()=>{
 const calls=[];
 const copy=new Function('invoke',ts.transpile(functionSource,{target:ts.ScriptTarget.ES2022})+';return copyFilePaths;')(async(...args)=>calls.push(args));
 assert.equal(await copy([{file_path:'/offline/My photo.raw'},{file_path:'C:\\Photos\\a.jpg'},{file_path:'/offline/My photo.raw'},null,{}]),2);
 assert.deepEqual(calls,[['copy_file_paths',{filePaths:['/offline/My photo.raw','C:\\Photos\\a.jpg']}]]);
 assert.equal(await copy([]),0);
 assert.equal(calls.length,1);
});

test('shortcuts copy grouped RAW and Live Photo paths, and File Info copies its current file', async () => {
  const calls = [];
  const copy = new Function('invoke', ts.transpile(functionSource, {target: ts.ScriptTarget.ES2022}) + ';return copyFilePaths;')(async (...args) => calls.push(args));
  const raw = {file_path:'/a.raw', media_subtype:'raw_jpeg_pair', live_photo_video_path:'/a.jpg'};
  const live = {file_path:'/b.heic', media_subtype:'live_photo', live_photo_video_path:'/b.mov'};
  assert.equal(await copy([raw, live, {file_path: '/a.jpg'}], true), 4);
  assert.deepEqual(calls[0][1].filePaths, ['/a.raw','/a.jpg','/b.heic','/b.mov']);
  assert.equal(await copy([{file_path:'/a.jpg'}]), 1);
  assert.deepEqual(calls[1][1].filePaths, ['/a.jpg']);
  assert.equal(await copy([raw]), 1);
  assert.deepEqual(calls[2][1].filePaths, ['/a.raw']);
});

test('RAW clipboard paths follow the grouping switch', async () => {
  const calls = [];
  const copy = new Function('invoke', ts.transpile(functionSource, {target: ts.ScriptTarget.ES2022}) + ';return copyFilePaths;')(async (...args) => calls.push(args));
  const raw = {file_path:'/a.raw',media_subtype:'raw_jpeg_pair',live_photo_video_path:'/a.jpg'};
  for (const [mode, expected] of [[true,['/a.raw','/a.jpg']],[false,['/a.raw']]]) {
    assert.equal(await copy([raw], mode), expected.length);
    assert.deepEqual(calls.at(-1)[1].filePaths, expected);
  }
  await assert.rejects(copy([{file_path:'/missing.raw',media_subtype:'raw_jpeg_pair'}], true));
  assert.equal(await copy([{file_path:'/standalone.raw'}], false), 1);
});

test('clipboard copy reaches policy validation even when the RAW primary is unavailable', () => {
  const source = readFileSync(new URL('../src/common/availability.ts', import.meta.url), 'utf8');
  const functions = source.slice(source.indexOf('const originalActions ='))
    .replace('export function requiresOriginalAction', 'function requiresOriginalAction');
  const requiresOriginal = new Function(ts.transpile(functions, {target: ts.ScriptTarget.ES2022})
    + ';return requiresOriginalAction;')();
  assert.equal(requiresOriginal('copy'), false);
  for (const action of ['move-within-library', 'move-to-folder', 'copy-to-folder', 'trash', 'rename']) {
    assert.equal(requiresOriginal(action), true);
  }
});
