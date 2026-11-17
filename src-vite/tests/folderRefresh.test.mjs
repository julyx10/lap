import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import ts from 'typescript';

const source = readFileSync(new URL('../src/components/AlbumFolder.vue', import.meta.url), 'utf8');
const refreshSource = source.slice(source.indexOf('const refreshFolders ='), source.indexOf('/// click expand icon'));

for (const [description, changed, treeChanged, refreshError] of [
  ['skips reloading unchanged files', false, false],
  ['reloads changed files', true, false],
  ['reloads after removing a subfolder', true, true],
  ['reloads partial changes before reporting failure', true, true, 'Folder became inaccessible'],
]) {
  test(`folder refresh ${description}`, async () => {
    const calls = [];
    const folder = { path: 'E:/Photos' };
    const children = [{ path: 'E:/Photos/Trips' }];
    const deps = {
      props: { albumId: 2 },
      libConfig: { _libraryId: 'library' },
      disposed: false,
      refreshingSubfolderPaths: { value: new Set() },
      refreshAlbumFolders: async (...args) => {
        calls.push(args);
        return {
          dirty_folder_count: Number(changed),
          deleted_folder_count: Number(treeChanged && !refreshError),
          folder_path_migrations: [],
          refresh_error: refreshError,
        };
      },
      tauriEmit: async (...args) => calls.push(args),
      fetchFolder: async () => ({ has_subfolders: true, children }),
      config: { settings: {} },
      localeMsg: { value: { tooltip: { refresh_folders: {} } } },
      console: { error: (_message, error) => assert.equal(error.message, refreshError) },
      toast: { success: () => calls.push('success'), error: () => calls.push('failure') },
    };
    const refresh = new Function(...Object.keys(deps), ts.transpile(refreshSource, {
      module: ts.ModuleKind.None,
    }) + '\nreturn refreshFolders;')(...Object.values(deps));

    await refresh(folder);
    assert.deepEqual(folder.children, children);
    assert.equal(folder.has_subfolders, true);
    assert.deepEqual(calls, changed
      ? [[2, folder.path, 'library'], ['refresh-content', {
        libraryId: 'library', albumId: 2, folderPath: folder.path, treeChanged,
      }], refreshError ? 'failure' : 'success']
      : [[2, folder.path, 'library'], 'success']);
  });
}
