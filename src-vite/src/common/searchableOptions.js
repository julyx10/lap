// Pure helpers for the searchable option picker used by smart album rules.
//
// Options are { value, label } pairs. Labels can be hierarchical:
//   - tags:      "Context|Parties|Cake"
//   - cameras:   "Canon › EOS R5"
//   - locations: "Bavaria › Munich"
// The leaf segment is what users usually type, so matching prefers it.

const LEAF_SEPARATORS = [' › ', '|'];

export function optionLeafLabel(label) {
  const text = String(label ?? '');
  let index = -1;
  let separatorLength = 0;
  for (const separator of LEAF_SEPARATORS) {
    const position = text.lastIndexOf(separator);
    if (position > index) {
      index = position;
      separatorLength = separator.length;
    }
  }
  return index >= 0 ? text.slice(index + separatorLength) : text;
}

export function normalizeQuery(query) {
  return String(query ?? '').trim().toLowerCase();
}

// Case-insensitive filter ranked as: exact leaf, leaf prefix, exact label,
// label prefix, leaf contains, label contains. Ties keep the source order so
// backend-provided ordering (name, make, admin region, ...) is preserved.
export function filterOptions(options, query, limit = 100) {
  const text = normalizeQuery(query);
  if (!text) return options.slice(0, limit);

  const ranked = [];
  for (let index = 0; index < options.length; index += 1) {
    const option = options[index];
    const label = String(option?.label ?? '').toLowerCase();
    if (!label.includes(text)) continue;
    const leaf = optionLeafLabel(label).trim();
    const rank = leaf === text
      ? 0
      : leaf.startsWith(text)
        ? 1
        : label === text
          ? 2
          : label.startsWith(text)
            ? 3
            : leaf.includes(text)
              ? 4
              : 5;
    ranked.push({ rank, index, option });
  }

  ranked.sort((a, b) => a.rank - b.rank || a.index - b.index);
  return ranked.slice(0, limit).map(entry => entry.option);
}

// Splits a label into [before, match, after] segments for highlighting.
export function highlightSegments(label, query) {
  const text = normalizeQuery(query);
  const source = String(label ?? '');
  if (!text) return [{ text: source, match: false }];

  const position = source.toLowerCase().indexOf(text);
  if (position < 0) return [{ text: source, match: false }];

  const segments = [];
  if (position > 0) segments.push({ text: source.slice(0, position), match: false });
  segments.push({ text: source.slice(position, position + text.length), match: true });
  if (position + text.length < source.length) {
    segments.push({ text: source.slice(position + text.length), match: false });
  }
  return segments;
}
