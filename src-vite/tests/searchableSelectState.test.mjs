import test from 'node:test';
import assert from 'node:assert/strict';
import {
  initialActiveIndex,
  isOptionSelected,
  moveActiveIndex,
  resolveActiveOption,
  resolveDisplayLabel,
  resolveSelectedOption,
} from '../src/common/searchableSelectState.js';

const options = [
  { value: 1, label: 'Cake' },
  { value: 2, label: 'Context|Parties|Cake' },
  { value: 'jpg', label: 'JPG' },
];

test('empty selection shows the placeholder instead of a raw empty label', () => {
  for (const modelValue of [null, undefined, '']) {
    assert.equal(resolveSelectedOption(options, modelValue), null);
    assert.equal(resolveDisplayLabel(null, modelValue, 'Select…'), 'Select…');
    assert.equal(isOptionSelected(1, modelValue), false);
  }
});

test('a selected option resolves by value with string/number coercion', () => {
  assert.deepEqual(resolveSelectedOption(options, 2), options[1]);
  assert.deepEqual(resolveSelectedOption(options, '2'), options[1]);
  assert.equal(resolveDisplayLabel(resolveSelectedOption(options, 'jpg'), 'jpg', 'Select…'), 'JPG');
  assert.equal(isOptionSelected('jpg', 'jpg'), true);
  assert.equal(isOptionSelected(1, 2), false);
});

test('an unknown non-empty value falls back to the raw value', () => {
  assert.equal(resolveSelectedOption(options, 99), null);
  assert.equal(resolveDisplayLabel(null, 99, 'Select…'), '99');
});

test('moving the highlight is clamped to the filtered list', () => {
  assert.equal(moveActiveIndex(0, 1, 3), 1);
  assert.equal(moveActiveIndex(2, 1, 3), 2);
  assert.equal(moveActiveIndex(0, -1, 3), 0);
  assert.equal(moveActiveIndex(0, 1, 0), 0);
});

test('opening the menu highlights the current selection or the first row', () => {
  assert.equal(initialActiveIndex(options, 'jpg'), 2);
  assert.equal(initialActiveIndex(options, 1), 0);
  assert.equal(initialActiveIndex(options, 99), 0);
  assert.equal(initialActiveIndex([], 1), 0);
});

test('committing clamps the highlighted index so Enter always picks a row', () => {
  assert.deepEqual(resolveActiveOption(options, 0), options[0]);
  assert.deepEqual(resolveActiveOption(options, 1), options[1]);
  assert.deepEqual(resolveActiveOption(options, 99), options[2]);
  assert.deepEqual(resolveActiveOption(options, -5), options[0]);
  assert.equal(resolveActiveOption([], 3), null);
});
