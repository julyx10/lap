import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';

// The location breadcrumb navigates a country > region > place title. Extract
// the sidebar branch from Content.vue and run it against the three shapes the
// selection can take.
const source = fs.readFileSync(new URL('../src/components/Content.vue', import.meta.url), 'utf8');
const start = source.indexOf('// Location: segments navigate');
const end = source.indexOf('// Camera: clicking parent segment', start);
assert.ok(start >= 0 && end > start, 'location breadcrumb branch not found in Content.vue');
const code = source.slice(start, end);

const SIDEBAR = { LOCATION: 4 };

function navigate(segmentIndex, selection) {
  const libConfig = { location: { ...selection } };
  new Function('SIDEBAR', 'libConfig', 'sidebarIndex', 'segmentIndex', code)(
    SIDEBAR, libConfig, SIDEBAR.LOCATION, segmentIndex,
  );
  return libConfig.location;
}

test('the country segment drops both the region and the place', () => {
  assert.deepEqual(
    navigate(0, { cc: 'US', admin1: 'California', name: 'San Francisco' }),
    { cc: 'US', admin1: null, name: null },
  );
});

test('the region segment drops only the place', () => {
  assert.deepEqual(
    navigate(1, { cc: 'US', admin1: 'California', name: 'San Francisco' }),
    { cc: 'US', admin1: 'California', name: null },
  );
});

test('a selection without a country code keeps navigating to the region', () => {
  assert.deepEqual(
    navigate(0, { cc: null, admin1: 'California', name: 'San Francisco' }),
    { cc: null, admin1: 'California', name: null },
  );
});

test('Hong Kong country breadcrumb clears the skipped region and place', () => {
  assert.deepEqual(
    navigate(0, { cc: 'HK', admin1: '', name: 'Hong Kong' }),
    { cc: 'HK', admin1: null, name: null },
  );
});

import { buildCountryChildren } from '../src/common/locationTree.js';

test('regions and direct places sort together by name and count', () => {
  const regions = [
    { admin1: '', names: ['Z', 'B', ''], counts: [2, 9, 3] },
    { admin1: 'A', names: ['City'], counts: [5] },
  ];
  assert.deepEqual(buildCountryChildren(regions, 0, 'en').map(n => n.name), ['A', 'B', 'Z']);
  assert.deepEqual(buildCountryChildren(regions, 1, 'en').map(n => n.name), ['Z', 'B', 'A']);
  assert.deepEqual(buildCountryChildren(regions, 2, 'en').map(n => n.name), ['Z', 'A', 'B']);
  assert.deepEqual(buildCountryChildren(regions, 3, 'en').map(n => n.name), ['B', 'A', 'Z']);
});

test('country-only records have no expandable children', () => {
  assert.deepEqual(buildCountryChildren([{ admin1: '', names: [''], counts: [4] }], 0, 'en'), []);
});

const locationSource = fs.readFileSync(new URL('../src/components/Location.vue', import.meta.url), 'utf8');
const restoreCode = locationSource.slice(locationSource.indexOf('function restoreLocationSelection()'),
  locationSource.indexOf('/// click location icon'))
  .replaceAll(': any', '').replaceAll(' as any', '');

function restoreSelection(selection, regions) {
  const libConfig = { location: { ...selection } };
  new Function('libConfig', 'locations', 'expandedCountries', restoreCode + ';restoreLocationSelection();')(
    libConfig, { value: regions }, { value: new Set() },
  );
  return libConfig.location;
}

test('a missing direct place falls back to the country without an empty-region filter', () => {
  assert.deepEqual(restoreSelection({ cc: 'HK', admin1: '', name: 'Hong Kong' },
    [{ cc: 'HK', admin1: '', names: [''], counts: [1] }]),
  { cc: 'HK', admin1: null, name: null });
});

test('a missing place with a known region falls back to that region', () => {
  assert.deepEqual(restoreSelection({ cc: 'US', admin1: 'California', name: 'San Francisco' },
    [{ cc: 'US', admin1: 'California', names: ['Los Angeles'], counts: [1] }]),
  { cc: 'US', admin1: 'California', name: null });
});

test('an existing direct place keeps its exact selection', () => {
  assert.deepEqual(restoreSelection({ cc: 'HK', admin1: '', name: 'Hong Kong' },
    [{ cc: 'HK', admin1: '', names: ['Hong Kong'], counts: [1] }]),
  { cc: 'HK', admin1: '', name: 'Hong Kong' });
});
