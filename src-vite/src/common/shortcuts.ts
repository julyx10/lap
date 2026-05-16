export type ShortcutPlatform = 'mac' | 'windows' | 'linux';

export type ShortcutContext =
  | 'global'
  | 'content'
  | 'media-viewer'
  | 'image-viewer'
  | 'print'
  | 'settings'
  | 'album-list'
  | 'album-folder'
  | 'tag-dialog'
  | 'map';

export type ShortcutActionId =
  | 'app.scale.increase'
  | 'app.scale.decrease'
  | 'app.scale.reset'
  | 'app.preferences'
  | 'file.openNewWindow'
  | 'file.editImage'
  | 'file.print'
  | 'file.copy'
  | 'file.rename'
  | 'file.moveTo'
  | 'file.trash'
  | 'file.refreshInfo'
  | 'file.searchSimilar'
  | 'meta.favorite'
  | 'meta.rating.clear'
  | 'meta.rating.one'
  | 'meta.rating.two'
  | 'meta.rating.three'
  | 'meta.rating.four'
  | 'meta.rating.five'
  | 'meta.tag'
  | 'meta.comment'
  | 'meta.rotate'
  | 'meta.info'
  | 'view.quickPreview'
  | 'view.close'
  | 'view.next'
  | 'view.previous'
  | 'view.first'
  | 'view.last'
  | 'view.zoomIn'
  | 'view.zoomOut'
  | 'view.zoomInDirectional'
  | 'view.zoomOutDirectional'
  | 'view.zoomFit'
  | 'view.togglePane'
  | 'slideshow.toggle';

export type ShortcutModifier = 'ctrl' | 'meta' | 'alt' | 'shift' | 'cmdOrCtrl';

type ShortcutLabel = string | Partial<Record<ShortcutPlatform, string>> & { default?: string };

export interface ShortcutBinding {
  key?: string;
  code?: string;
  modifiers?: readonly ShortcutModifier[];
  platforms?: readonly ShortcutPlatform[];
  allowShift?: boolean;
  label: ShortcutLabel;
}

export interface ShortcutDefinition {
  id: ShortcutActionId;
  contexts: readonly ShortcutContext[];
  defaultBindings: readonly ShortcutBinding[];
  readonly?: boolean;
}

export interface ShortcutEventLike {
  key: string;
  code?: string;
  altKey?: boolean;
  ctrlKey?: boolean;
  metaKey?: boolean;
  shiftKey?: boolean;
}

export const DEFAULT_PLATFORM: ShortcutPlatform =
  typeof navigator !== 'undefined' && /mac/i.test(navigator.platform) ? 'mac' : 'windows';

