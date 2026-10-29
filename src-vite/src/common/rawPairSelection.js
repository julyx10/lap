// Only map a hidden companion when its primary belongs to the current query.
// If both IDs were selected, Set deduplicates them without losing focus.
export function mapVisibleRawPairSelection(selectedIds, availableIds, mapping) {
  const mapId = id => availableIds.has(id) ? id : Number(mapping?.[String(id)] || 0);
  const selected = new Set(Array.from(selectedIds, mapId).filter(id => availableIds.has(id)));
  return { selected, mapId };
}
