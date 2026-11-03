export interface LibraryTag {
  id: number;
  name: string;
  parent_id: number | null;
  path: string;
  count?: number;
}
export interface TagRow extends LibraryTag {
  depth: number;
  hasChildren: boolean;
}

export function isValidTagName(name: string): boolean {
  name = name.trim();
  return name.length > 0 &&
    Array.from(name).length <= 255 && !/[,;|\0]/.test(name);
}

export function ancestorIds(tags: LibraryTag[], id: number): number[] {
  const byId = new Map(tags.map((tag) => [tag.id, tag]));
  const result: number[] = [];
  const seen = new Set<number>();
  let parent = byId.get(id)?.parent_id;
  while (parent != null && !seen.has(parent)) {
    seen.add(parent);
    result.push(parent);
    parent = byId.get(parent)?.parent_id;
  }
  return result;
}

export function subtreeIds(tags: LibraryTag[], id: number): Set<number> {
  const children = new Map<number, number[]>();
  for (const tag of tags)
    if (tag.parent_id != null) {
      if (!children.has(tag.parent_id)) children.set(tag.parent_id, []);
      children.get(tag.parent_id)!.push(tag.id);
    }
  const result = new Set<number>();
  const pending = [id];
  while (pending.length) {
    const next = pending.pop()!;
    if (result.has(next)) continue;
    result.add(next);
    pending.push(...(children.get(next) || []));
  }
  return result;
}

export function tagRows(
  tags: LibraryTag[],
  search = "",
  collapsed: number[] = [],
  sort = 0,
  draftId?: number,
): TagRow[] {
  const children = new Map<number | null, LibraryTag[]>();
  const byId = new Map(tags.map((tag) => [tag.id, tag]));
  const query = search.trim().toLocaleLowerCase();
  const included = new Set<number>();
  if (query)
    for (const tag of tags) {
      if (!(tag.path || tag.name).toLocaleLowerCase().includes(query)) continue;
      let node: LibraryTag | undefined = tag;
      while (node && !included.has(node.id)) {
        included.add(node.id);
        node = node.parent_id == null ? undefined : byId.get(node.parent_id);
      }
    }
  for (const tag of tags) {
    if (query && !included.has(tag.id)) continue;
    const parent = tag.parent_id ?? null;
    if (!children.has(parent)) children.set(parent, []);
    children.get(parent)!.push(tag);
  }
  for (const siblings of children.values())
    siblings.sort((a, b) => {
      if (a.id === draftId || b.id === draftId)
        return Number(a.id === draftId) - Number(b.id === draftId);
      const count =
        sort === 2
          ? Number(a.count || 0) - Number(b.count || 0)
          : sort === 3
            ? Number(b.count || 0) - Number(a.count || 0)
            : 0;
      return (
        count ||
        (sort === 1
          ? b.name.localeCompare(a.name)
          : a.name.localeCompare(b.name)) ||
        a.id - b.id
      );
    });
  const result: TagRow[] = [];
  const closed = new Set(collapsed);
  const seen = new Set<number>();
  const stack = (children.get(null) || [])
    .map((tag) => ({ tag, depth: 0 }))
    .reverse();
  while (stack.length) {
    const { tag, depth } = stack.pop()!;
    if (seen.has(tag.id)) continue;
    seen.add(tag.id);
    const next = children.get(tag.id) || [];
    result.push({ ...tag, depth, hasChildren: next.length > 0 });
    if (query || !closed.has(tag.id))
      stack.push(
        ...next.map((child) => ({ tag: child, depth: depth + 1 })).reverse(),
      );
  }
  return result;
}
