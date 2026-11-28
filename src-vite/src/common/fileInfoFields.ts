export const editableMetadataFields = [
  { key: 'e_title', label: 'file_info.detail_fields.title' },
  { key: 'e_headline', label: 'file_info.detail_fields.headline' },
  { key: 'comments', label: 'file_info.comment' },
  { key: 'e_artist', label: 'file_info.detail_fields.creator' },
  { key: 'e_copyright', label: 'file_info.detail_fields.copyright' },
] as const;

export type CameraSettingsBrand = 'fujifilm' | 'canon' | 'nikon' | 'sony' | 'panasonic' | 'olympus' | 'pentax';

export const cameraSettingsBrands: ReadonlyArray<{ key: CameraSettingsBrand; label: string }> = [
  { key: 'fujifilm', label: 'Fujifilm' },
  { key: 'canon', label: 'Canon' },
  { key: 'nikon', label: 'Nikon' },
  { key: 'sony', label: 'Sony' },
  { key: 'panasonic', label: 'Panasonic / Leica' },
  { key: 'olympus', label: 'Olympus / OM System' },
  { key: 'pentax', label: 'Ricoh / Pentax' },
];

const ALL_CAMERA_BRANDS: readonly CameraSettingsBrand[] = [
  'fujifilm', 'canon', 'nikon', 'sony', 'panasonic', 'olympus', 'pentax',
];

const cameraSettingsFieldDefs: ReadonlyArray<{ key: string; brands: readonly CameraSettingsBrand[] }> = [
  { key: 'filmSimulation', brands: ['fujifilm'] },
  { key: 'pictureStyle', brands: ['canon'] },
  { key: 'pictureControl', brands: ['nikon'] },
  { key: 'creativeStyle', brands: ['sony'] },
  { key: 'pictureProfile', brands: ['sony'] },
  { key: 'pictureEffect', brands: ['sony'] },
  { key: 'photoStyle', brands: ['panasonic'] },
  { key: 'pictureMode', brands: ['olympus'] },
  { key: 'artFilter', brands: ['olympus'] },
  { key: 'imageTone', brands: ['pentax'] },
  { key: 'dynamicRange', brands: ['fujifilm'] },
  { key: 'autoLightingOptimizer', brands: ['canon'] },
  { key: 'highlightTonePriority', brands: ['canon'] },
  { key: 'activeDLighting', brands: ['nikon'] },
  { key: 'dynamicRangeOptimizer', brands: ['sony'] },
  { key: 'intelligentDRange', brands: ['panasonic'] },
  { key: 'gradation', brands: ['olympus'] },
  { key: 'dynamicRangeExpansion', brands: ['pentax'] },
  { key: 'shadowCorrection', brands: ['pentax'] },
  { key: 'highlightTone', brands: ['fujifilm'] },
  { key: 'shadowTone', brands: ['fujifilm'] },
  { key: 'contrast', brands: ALL_CAMERA_BRANDS },
  { key: 'saturation', brands: ALL_CAMERA_BRANDS },
  { key: 'sharpness', brands: ALL_CAMERA_BRANDS },
  { key: 'noiseReduction', brands: ALL_CAMERA_BRANDS },
  { key: 'clarity', brands: ['fujifilm', 'nikon', 'pentax'] },
  { key: 'whiteBalanceShift', brands: ALL_CAMERA_BRANDS },
  { key: 'grainEffect', brands: ['fujifilm', 'olympus'] },
  { key: 'colorChromeEffect', brands: ['fujifilm'] },
  { key: 'colorChromeFxBlue', brands: ['fujifilm'] },
  { key: 'focusMode', brands: ALL_CAMERA_BRANDS },
  { key: 'driveMode', brands: ['fujifilm', 'canon', 'nikon', 'sony', 'olympus', 'pentax'] },
  { key: 'imageStabilization', brands: ALL_CAMERA_BRANDS },
  { key: 'shutterCount', brands: ['nikon', 'sony', 'pentax'] },
];

export const cameraSettingsFields = cameraSettingsFieldDefs.map(({ key, brands }) => ({
  key: `cameraSettings.${key}`,
  label: `file_info.detail_fields.${key}`,
  brands,
}));

export function filterCameraSettingsFields(brand: 'all' | CameraSettingsBrand) {
  return brand === 'all'
    ? cameraSettingsFields
    : cameraSettingsFields.filter(field => field.brands.includes(brand));
}

// Field choices are independent of section expansion.
const fields = (section: string, keys: string[], details = false) => keys.map(key => ({
  key: `${section}.${key}`,
  label: `file_info.${details ? 'detail_fields.' : ''}${key}`,
}));
export const fileInfoFieldGroups = [
  { key: 'organization', label: 'file_info.organization', fields:
    editableMetadataFields.map(field => ({ ...field, key: `organization.${field.key}` })),
  },
  { key: 'file', label: 'file_info.file', fields: [
    ...fields('file', ['name', 'folder', 'album_name', 'size', 'dimension']),
    ...fields('file', ['iccProfile', 'colorSpace', 'bitDepth'], true),
    ...fields('file', ['created_at', 'modified_at', 'last_scan_time']),
  ] },
  { key: 'capture', label: 'file_info.capture', fields: [
    ...fields('capture', ['taken_at', 'camera', 'lens', 'capture_settings']),
    ...fields('capture', ['flash', 'whiteBalance', 'exposureMode', 'exposureProgram', 'meteringMode', 'focalLength35mm', 'timeZone'], true),
  ] },
  { key: 'cameraSettings', label: 'file_info.camera_settings', fields: cameraSettingsFields },
  { key: 'location', label: 'file_info.geo_location', fields: [
    ...fields('location', ['geo_location']),
    { key: 'location.gps', label: 'GPS' },
    ...fields('location', ['direction', 'directionRef'], true),
    { key: 'location.map', label: 'file_info.map' },
  ] },
  { key: 'source', label: 'file_info.source_metadata', fields: [
    ...fields('source', ['software', 'credit', 'source', 'usageTerms', 'rightsUrl'], true),
  ] },
];
export const allFileInfoFields = fileInfoFieldGroups.flatMap(group => group.fields.map(field => field.key));
export const defaultFileInfoFields = [
  'organization.e_title', 'organization.comments',
  'file.name', 'file.folder', 'file.size', 'file.dimension',
  'capture.taken_at', 'capture.camera', 'capture.lens', 'capture.capture_settings',
  'location.geo_location', 'location.gps', 'location.map',
];
export function normalizeFileInfoFields(value: unknown): string[] {
  const selected = new Set(Array.isArray(value) ? value : defaultFileInfoFields);
  return allFileInfoFields.filter(key => selected.has(key));
}

export function getVisibleEditableMetadataFields(value: unknown) {
  const selected = new Set(normalizeFileInfoFields(value));
  return editableMetadataFields.filter(field => selected.has(`organization.${field.key}`));
}
