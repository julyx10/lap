import ts from 'typescript';
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { compile, computed, createSSRApp, ref, reactive } from 'vue';
import { renderToString } from 'vue/server-renderer';

const source = await readFile(new URL('../src/components/FileInfo.vue', import.meta.url), 'utf8');

test('title and headline display stored values, including empty values, instead of lazy metadata', () => {
  const start = source.indexOf('const sourceKeys =');
  const end = source.indexOf('const sourceDetailRows', start);
  const rows = fileInfo => new Function('props', 'metadataDetails', 't', ts.transpile(
    `${source.slice(start, end)}; return detailRows(['title', 'headline']);`, { target: ts.ScriptTarget.ES2022 }
  ))({ fileInfo }, { value: { title: { values: ['stale title'] }, headline: { values: ['stale headline'] } } }, key => key);
  assert.deepEqual(rows({ e_title: 'Stored title', e_headline: 'Stored headline' }).map(row => row.value), ['Stored title', 'Stored headline']);
  assert.deepEqual(rows({ e_title: null, e_headline: '' }), []);
});
function section(key) {
  const title = source.indexOf(`$t('file_info.${key}')`);
  assert.ok(title > 0);
  const start = source.lastIndexOf('      <div ', title);
  const end = source.indexOf('\n      </div>', title) + '\n      </div>'.length;
  assert.ok(start >= 0 && end > title);
  return source.slice(start, end);
}
async function render(key, overrides = {}) {
  const context = {
    detailSections: {source:{loading:false}},
    sourceDetailRows: [{ key: "description", value: "ORIGINAL SOURCE TEXT", source: "EXIF", label: "file_info.detail_fields.description" }], detailsLoading: false, detailsError: false,
    showOrganizationPanel: true, showSourceMetadataPanel: false, hasSourceMetadata: true,
    toggleOrganization() {}, toggleSourceMetadata() {}, emit() {},
    isPrimaryGeneralInfo: false, displayTags: [], fileCollections: [], filePersons: [],
    config: { settings: { face: { enabled: false } } },
    fileInfo: { comments: '', e_description: 'ORIGINAL SOURCE TEXT' }, ...overrides,
  };
  const app = createSSRApp({ data: () => context, render: compile(section(key)) });
  app.config.globalProperties.$t = key => key;
  for (const name of ['IconRight', 'FavoriteRatingControl', 'TButton', 'IconRotate', 'IconCollection', 'IconPerson']) {
    app.component(name, { render: () => null });
  }
  return renderToString(app);
}

test('organization keeps catalog controls on the companion tab and hides empty descriptions', async () => {
  const html = await render('organization');
  assert.doesNotMatch(html, /file_info.marks/);
  assert.doesNotMatch(html, /file_info.comment/);
  assert.doesNotMatch(html, /<template[\s>]/, 'description must be visible DOM, not inert template content');
  assert.doesNotMatch(html, /—/);
  assert.doesNotMatch(html, /<button[^>]*text-left/);
  assert.doesNotMatch(html, /ORIGINAL SOURCE TEXT/);
  const filled = await render('organization', { fileInfo: { comments: 'USER DESCRIPTION' } });
  assert.match(filled, /file_info.comment/);
  assert.match(filled, /USER DESCRIPTION/);
});

test('source metadata is hidden until expanded and retains the original description', async () => {
  assert.doesNotMatch(await render('source_metadata'), /ORIGINAL SOURCE TEXT/);
  const expanded = await render('source_metadata', { showSourceMetadataPanel: true });
  assert.match(expanded, /ORIGINAL SOURCE TEXT/);
  assert.match(expanded, /file_info.detail_fields.description/);
});

test('location accepts zero coordinates and rejects missing or invalid coordinates', () => {
  const start = source.indexOf('const hasCoordinates = computed(');
  const end = source.indexOf('const cameraText', start);
  assert.ok(start >= 0 && end > start);
  const valid = fileInfo => new Function('computed', 'props', `${source.slice(start, end)}; return hasCoordinates.value;`)(computed, { fileInfo });
  for (const [lat, lon, expected] of [[0,0,true], ['0','120',true], [null,0,false], ['',0,false], [91,0,false], [0,181,false], ['bad',0,false]]) {
    assert.equal(valid({ gps_latitude: lat, gps_longitude: lon }), expected);
  }
});


