// Pure interaction helpers for the searchable option picker.
//
// The component keeps only DOM wiring; everything that decides what the control
// shows and which option a key press lands on lives here so it can be tested
// without a browser environment.

function isEmptyValue(value) {
  return value === null || value === undefined || value === '';
}

// An empty model (null, undefined or '') means "nothing selected yet" and must
// show the placeholder instead of an empty label.
export function resolveSelectedOption(options, modelValue) {
  if (isEmptyValue(modelValue)) return null;
  return options.find(option => String(option.value) === String(modelValue)) ?? null;
}

export function isOptionSelected(optionValue, modelValue) {
  if (isEmptyValue(modelValue)) return false;
  return String(optionValue) === String(modelValue);
}

export function resolveDisplayLabel(selectedOption, modelValue, placeholder) {
  if (selectedOption) return selectedOption.label;
  if (!isEmptyValue(modelValue)) return String(modelValue);
  return placeholder;
}

// Moves the highlighted row, clamped to the filtered list bounds.
export function moveActiveIndex(current, delta, optionCount) {
  if (optionCount <= 0) return 0;
  return Math.min(Math.max(current + delta, 0), optionCount - 1);
}

// When the menu opens, highlight the current selection (or the first row).
export function initialActiveIndex(options, modelValue) {
  const index = options.findIndex(option => isOptionSelected(option.value, modelValue));
  return index >= 0 ? index : 0;
}

// Option a key press should commit; clamps so Enter never falls off the list.
export function resolveActiveOption(options, index) {
  if (options.length === 0) return null;
  const bounded = Math.min(Math.max(index, 0), options.length - 1);
  return options[bounded];
}
