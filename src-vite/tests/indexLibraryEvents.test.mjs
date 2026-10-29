import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const api = readFileSync(new URL('../src/common/api.js', import.meta.url), 'utf8');
const helper = api.slice(api.indexOf('function listenCurrentLibraryIndexEvent('), api.indexOf('// deduplication'));
const finished = api.slice(api.indexOf('export async function listenIndexFinished('), api.indexOf('// index faces'));
const source = (helper + finished).replace(/^export /gm, '');

function createListeners() {
  const listeners = new Map();
  const libConfig = { _libraryId: 'library-a' };
  const listen = async (name, callback) => {
    listeners.set(name, callback);
    return () => listeners.delete(name);
  };
  const api = new Function('listen', 'libConfig', `${source}; return { listenIndexProgress, listenIndexFinished };`)(listen, libConfig);
  return { listeners, libConfig, api };
}

test('index events follow the current library and accept legacy payloads', async () => {
  const { listeners, libConfig, api } = createListeners();
  for (const [method, name] of [['listenIndexProgress', 'index_progress'], ['listenIndexFinished', 'index_finished']]) {
    const received = [];
    const unlisten = await api[method](event => received.push(event));
    const dispatch = payload => listeners.get(name)({ payload });
    dispatch({ library_id: 'library-b', album_id: 1 });
    assert.equal(received.length, 0);
    const current = { library_id: 'library-a', album_id: 1 };
    dispatch(current);
    assert.equal(received.at(-1).payload, current);
    dispatch({ libraryId: 'library-a' });
    dispatch({ album_id: 1 });
    dispatch({ library_id: '' });
    dispatch(null);
    assert.equal(received.length, 5);
    libConfig._libraryId = 'library-b';
    dispatch({ library_id: 'library-a' });
    assert.equal(received.length, 5);
    dispatch({ library_id: 'library-b' });
    assert.equal(received.length, 6);
    libConfig._libraryId = 'library-a';
    unlisten();
    assert.equal(listeners.has(name), false);
  }
});

test('home and smart album use the filtered finished listener', () => {
  for (const path of ['../src/views/Home.vue', '../src/components/SmartAlbumList.vue']) {
    const component = readFileSync(new URL(path, import.meta.url), 'utf8');
    assert.match(component, /listenIndexFinished\(/);
    assert.doesNotMatch(component, /listen\(\s*['"]index_finished['"]/);
  }
});
