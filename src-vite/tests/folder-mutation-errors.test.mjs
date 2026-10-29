import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import vm from 'node:vm';

const api = await readFile(new URL('../src/common/api.js', import.meta.url), 'utf8');
const component = await readFile(new URL('../src/components/AlbumFolder.vue', import.meta.url), 'utf8');
const extract = name => api.match(new RegExp(`export async function ${name}\\([^]*?\\n}`))[0].replace('export ', '');

test('folder API preserves registration errors and null failures', async () => {
  const context = vm.createContext({ invoke: async () => { throw 'scan busy'; }, console });
  const create = vm.runInContext(`(${extract('createFolder')})`, context);
  const rename = vm.runInContext(`(${extract('renameFolder')})`, context);
  await assert.rejects(create('/album', 'new'), error => error === 'scan busy');
  await assert.rejects(rename('/album/old', 'new'), error => error === 'scan busy');
  context.invoke = async () => null;
  assert.equal(await create('/album', 'new'), null);
  assert.equal(await rename('/album/old', 'new'), null);
});

test('transfer busy notices remain visible without changing null-return contract', async () => {
  const messages = [];
  const busy = 'Library scan or folder recovery is in progress; retry the file operation';
  const context = vm.createContext({ invoke: async () => { throw busy; }, config: { settings: {} }, console,
    useToast: () => ({ warning: message => messages.push(message) }) });
  const helper = api.match(/let lastTransferBusyNotice = 0;[\s\S]*?\n}/)[0];
  vm.runInContext(helper, context);
  for (const name of ['moveFile', 'moveFileOutsideLibrary', 'copyFile', 'moveFolder', 'moveFolderOutsideLibrary', 'copyFolder']) {
    const fn = vm.runInContext(`(${extract(name)})`, context);
    assert.equal(await fn('a', 'b', 'c', 'd'), null);
  }
  assert.deepEqual(messages, [busy]);
});

test('folder UI catches and displays operation errors', () => {
  assert.match(component, /await createFolder\(parent\.path, name\)[\s\S]*?catch \(error\)[\s\S]*?toast\.error\(/);
  assert.match(component, /await renameFolder\(oldFolderPath, newFolderName\)[\s\S]*?catch \(error\)[\s\S]*?toast\.error\(String\(error\)\)/);
});