export const SHORTCUTS: readonly ShortcutDefinition[] = [
  {
    id: 'app.scale.increase',
    contexts: ['global'],
    defaultBindings: [
      { key: '=', modifiers: ['cmdOrCtrl'], label: { mac: '⌘=', windows: 'Ctrl+=', linux: 'Ctrl+=' } },
      { key: '+', modifiers: ['cmdOrCtrl'], allowShift: true, label: { mac: '⌘+', windows: 'Ctrl++', linux: 'Ctrl++' } },
      { code: 'NumpadAdd', modifiers: ['cmdOrCtrl'], label: { mac: '⌘+', windows: 'Ctrl++', linux: 'Ctrl++' } },
    ],
  },
  {
    id: 'app.scale.decrease',
    contexts: ['global'],
    defaultBindings: [
      { key: '-', modifiers: ['cmdOrCtrl'], label: { mac: '⌘-', windows: 'Ctrl+-', linux: 'Ctrl+-' } },
      { key: '_', modifiers: ['cmdOrCtrl', 'shift'], label: { mac: '⌘-', windows: 'Ctrl+-', linux: 'Ctrl+-' } },
      { code: 'NumpadSubtract', modifiers: ['cmdOrCtrl'], label: { mac: '⌘-', windows: 'Ctrl+-', linux: 'Ctrl+-' } },
    ],
  },
  {
    id: 'app.scale.reset',
    contexts: ['global'],
    defaultBindings: [
      { key: '0', modifiers: ['cmdOrCtrl'], label: { mac: '⌘0', windows: 'Ctrl+0', linux: 'Ctrl+0' } },
      { code: 'Numpad0', modifiers: ['cmdOrCtrl'], label: { mac: '⌘0', windows: 'Ctrl+0', linux: 'Ctrl+0' } },
    ],
  },
  {
    id: 'app.preferences',
    contexts: ['global'],
    defaultBindings: [
      { key: ',', modifiers: ['cmdOrCtrl'], label: { mac: '⌘,', windows: 'Ctrl+,', linux: 'Ctrl+,' } },
    ],
  },
  {
    id: 'file.openNewWindow',
    contexts: ['content'],
    defaultBindings: [
      { key: 'Enter', modifiers: ['cmdOrCtrl'], allowShift: true, label: { mac: '⌘⏎', windows: 'Ctrl+Enter', linux: 'Ctrl+Enter' } },
    ],
  },
  {
    id: 'file.editImage',
    contexts: ['content'],
    defaultBindings: [
      { code: 'KeyE', modifiers: ['cmdOrCtrl'], allowShift: true, label: { mac: '⌘E', windows: 'Ctrl+E', linux: 'Ctrl+E' } },
    ],
  },
  {
    id: 'file.print',
    contexts: ['print'],
    defaultBindings: [
      { code: 'KeyP', modifiers: ['cmdOrCtrl'], label: { mac: '⌘P', windows: 'Ctrl+P', linux: 'Ctrl+P' } },
    ],
  },
  {
    id: 'file.copy',
    contexts: ['content'],
    defaultBindings: [
      { code: 'KeyC', modifiers: ['cmdOrCtrl'], allowShift: true, label: { mac: '⌘C', windows: 'Ctrl+C', linux: 'Ctrl+C' } },
    ],
  },
  {
    id: 'file.rename',
    contexts: ['content'],
    defaultBindings: [
      { key: 'Enter', platforms: ['mac'], allowShift: true, label: { mac: '⏎' } },
      { key: 'F2', platforms: ['windows', 'linux'], allowShift: true, label: { windows: 'F2', linux: 'F2' } },
    ],
  },
  {
    id: 'file.moveTo',
    contexts: ['content'],
    defaultBindings: [{ code: 'KeyM', allowShift: true, label: 'M' }],
  },
  {
    id: 'file.trash',
    contexts: ['content'],
    defaultBindings: [
      { key: 'Backspace', modifiers: ['meta'], platforms: ['mac'], allowShift: true, label: { mac: '⌘⌫' } },
      { key: 'Delete', platforms: ['windows', 'linux'], allowShift: true, label: { windows: 'Del', linux: 'Del' } },
    ],
  },
  {
    id: 'file.refreshInfo',
    contexts: ['content'],
    defaultBindings: [{ code: 'KeyR', modifiers: ['shift'], label: { mac: '⇧R', windows: 'Shift+R', linux: 'Shift+R' } }],
  },
  {
    id: 'file.searchSimilar',
    contexts: ['content'],
    defaultBindings: [{ code: 'KeyS', allowShift: true, label: 'S' }],
  },
  {
    id: 'meta.favorite',
    contexts: ['content', 'media-viewer', 'image-viewer'],
    defaultBindings: [{ code: 'KeyF', allowShift: true, label: 'F' }],
  },
  {
    id: 'meta.rating.clear',
    contexts: ['content', 'media-viewer', 'image-viewer'],
    defaultBindings: [{ key: '0', label: '0' }],
  },
  {
    id: 'meta.rating.one',
    contexts: ['content', 'media-viewer', 'image-viewer'],
    defaultBindings: [{ key: '1', label: '1' }],
  },
  {
    id: 'meta.rating.two',
    contexts: ['content', 'media-viewer', 'image-viewer'],
    defaultBindings: [{ key: '2', label: '2' }],
  },
  {
    id: 'meta.rating.three',
    contexts: ['content', 'media-viewer', 'image-viewer'],
    defaultBindings: [{ key: '3', label: '3' }],
  },
  {
    id: 'meta.rating.four',
    contexts: ['content', 'media-viewer', 'image-viewer'],
    defaultBindings: [{ key: '4', label: '4' }],
  },
  {
    id: 'meta.rating.five',
    contexts: ['content', 'media-viewer', 'image-viewer'],
    defaultBindings: [{ key: '5', label: '5' }],
  },
  {
    id: 'meta.tag',
    contexts: ['content', 'media-viewer', 'image-viewer'],
    defaultBindings: [{ code: 'KeyT', allowShift: true, label: 'T' }],
  },
  {
    id: 'meta.comment',
    contexts: ['content', 'media-viewer', 'image-viewer'],
    defaultBindings: [{ code: 'KeyC', allowShift: true, label: 'C' }],
  },
  {
    id: 'meta.rotate',
    contexts: ['content', 'media-viewer', 'image-viewer'],
    defaultBindings: [{ code: 'KeyR', allowShift: true, label: 'R' }],
  },
  {
    id: 'meta.info',
    contexts: ['content', 'media-viewer'],
    defaultBindings: [
      { code: 'KeyI', allowShift: true, label: 'I' },
      { code: 'KeyI', modifiers: ['cmdOrCtrl'], allowShift: true, label: { mac: '⌘I', windows: 'Ctrl+I', linux: 'Ctrl+I' } },
    ],
  },
  {
    id: 'view.quickPreview',
    contexts: ['content'],
    defaultBindings: [
      { key: 'Enter', platforms: ['windows', 'linux'], label: { windows: 'Enter', linux: 'Enter' } },
      { key: ' ', label: 'Space' },
      { key: 'Space', label: 'Space' },
    ],
  },
  {
    id: 'view.close',
    contexts: ['content', 'image-viewer', 'settings', 'print'],
    defaultBindings: [{ key: 'Escape', label: 'Esc' }],
  },
  {
    id: 'view.next',
    contexts: ['content', 'image-viewer', 'album-list', 'album-folder'],
    defaultBindings: [{ key: 'ArrowRight', label: '→' }],
  },
  {
    id: 'view.previous',
    contexts: ['content', 'image-viewer', 'album-list', 'album-folder'],
    defaultBindings: [{ key: 'ArrowLeft', label: '←' }],
  },
  {
    id: 'view.first',
    contexts: ['content', 'image-viewer'],
    defaultBindings: [{ key: 'Home', label: 'Home' }],
  },
  {
    id: 'view.last',
    contexts: ['content', 'image-viewer'],
    defaultBindings: [{ key: 'End', label: 'End' }],
  },
  {
    id: 'view.zoomIn',
    contexts: ['content', 'media-viewer', 'image-viewer', 'map'],
    defaultBindings: [{ key: '=', label: '=' }],
  },
  {
    id: 'view.zoomOut',
    contexts: ['content', 'media-viewer', 'image-viewer', 'map'],
    defaultBindings: [{ key: '-', label: '-' }],
  },
  {
    id: 'view.zoomInDirectional',
    contexts: ['image-viewer'],
    defaultBindings: [{ key: 'ArrowUp', label: '↑' }],
  },
  {
    id: 'view.zoomOutDirectional',
    contexts: ['image-viewer'],
    defaultBindings: [{ key: 'ArrowDown', label: '↓' }],
  },
  {
    id: 'view.zoomFit',
    contexts: ['content', 'media-viewer', 'image-viewer'],
    defaultBindings: [
      { key: ' ', label: 'Space' },
      { key: 'Space', label: 'Space' },
    ],
  },
  {
    id: 'view.togglePane',
    contexts: ['image-viewer'],
    defaultBindings: [{ key: 'Tab', label: 'Tab' }],
  },
  {
    id: 'slideshow.toggle',
    contexts: ['content', 'media-viewer', 'image-viewer'],
    defaultBindings: [{ code: 'KeyP', allowShift: true, label: 'P' }],
  },
] as const;

