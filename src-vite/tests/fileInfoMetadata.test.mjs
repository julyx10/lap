import ts from 'typescript';
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { compile, computed, createSSRApp } from 'vue';
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
  app.component('SmoothHeight', { setup: (_, { slots }) => () => slots.default?.() });
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
  const evaluate = (fileInfo, coordinates = false, location = '') => new Function(
    'computed', 'props', 'hasCoordinates', 'formatGeoLocation', 'hasAltitude', 'detailsLoaded', 'captureDetailRows', 'locationDetailRows', 'sourceDetailRows', 'detailSections', 'cameraText', 'exposureText', 'captureTimeText',
    `${source.slice(start, end)}; return [hasCaptureInfo.value, hasLocationInfo.value];`,
  )(computed, { fileInfo }, { value: coordinates }, () => location, { value: fileInfo?.gps_altitude != null }, { value: true }, { value: [] }, { value: [] }, { value: [] }, {value:{capture:{loaded:true},location:{loaded:true},source:{loaded:true}}}, {value: fileInfo.e_make || fileInfo.e_model || ''}, {value: ['e_focal_length','e_exposure_time','e_f_number','e_iso_speed','e_exposure_bias'].map(k=>fileInfo[k] || '').join('').trim()}, {value: fileInfo.e_date_time && fileInfo.taken_date ? 'date' : ''});
  assert.deepEqual(evaluate({}), [false, false]);
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