test('capture and location sections are hidden only when all their displayed data is empty', () => {
  const start = source.indexOf('const hasCaptureInfo = computed(');
  const end = source.indexOf('const showBasicInfoPanel', start);
  assert.ok(start >= 0 && end > start);
  const evaluate = (fileInfo, coordinates = false, location = '', state = { loaded: true }) => new Function(
    'computed', 'props', 'hasCoordinates', 'formatGeoLocation', 'hasAltitude', 'detailsLoaded', 'captureDetailRows', 'locationDetailRows', 'sourceDetailRows', 'detailSections', 'cameraText', 'exposureText', 'captureTimeText',
    `${source.slice(start, end)}; return [hasCaptureInfo.value, hasLocationInfo.value];`,
  )(computed, { fileInfo }, { value: coordinates }, () => location, { value: fileInfo?.gps_altitude != null }, { value: true }, { value: [] }, { value: [] }, { value: [] }, {value:{capture:state,location:state,source:state}}, {value: fileInfo.e_make || fileInfo.e_model || ''}, {value: ['e_focal_length','e_exposure_time','e_f_number','e_iso_speed','e_exposure_bias'].map(k=>fileInfo[k] || '').join('').trim()}, {value: fileInfo.e_date_time && fileInfo.taken_date ? 'date' : ''});
  assert.deepEqual(evaluate({}), [false, false]);
  assert.deepEqual(evaluate({}, false, '', { loaded: true, loading: true }), [false, false], 'refreshing empty sections must not insert temporary headers');
  assert.deepEqual(evaluate({}, false, '', { loaded: true, error: true }), [false, false]);
  assert.deepEqual(evaluate({ e_make: '  ', e_model: null }), [false, false]);
  for (const key of ['e_make', 'e_model', 'e_lens_model', 'e_focal_length', 'e_exposure_time', 'e_f_number', 'e_iso_speed', 'e_exposure_bias']) {
    assert.equal(evaluate({ [key]: 'value' })[0], true);
  }
  assert.equal(evaluate({ e_date_time: '2026:01:01', taken_date: 1767225600 })[0], true);
  assert.deepEqual(evaluate({}, true), [false, true]);
  assert.deepEqual(evaluate({ gps_altitude: 0 }), [false, true]);
  assert.deepEqual(evaluate({}, false, 'Beijing'), [false, true]);
  assert.deepEqual(evaluate({}, false, '  '), [false, false]);
  assert.match(source, /<div v-if="hasCaptureInfo"/);
  assert.match(source, /<div v-if="hasLocationInfo"/);
});


test('GPS display uses DMS and hemisphere letters with correct rounding at boundaries', async () => {
  const utils = await readFile(new URL('../src/common/utils.ts', import.meta.url), 'utf8');
  const start = utils.indexOf('export function formatGpsCoordinates(');
  const end = utils.indexOf('/// format camera', start);
  assert.ok(start >= 0 && end > start);
  const code = ts.transpile(utils.slice(start, end).replace('export ', ''), { target: ts.ScriptTarget.ES2022 });
  const format = new Function(`${code}; return formatGpsCoordinates;`)();
  assert.equal(format(39.9042, 116.4074), '39°54′15.12″ N, 116°24′26.64″ E');
  assert.equal(format(-33.8688, -151.2093), '33°52′07.68″ S, 151°12′33.48″ W');
  assert.equal(format(0, 0), '0°00′00.00″ N, 0°00′00.00″ E');
  assert.equal(format(89.99999999, 179.99999999), '90°00′00.00″ N, 180°00′00.00″ E');
  for (const pair of [[null,0], ['',0], [' ',0], [91,0], [0,181], ['bad',0]]) assert.equal(format(...pair), '');
});


test('source metadata section is absent when all source fields are empty', async () => {
  assert.doesNotMatch(await render('source_metadata', { hasSourceMetadata: false }), /file_info.source_metadata/);
  const start = source.indexOf('const hasSourceMetadata = computed(');
  const end = source.indexOf('const showBasicInfoPanel', start);
  assert.ok(start >= 0 && end > start);
  const hasData = (loaded, rows) => new Function('computed', 'detailsLoaded', 'sourceDetailRows', 'detailSections', 'showSourceMetadataPanel', `${source.slice(start, end)}; return hasSourceMetadata.value;`)(computed, {value:loaded}, {value:rows}, {value:{source:{loaded}}}, {value:false});
  assert.equal(hasData(false, []), true);
  assert.equal(hasData(true, []), false);
  assert.equal(hasData(true, [{value:'Author'}]), true);
});


test('altitude accepts sea level and negative elevations but not missing or invalid values', () => {
  const start = source.indexOf('const hasAltitude = computed(');
  const end = source.indexOf('const cameraText', start);
  assert.ok(start >= 0 && end > start);
  const valid = altitude => new Function('computed', 'props', `${source.slice(start, end)}; return hasAltitude.value;`)(computed, { fileInfo: { gps_altitude: altitude } });
  for (const altitude of [0, -12.5, 1234, '0']) assert.equal(valid(altitude), true);
  for (const altitude of [null, undefined, '', ' ', NaN, Infinity, 'bad']) assert.equal(valid(altitude), false);
});