const shortcutMap = new Map<ShortcutActionId, ShortcutDefinition>(
  SHORTCUTS.map((shortcut) => [shortcut.id, shortcut]),
);

export function getShortcut(actionId: ShortcutActionId): ShortcutDefinition {
  const shortcut = shortcutMap.get(actionId);
  if (!shortcut) {
    throw new Error(`Unknown shortcut action: ${actionId}`);
  }
  return shortcut;
}

export function getShortcutsByContext(context: ShortcutContext): readonly ShortcutDefinition[] {
  return SHORTCUTS.filter((shortcut) => shortcut.contexts.includes(context));
}

export function getShortcutLabels(
  actionId: ShortcutActionId,
  platform: ShortcutPlatform = DEFAULT_PLATFORM,
): string[] {
  return getShortcut(actionId).defaultBindings
    .filter((binding) => isBindingAvailableOnPlatform(binding, platform))
    .map((binding) => getBindingLabel(binding, platform));
}

export function getShortcutLabel(
  actionId: ShortcutActionId,
  platform: ShortcutPlatform = DEFAULT_PLATFORM,
): string {
  return getShortcutLabels(actionId, platform)[0] ?? '';
}

export function matchesShortcut(
  actionId: ShortcutActionId,
  event: ShortcutEventLike,
  platform: ShortcutPlatform = DEFAULT_PLATFORM,
): boolean {
  return getShortcut(actionId).defaultBindings.some((binding) => matchesBinding(binding, event, platform));
}

