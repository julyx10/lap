// Regions and places without a region share one visible sibling list.
export function buildCountryChildren(regions, sort, locale) {
  const children = [];
  for (const location of regions) {
    if (location.admin1) {
      children.push({ kind: 'region', key: JSON.stringify(['region', location.admin1]),
        name: location.admin1, count: location.counts.reduce((sum, count) => sum + count, 0), location });
    } else {
      location.names.forEach((name, index) => {
        if (name) children.push({ kind: 'place', key: JSON.stringify(['place', name]),
          name, count: location.counts[index], location });
      });
    }
  }
  return children.sort((a, b) => {
    const label = a.name.localeCompare(b.name, locale);
    if (sort === 2) return a.count - b.count || label;
    if (sort === 3) return b.count - a.count || label;
    return sort === 1 ? -label : label;
  });
}