function detailsFixture(file = { id: 1, file_path: '/1.jpg', e_title: 'DB title' }) {
  const props = reactive({ fileInfo: file });
  const pending = [], calls = [], timers = [];
  const deps = {
    props, ref, computed, libConfig: { _libraryId: 'test' }, t: key => key,
    isOriginalUnavailable: file => file?.album_accessible === false,
    showSourceMetadataPanel: ref(true), showMetadataPanel: ref(true), showMapPanel: ref(false),
    generalFileInfo: ref({}), refreshingFile: ref(null),
    activeInfoKey: { get value() { return `test:${props.fileInfo.id}`; } },
    setTimeout: callback => { const timer = { callback }; timers.push(timer); return timer; },
    clearTimeout: timer => { if (timer) timer.cancelled = true; },
    invoke: (command, args) => {
      calls.push({ command, args });
      return command === 'cancel_file_metadata_requests' ? Promise.resolve() : new Promise((resolve, reject) => pending.push({ args, resolve, reject }));
    },
  };
  const a = source.indexOf('type MetadataDetail =');
  const b = source.indexOf('const LoadingIcon', a);
  const w = source.indexOf('watch(() => [libConfig._libraryId, props.fileInfo?.id, props.fileInfo?.file_path');
  const c = source.indexOf('  const sameFile', w);
  const d = source.indexOf('}, { immediate: true });', c);
  assert.ok(a >= 0 && b > a && c > w && d > c);
  const body = ts.transpile(`${source.slice(a, b)}
    function changeFile(values, previous) { ${source.slice(c, d)} }
    return { changeFile, infoReady, metadataDetails, detailSections, loadMetadataDetails, sourceDetailRows };
  `, { target: ts.ScriptTarget.ES2022 });
  const state = new Function(...Object.keys(deps), body)(...Object.values(deps));
  const select = id => {
    const previous = ['test', props.fileInfo.id];
    props.fileInfo = { id, file_path: `/${id}.jpg`, e_title: `DB ${id}` };
    state.changeFile(['test', id], previous);
  };
  return { props, pending, calls, timers, state, select };
}
const settle = () => new Promise(resolve => setImmediate(resolve));

test('uncached file waits for all expanded sections and publishes successful fields with DB fallback', async () => {
  const f = detailsFixture();
  f.state.changeFile(['test', 1], undefined);
  assert.equal(f.pending.length, 2);
  assert.equal(f.state.infoReady.value, false);
  f.pending[0].resolve({ credit: { values: ['credit'], source: 'XMP' } });
  await settle();
  assert.equal(f.state.infoReady.value, false);
  assert.deepEqual(f.state.metadataDetails.value, {});
  f.pending[1].reject(new Error('read failed'));
  await settle();
  assert.equal(f.state.infoReady.value, true);
  assert.equal(f.state.metadataDetails.value.credit.values[0], 'credit');
  assert.ok(f.state.sourceDetailRows.value.some(row => row.value === 'DB title'));
});

test('offline albums and cloud-only originals show DB immediately without any file-detail IPC', () => {
  for (const flags of [{ album_accessible: false }, { cloud_only: true }]) {
    const f = detailsFixture({ id: 1, e_title: 'DB title', ...flags });
    f.state.changeFile(['test', 1], undefined);
    assert.equal(f.state.infoReady.value, true);
    assert.equal(f.calls.length, 0);
    assert.deepEqual(f.state.metadataDetails.value, {});
  }
});

test('cached files show immediately and late replies cannot publish a previous file', async () => {
  const f = detailsFixture();
  f.state.changeFile(['test', 1], undefined);
  f.pending[0].resolve({ credit: { values: ['cached'], source: 'XMP' } });
  f.pending[1].resolve({});
  await settle();
  f.select(2);
  assert.equal(f.state.infoReady.value, false);
  f.select(1);
  assert.equal(f.state.infoReady.value, true);
  assert.equal(f.state.metadataDetails.value.credit.values[0], 'cached');
  f.pending[2].resolve({ credit: { values: ['stale'], source: 'XMP' } });
  f.pending[3].resolve({});
  f.pending[4].resolve({ credit: { values: ['cached'], source: 'XMP' } });
  f.pending[5].resolve({});
  await settle();
  assert.equal(f.state.metadataDetails.value.credit.values[0], 'cached');
});

test('timeout cancels reads and releases the complete DB-only panel', async () => {
  const f = detailsFixture();
  f.state.changeFile(['test', 1], undefined);
  for (const timer of f.timers) timer.callback();
  await settle();
  assert.equal(f.state.infoReady.value, true);
  assert.deepEqual(f.state.metadataDetails.value, {});
  assert.equal(f.calls.filter(call => call.command === 'cancel_file_metadata_requests').length, 2);
});

test('empty-file message is shown only when the parent confirms the new view is empty', async () => {
  const content = await readFile(new URL('../src/components/Content.vue', import.meta.url), 'utf8');
  assert.match(content, /:empty="hasConfirmedEmptyContent"/);
  const start = source.indexOf('<div v-else-if="empty"');
  const end = source.indexOf('\n  </div>\n</template>', start);
  assert.ok(start >= 0 && end > start);
  const template = source.slice(start, end).replace('v-else-if="empty"', 'v-if="empty"');
  for (const empty of [false, true]) {
    const app = createSSRApp({ data: () => ({ empty }), render: compile(template) });
    app.config.globalProperties.$t = key => key;
    app.component('IconFile', { render: () => null });
    const html = await renderToString(app);
    assert.equal(html.includes('file_info.empty_title'), empty);
  }
});