export function findShortcutMatches(
  context: ShortcutContext,
  event: ShortcutEventLike,
  platform: ShortcutPlatform = DEFAULT_PLATFORM,
): ShortcutActionId[] {
  return getShortcutsByContext(context)
    .filter((shortcut) => shortcut.defaultBindings.some((binding) => matchesBinding(binding, event, platform)))
    .map((shortcut) => shortcut.id);
}

export function getShortcutConflicts(
  context: ShortcutContext,
  platform: ShortcutPlatform = DEFAULT_PLATFORM,
): Array<{ signature: string; actions: ShortcutActionId[] }> {
  const actionsBySignature = new Map<string, ShortcutActionId[]>();

  for (const shortcut of getShortcutsByContext(context)) {
    for (const binding of shortcut.defaultBindings) {
      if (!isBindingAvailableOnPlatform(binding, platform)) continue;
      for (const signature of getBindingSignatures(binding, platform)) {
        const actions = actionsBySignature.get(signature) ?? [];
        actions.push(shortcut.id);
        actionsBySignature.set(signature, actions);
      }
    }
  }

  return Array.from(actionsBySignature.entries())
    .filter(([, actions]) => actions.length > 1)
    .map(([signature, actions]) => ({ signature, actions }));
}

export function getBindingLabel(
  binding: ShortcutBinding,
  platform: ShortcutPlatform = DEFAULT_PLATFORM,
): string {
  if (typeof binding.label === 'string') return binding.label;
  return binding.label[platform] ?? binding.label.default ?? '';
}

export function getBindingSignature(
  binding: ShortcutBinding,
  platform: ShortcutPlatform = DEFAULT_PLATFORM,
): string {
  return getBindingSignatures(binding, platform)[0];
}

function getBindingSignatures(
  binding: ShortcutBinding,
  platform: ShortcutPlatform = DEFAULT_PLATFORM,
): string[] {
  const modifiers = (binding.modifiers ?? []).map((modifier) =>
    modifier === 'cmdOrCtrl' ? (platform === 'mac' ? 'meta' : 'ctrl') : modifier,
  );
  modifiers.sort();
  const baseSignature = [...modifiers, binding.code ?? '', binding.key ?? ''].join('+');

  if (!binding.allowShift || modifiers.includes('shift')) {
    return [baseSignature];
  }

  const shiftedModifiers = [...modifiers, 'shift'];
  shiftedModifiers.sort();
  return [baseSignature, [...shiftedModifiers, binding.code ?? '', binding.key ?? ''].join('+')];
}

function matchesBinding(
  binding: ShortcutBinding,
  event: ShortcutEventLike,
  platform: ShortcutPlatform,
): boolean {
  if (!isBindingAvailableOnPlatform(binding, platform)) return false;
  if (binding.key && event.key !== binding.key) return false;
  if (binding.code && event.code !== binding.code) return false;

  const modifiers = new Set(binding.modifiers ?? []);
  const expectsCtrl = modifiers.has('ctrl') || (modifiers.has('cmdOrCtrl') && platform !== 'mac');
  const expectsMeta = modifiers.has('meta') || (modifiers.has('cmdOrCtrl') && platform === 'mac');

  return Boolean(event.ctrlKey) === expectsCtrl
    && Boolean(event.metaKey) === expectsMeta
    && Boolean(event.altKey) === modifiers.has('alt')
    && (binding.allowShift && !modifiers.has('shift')
      ? true
      : Boolean(event.shiftKey) === modifiers.has('shift'));
}

function isBindingAvailableOnPlatform(binding: ShortcutBinding, platform: ShortcutPlatform): boolean {
  return !binding.platforms || binding.platforms.includes(platform);
}
