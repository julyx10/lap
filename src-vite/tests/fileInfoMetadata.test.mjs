import ts from 'typescript';
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { compile, computed, createSSRApp, ref, reactive } from 'vue';
import { renderToString } from 'vue/server-renderer';

const source = await readFile(new URL('../src/components/FileInfo.vue', import.meta.url), 'utf8');

test('searchable source metadata displays the indexed values, including empty values', () => {
  const start = source.indexOf('const sourceKeys =');
  const end = source.indexOf('const sourceDetailRows', start);
  const keys = ['title', 'headline', 'creator', 'copyright', 'software'];
  const fields = ['e_title', 'e_headline', 'e_artist', 'e_copyright', 'e_software'];
  const details = Object.fromEntries(keys.map(key => [key, { values: [`stale ${key}`] }]));
  const rows = fileInfo => new Function('props', 'metadataDetails', 't', 'fieldVisible', 'displayField', ts.transpile(
    `${source.slice(start, end)}; return detailRows('source', ${JSON.stringify(keys)});`, { target: ts.ScriptTarget.ES2022 }
  ))({ fileInfo }, { value: details }, key => key, () => true, () => true);
  assert.deepEqual(rows(Object.fromEntries(fields.map((field, index) => [field, keys[index]]))).map(row => row.value), keys);
  assert.deepEqual(rows({}).map(row => row.value), keys.map(() => ''));
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
    sourceDetailRows: [{ key: "software", value: "SOURCE SOFTWARE", source: "EXIF", label: "file_info.detail_fields.software" }], detailsLoading: false, detailsError: false,
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

test('source metadata is hidden until expanded and displays source software', async () => {
  assert.doesNotMatch(await render('source_metadata'), /SOURCE SOFTWARE/);
  const expanded = await render('source_metadata', { showSourceMetadataPanel: true });
  assert.match(expanded, /SOURCE SOFTWARE/);
  assert.match(expanded, /file_info.detail_fields.software/);
  assert.doesNotMatch(source, /description: 'e_description'/);
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


test('section visibility follows field choices even when values are empty', () => {
  const start = source.indexOf('const hasCaptureInfo = computed(');
  const end = source.indexOf('const showBasicInfoPanel', start);
  const evaluate = selected => new Function('computed', 'hasVisibleFields', 'hideEmptyFields',
    `${source.slice(start, end)}; return [hasCaptureInfo.value, hasLocationInfo.value, hasSourceMetadata.value];`
  )(computed, section => selected.includes(section), { value: false });
  assert.deepEqual(evaluate([]), [false, false, false]);
  assert.deepEqual(evaluate(['capture', 'source']), [true, false, true]);
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


test('source metadata section is absent when no source fields are selected', async () => {
  assert.doesNotMatch(await render('source_metadata', { hasSourceMetadata: false }), /file_info.source_metadata/);
});


test('altitude accepts sea level and negative elevations but not missing or invalid values', () => {
  const start = source.indexOf('const hasAltitude = computed(');
  const end = source.indexOf('const cameraText', start);
  assert.ok(start >= 0 && end > start);
  const valid = altitude => new Function('computed', 'props', `${source.slice(start, end)}; return hasAltitude.value;`)(computed, { fileInfo: { gps_altitude: altitude } });
  for (const altitude of [0, -12.5, 1234, '0']) assert.equal(valid(altitude), true);
  for (const altitude of [null, undefined, '', ' ', NaN, Infinity, 'bad']) assert.equal(valid(altitude), false);
});

function detailsFixture(file = { id: 1, file_path: '/1.jpg', e_title: 'DB title' }, fieldVisible = () => true) {
  const props = reactive({ fileInfo: file });
  const pending = [], calls = [], timers = [];
  const deps = {
    props, ref, computed, fieldVisible, displayField: key => fieldVisible(key), libConfig: { _libraryId: 'test' }, t: key => key,
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


test('DB-only field choices do not start realtime metadata reads', () => {
  const f = detailsFixture(undefined, key => ['source.title', 'source.headline', 'capture.camera'].includes(key));
  f.state.changeFile(['test', 1], undefined);
  assert.equal(f.calls.length, 0);
  assert.equal(f.state.infoReady.value, true);
  assert.equal(f.state.sourceDetailRows.value[0].value, 'DB title');
});

test('field choices allow no fields and reject unknown fields', async () => {
  const registry = await readFile(new URL('../src/common/fileInfoFields.ts', import.meta.url), 'utf8');
  const module = await import(`data:text/javascript;base64,${Buffer.from(ts.transpile(registry, {target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext})).toString('base64')}`);
  assert.deepEqual(module.normalizeFileInfoFields([]), []);
  assert.equal(module.allFileInfoFields.includes('source.description'), false);
  assert.deepEqual(module.normalizeFileInfoFields(['unknown', 'source.title', 'source.title']), ['source.title']);
  assert.deepEqual(module.normalizeFileInfoFields(undefined), module.normalizeFileInfoFields(module.defaultFileInfoFields));
  assert.equal(module.normalizeFileInfoFields(module.defaultFileInfoFields).length, module.defaultFileInfoFields.length);
  for (const locale of ['en','zh-CN','zh-TW','de','es','fr','hu','it','ja','ko','nl','pl','pt','ru','uk']) {
    const messages = JSON.parse(await readFile(new URL(`../src/locales/${locale}.json`, import.meta.url), 'utf8'));
    for (const group of module.fileInfoFieldGroups) {
      for (const key of [group.label, ...group.fields.map(field => field.label)].filter(key => key !== 'GPS')) {
        assert.equal(typeof key.split('.').reduce((object, part) => object?.[part], messages), 'string', `${locale}: ${key}`);
      }
    }
  }
});


test('auto-hide removes only empty fields, retaining zero values', () => {
  const start = source.indexOf('const hideEmptyFields =');
  const end = source.indexOf('const hasVisibleFields =', start);
  const config = reactive({ infoPanel: { hideEmptyFields: false } });
  const display = new Function('computed', 'config', 'fieldVisible', `${ts.transpile(source.slice(start, end))}; return displayField;`)(computed, config, key => key !== 'hidden');
  assert.equal(display('file.size', null), true);
  config.infoPanel.hideEmptyFields = true;
  for (const value of [null, undefined, '', '  ']) assert.equal(display('file.size', value), false);
  for (const value of [0, '0', 'text']) assert.equal(display('file.size', value), true);
  assert.equal(display('hidden', 'text'), false);
  assert.match(source, /v-if="displayField\('file.name'/);
  assert.match(source, /v-if="hasFileInfo"/);
});

test('auto-hide removes empty sections while retaining sections with selected content', () => {
  const start = source.indexOf('const hasCaptureInfo =');
  const end = source.indexOf('const showBasicInfoPanel', start);
  const selected = new Set(['capture.camera', 'location.gps', 'source.title']);
  const deps = {
    computed, hideEmptyFields: { value: true },
    displayField: (key, value) => selected.has(key) && value != null && String(value).trim() !== '',
    fieldVisible: key => selected.has(key), hasVisibleFields: () => true,
    cameraText: ref(''), captureTimeText: ref(''), exposureText: ref(''), props: { fileInfo: {} },
    captureDetailRows: ref([]), locationDetailRows: ref([]), sourceDetailRows: ref([]),
    hasCoordinates: ref(false), hasAltitude: ref(false), formatGeoLocation: () => '',
  };
  const sections = new Function(...Object.keys(deps), `${source.slice(start, end)}; return [hasCaptureInfo, hasLocationInfo, hasSourceMetadata];`)(...Object.values(deps));
  assert.deepEqual(sections.map(section => section.value), [false, false, false]);
  deps.cameraText.value = 'Nikon';
  deps.hasAltitude.value = true;
  deps.sourceDetailRows.value = [{ key: 'title', value: 'Title' }];
  assert.deepEqual(sections.map(section => section.value), [true, true, true]);
});


test('File section follows rendered fields including auto-hide and video-only duration', () => {
  const start = source.indexOf('const hasFileInfo =');
  const end = source.indexOf('const hasCaptureInfo =', start);
  const generalFileInfo = ref({ file_type: 1 });
  const imageDetailRows = ref([]);
  const selected = new Set(['file.iccProfile']);
  let hideEmpty = true;
  const displayField = (key, value) => selected.has(key) && (!hideEmpty || (value != null && String(value).trim() !== ''));
  const hasInfo = new Function('computed', 'generalFileInfo', 'generalFolderBreadcrumbs', 'imageDetailRows', 'displayField',
    `${source.slice(start, end)}; return hasFileInfo;`)(computed, generalFileInfo, ref([]), imageDetailRows, displayField);
  assert.equal(hasInfo.value, false, 'missing selected ICC profile must not leave an empty section');
  imageDetailRows.value = [{ key: 'iccProfile', value: 'sRGB' }];
  assert.equal(hasInfo.value, true);
  imageDetailRows.value = [];
  selected.clear();
  selected.add('file.duration');
  generalFileInfo.value = { file_type: 1, duration: 15 };
  assert.equal(hasInfo.value, false, 'duration does not apply to images');
  generalFileInfo.value = { file_type: 2, duration: 15 };
  assert.equal(hasInfo.value, true);
  generalFileInfo.value = { file_type: 2 };
  assert.equal(hasInfo.value, false, 'empty video duration is hidden');
  hideEmpty = false;
  generalFileInfo.value = { file_type: 2 };
  assert.equal(hasInfo.value, true, 'empty video duration remains visible with auto-hide disabled');
  generalFileInfo.value = { file_type: 1 };
  assert.equal(hasInfo.value, false, 'image duration stays hidden even with auto-hide disabled');
  selected.clear();
  generalFileInfo.value = { file_type: 1, name: 'image.jpg' };
  assert.equal(hasInfo.value, false, 'no selected fields hides the section');
  selected.add('file.size');
  hideEmpty = true;
  generalFileInfo.value = { file_type: 1, size: 0 };
  assert.equal(hasInfo.value, true, 'zero bytes is a valid value');
});
