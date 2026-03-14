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
  { id: 'family', items: [{ id: 'family', prompt: 'a family photo with parents children and relatives together' }] },
  { id: 'kids', items: [{ id: 'kids', prompt: 'a photo of children or baby' }] },
  { id: 'pets', items: [{ id: 'pets', prompt: 'a photo of a pet dog or cat' }] },
  { id: 'portraits', items: [{ id: 'portraits', prompt: 'a portrait photo of one person' }] },
  { id: 'food', items: [{ id: 'food', prompt: 'a close-up photo of food dish meal on table' }] },
  { id: 'sports', items: [{ id: 'sports', prompt: 'a sports action photo of people running playing or exercising' }] },
  { id: 'landscape', items: [{ id: 'landscape', prompt: 'a natural landscape photo with mountain lake ocean forest or sky' }] },
  { id: 'night', items: [{ id: 'night', prompt: 'a night scene photo in low light with lights or stars' }] },
];

export function getSmartTagById(id: string | null | undefined): SmartTagDef | null {
  if (!id) return null;
  for (const category of SMART_TAG_CATEGORIES) {
    const found = category.items.find(item => item.id === id);
    if (found) return found;
  }
  return null;
}
