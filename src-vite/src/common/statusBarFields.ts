export const statusBarFields: Record<string, string> = {
  name: 'statusbar.name_and_size',
  dimension: 'file_info.dimension',
  folder: 'file_info.folder',
  album_name: 'file_info.album_name',
  colorInfo: 'statusbar.color_info',
  takenDate: 'file_info.taken_at',
  camera: 'file_info.camera',
  lens: 'file_info.lens',
  exposure: 'file_info.capture_settings',
  location: 'file_info.geo_location',
};

const defaultVisibleFields = new Set(['name', 'folder', 'dimension']);

export type StatusBarField = { key: string; visible: boolean };

export function normalizeStatusBarFields(value: unknown): StatusBarField[] {
  const fields = new Map<string, StatusBarField>();
  if (Array.isArray(value)) {
    const currentKeys = new Set(value.map(field => field?.key));
    for (const field of value) {
      if (!field || typeof field.visible !== 'boolean') continue;
      const legacyColor = ['iccProfile', 'colorSpace', 'bitDepth'].includes(field.key);
      if (legacyColor && currentKeys.has('colorInfo')) continue;
      const key = legacyColor ? 'colorInfo' : field.key;
      if (!Object.hasOwn(statusBarFields, key)) continue;
      const previous = fields.get(key);
      if (!previous) fields.set(key, { key, visible: field.visible });
      else if (legacyColor) previous.visible ||= field.visible;
    }
  }
  for (const key of Object.keys(statusBarFields)) {
    if (!fields.has(key)) fields.set(key, { key, visible: defaultVisibleFields.has(key) });
  }
  return [...fields.values()];
}
