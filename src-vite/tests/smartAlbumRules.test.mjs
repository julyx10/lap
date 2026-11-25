import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';

const source = fs.readFileSync(new URL('../src/components/SmartAlbumEdit.vue', import.meta.url), 'utf8');
const section = (start, end) => source.slice(source.indexOf(start), source.indexOf(end, source.indexOf(start)));
const code = ts.transpile([
  section('function getOperatorOptions(', 'function resetRule('),
  section('function isRuleValid(', 'async function addRule('),
  section('function isDateRuleField(', 'function normalizeDateValue('),
  section('function readNumberInput(', 'function scaleValue('),
  section('function dateToUnix(', 'async function loadOptions('),
].join('\n'), { target: ts.ScriptTarget.ES2022 });
const { isRuleValid, getOperatorOptions, readNumberInput, dateInputToUnix, dateInputToUnixExclusiveEnd } = new Function('t', `${code};return {isRuleValid,getOperatorOptions,readNumberInput,dateInputToUnix,dateInputToUnixExclusiveEnd};`)(key => key);
const valid = (field, operator, value) => isRuleValid({ field, operator, value });

test('numeric validation rejects blank, non-finite, negative, fractional and reversed values', () => {
  for (const field of ['width', 'height', 'duration', 'rating', 'size']) {
    for (const value of ['', null, undefined, NaN, Infinity, -1]) assert.equal(valid(field, 'is', value), false, field);
    for (const value of [{}, { min: 1 }, { max: 5 }, { min: '', max: 5 }, { min: 5, max: 1 }]) {
      assert.equal(valid(field, 'between', value), false, `${field}: ${JSON.stringify(value)}`);
    }
    assert.equal(valid(field, 'between', { min: 0, max: 5 }), true);
    assert.equal(valid(field, 'empty', null), true);
    assert.equal(valid(field, 'not_empty', null), true);
  }
  assert.equal(valid('rating', 'is', 6), false);
  assert.equal(valid('rating', 'is', 2.5), false);
  assert.equal(valid('width', 'is', 1.5), false);
  assert.equal(valid('duration', 'is', 1.5), false);
  assert.equal(valid('size', 'is', 1.5), true); // MB may be fractional.
  assert.equal(valid('culling', 'is', 3), false);
  assert.equal(readNumberInput({ target: { value: '', valueAsNumber: NaN } }), '');
  assert.equal(readNumberInput({ target: { value: '0', valueAsNumber: 0 } }), 0);
});

test('new numeric equality and date-on operators coexist with saved period rules', () => {
  for (const field of ['width', 'height', 'duration', 'size']) {
    const options = getOperatorOptions(field).map(option => option.value);
    assert.ok(options.includes('is'));
    assert.ok(options.includes('is_not'));
  }
  for (const field of ['date_taken', 'date_created', 'date_modified']) {
    assert.ok(getOperatorOptions(field).some(option => option.value === 'on'));
    assert.equal(valid(field, 'is', 'this_month'), true);
    assert.equal(valid(field, 'on', { value: 1700000000 }), true);
    assert.equal(valid(field, 'on', { value: NaN }), false);
    assert.equal(valid(field, 'on', { value: dateInputToUnix('') }), false);
    assert.equal(valid(field, 'on', { value: Number.MAX_SAFE_INTEGER }), false);
    assert.equal(valid(field, 'between', { start: 20, end: 10 }), false);
    assert.equal(valid(field, 'between', { start: 10, end: 10 }), false);
    assert.equal(valid(field, 'between', { start: 10, end: 20 }), true);
    assert.equal(valid(field, 'in_last', { amount: 1.5, unit: 'day' }), false);
    assert.equal(valid(field, 'in_last', { amount: 2, unit: 'month' }), true);
  }
});

test('inclusive end dates advance by a local calendar day across DST', () => {
  assert.equal(dateInputToUnixExclusiveEnd(''), null);
  for (const [day, next] of [['2024-03-10', '2024-03-11'], ['2024-11-03', '2024-11-04']]) {
    assert.equal(dateInputToUnixExclusiveEnd(day), dateInputToUnix(next));
  }
});
