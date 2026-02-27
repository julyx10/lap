export interface SmartTagDef {
  id: string;
  // CLIP text prompt should stay in English for stable semantic search.
  prompt: string;
}

// Smart-tag retrieval usually needs a looser threshold than free-text search.
export const SMART_TAG_SEARCH_THRESHOLD = 0.2;

export interface SmartTagCategoryDef {
  id: string;
  items: SmartTagDef[];
}

export const SMART_TAG_CATEGORIES: SmartTagCategoryDef[] = [
  { id: 'people', items: [{ id: 'people', prompt: 'a photo of a person' }] },
  { id: 'kids', items: [{ id: 'kids', prompt: 'a photo of a child or baby' }] },
  { id: 'pets', items: [{ id: 'pets', prompt: 'a photo of a pet cat or dog' }] },
  { id: 'food', items: [{ id: 'food', prompt: 'a photo of food on a table' }] },
  { id: 'travel', items: [{ id: 'travel', prompt: 'a photo from a travel destination' }] },
];

export function getSmartTagById(id: string | null | undefined): SmartTagDef | null {
  if (!id) return null;
  for (const category of SMART_TAG_CATEGORIES) {
    const found = category.items.find(item => item.id === id);
    if (found) return found;
  }
  return null;
}
