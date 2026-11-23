export interface TagFilterState {
  id: number | null;
  multi: boolean;
  selectedIds: number[];
  matchAny: boolean;
  previousId?: number | null;
}
export function enterTagFilter(state: TagFilterState) {
  if (state.multi) return;
  state.previousId = state.id;
  state.selectedIds = state.id ? [state.id] : [];
  state.matchAny = false;
  state.multi = true;
}
export function selectFilterTag(state: TagFilterState, id: number, modifier = false) {
  if (!state.multi && modifier) enterTagFilter(state);
  if (state.multi) {
    const ids = state.selectedIds || [];
    state.selectedIds = ids.includes(id) ? ids.filter(value => value !== id) : [...ids, id];
  }
  state.id = id;
}
export function exitTagFilter(state: TagFilterState) {
  if (!state.multi) return;
  if (state.previousId !== undefined) state.id = state.previousId;
  state.previousId = null;
  state.multi = false;
  state.selectedIds = [];
}
