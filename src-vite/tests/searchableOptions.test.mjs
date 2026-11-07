import test from 'node:test';
import assert from 'node:assert/strict';
import { filterOptions, highlightSegments, optionLeafLabel } from '../src/common/searchableOptions.js';

const tagOptions = [
  { value: 1, label: 'Cake' },
  { value: 2, label: 'Context|Parties|Cake' },
  { value: 3, label: 'Whatever|Cake' },
  { value: 4, label: 'Cakewalk' },
  { value: 5, label: 'Travel' },
];

test('optionLeafLabel returns the last path segment', () => {
  assert.equal(optionLeafLabel('Context|Parties|Cake'), 'Cake');
  assert.equal(optionLeafLabel('Canon › EOS R5'), 'EOS R5');
  assert.equal(optionLeafLabel('Bavaria › Munich'), 'Munich');
  assert.equal(optionLeafLabel('Cake'), 'Cake');
  assert.equal(optionLeafLabel(null), '');
});

test('empty query keeps the source order and respects the limit', () => {
  assert.deepEqual(filterOptions(tagOptions, ''), tagOptions);
  assert.deepEqual(filterOptions(tagOptions, '   '), tagOptions);
  assert.deepEqual(filterOptions(tagOptions, '', 2), tagOptions.slice(0, 2));
});

test('filters case-insensitively, ranked by leaf before label', () => {
  const matches = filterOptions(tagOptions, 'cake');
  assert.deepEqual(matches.map(option => option.value), [1, 2, 3, 4]);
  assert.equal(optionLeafLabel(matches[0].label), 'Cake');

  assert.deepEqual(filterOptions(tagOptions, 'PARTIES').map(option => option.value), [2]);
  assert.deepEqual(filterOptions(tagOptions, 'travel').map(option => option.value), [5]);
  assert.deepEqual(filterOptions(tagOptions, 'missing'), []);
});

test('camera and location labels match on the leaf segment', () => {
  const options = [
    { value: 'Canon||', label: 'Canon' },
    { value: 'Canon||EOS R5', label: 'Canon › EOS R5' },
    { value: 'Nikon||Z8', label: 'Nikon › Z8' },
  ];
  assert.deepEqual(filterOptions(options, 'r5').map(option => option.value), ['Canon||EOS R5']);
  assert.deepEqual(filterOptions(options, 'nikon').map(option => option.value), ['Nikon||Z8']);
});

test('limit is applied after ranking', () => {
  const matches = filterOptions(tagOptions, 'cake', 2);
  assert.equal(matches.length, 2);
  assert.deepEqual(matches.map(option => option.value), [1, 2]);
});

test('highlightSegments marks the first case-insensitive occurrence', () => {
  assert.deepEqual(highlightSegments('Context|Parties|Cake', 'parties'), [
    { text: 'Context|', match: false },
    { text: 'Parties', match: true },
    { text: '|Cake', match: false },
  ]);
  assert.deepEqual(highlightSegments('Cake', ''), [{ text: 'Cake', match: false }]);
  assert.deepEqual(highlightSegments('Cake', 'zzz'), [{ text: 'Cake', match: false }]);
});
