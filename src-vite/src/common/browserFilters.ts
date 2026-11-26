import { reactive } from 'vue';

export const browserFilters = reactive({ favorite: -1, rating: -1, culling: -1, tags: -1,
  ratingOp: 'gte' });

export function resetBrowserFilterState() {
  Object.assign(browserFilters, { favorite: -1, rating: -1, culling: -1, tags: -1,
    ratingOp: 'gte' });
}

export type BrowserFilterState = {
  favorite: number;
  rating: number;
  culling: number;
  tags: number;
  ratingOp: string;
};

export function passesBrowserFilters(
  file: any,
  filters: BrowserFilterState = browserFilters,
) {
  if (!file) return false;

  const favorite = Number(filters.favorite);
  if (favorite === 0 || favorite === 1) {
    if (Number(Boolean(file.is_favorite)) !== favorite) return false;
  }

  const ratingFilter = Number(filters.rating);
  const rating = Number(file.rating || 0);
  if (ratingFilter === -2) {
    if (rating <= 0) return false;
  } else if (ratingFilter >= 0 && ratingFilter <= 5) {
    const op = ratingFilter === 0 ? 'eq' : filters.ratingOp;
    if (op === 'eq' && rating !== ratingFilter) return false;
    if (op === 'lte' && rating > ratingFilter) return false;
    if (op === 'gte' && rating < ratingFilter) return false;
  }

  const culling = Number(filters.culling);
  if ((culling === 0 || culling === 1 || culling === 2)
    && Number(file.culling_flag || 0) !== culling) return false;

  const tags = Number(filters.tags);
  if (tags === 0 && Boolean(file.has_tags)) return false;
  if (tags === 1 && !file.has_tags) return false;

  return true;
}
