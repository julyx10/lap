// Field choices are independent of section expansion and Organization.
const fields = (section: string, keys: string[], details = false) => keys.map(key => ({
  key: `${section}.${key}`,
  label: `file_info.${details ? 'detail_fields.' : ''}${key}`,
}));
export const fileInfoFieldGroups = [
  { key: 'file', label: 'file_info.file', fields: [
    ...fields('file', ['name', 'folder', 'album_name', 'size', 'dimension', 'duration']),
    ...fields('file', ['iccProfile', 'colorSpace', 'bitDepth'], true),
    ...fields('file', ['created_at', 'modified_at', 'last_scan_time']),
  ] },
  { key: 'capture', label: 'file_info.capture', fields: [
    ...fields('capture', ['taken_at', 'camera', 'lens', 'capture_settings']),
    ...fields('capture', ['flash', 'whiteBalance', 'exposureMode', 'exposureProgram', 'meteringMode', 'focalLength35mm', 'timeZone'], true),
  ] },
  { key: 'location', label: 'file_info.geo_location', fields: [
    ...fields('location', ['geo_location']),
    { key: 'location.gps', label: 'GPS' },
    ...fields('location', ['direction', 'directionRef'], true),
    { key: 'location.map', label: 'file_info.map' },
  ] },
  { key: 'source', label: 'file_info.source_metadata', fields: fields('source',
    ['title', 'headline', 'creator', 'copyright', 'description', 'software', 'credit', 'source', 'usageTerms', 'rightsUrl'], true) },
];
export const allFileInfoFields = fileInfoFieldGroups.flatMap(group => group.fields.map(field => field.key));
export const defaultFileInfoFields = [
  'file.name', 'file.folder', 'file.size', 'file.dimension', 'file.duration',
  'capture.taken_at', 'capture.camera', 'capture.lens', 'capture.capture_settings',
  'location.geo_location', 'location.gps', 'location.map',
];
export function normalizeFileInfoFields(value: unknown): string[] {
  const selected = new Set(Array.isArray(value) ? value : defaultFileInfoFields);
  return allFileInfoFields.filter(key => selected.has(key));
}
