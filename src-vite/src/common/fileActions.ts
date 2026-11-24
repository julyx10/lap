import { checkFileAccessibility, copyImages, renameFile, revealPath } from '@/common/api';
import { isOriginalStorageUnavailable } from '@/common/availability';
import { combineFileName, copyFilePaths, getClipboardItemPaths } from '@/common/utils';

export type FileActionHandler = () => void | Promise<void>;

export interface FileActionDispatchHooks {
  /** Run a menu handler, after the caller's unsaved-changes guard. */
  run: (handler: FileActionHandler) => void;
  onRating: (rating: number) => void;
  onCulling: (flag: number) => void;
  onExternalApp: (appId: string) => void;
  onManageExternalApps: () => void;
}

/**
 * Dispatch a file-menu action without owning any component state.
 *
 * The action ids are defined by `common/fileMenu.ts`; handlers remain local to
 * the window that owns the target file.
 */
export function dispatchFileAction(
  action: string,
  handlers: Record<string, FileActionHandler | undefined>,
  hooks: FileActionDispatchHooks,
): boolean {
  if (action.startsWith('rating-')) {
    const rating = Number.parseInt(action.slice('rating-'.length), 10);
    if (!Number.isNaN(rating)) hooks.onRating(rating);
    return true;
  }

  if (action.startsWith('culling-')) {
    const flag = action === 'culling-pick'
      ? 1
      : action === 'culling-reject'
        ? 2
        : action === 'culling-unreviewed'
          ? 0
          : -1;
    if (flag >= 0) hooks.onCulling(flag);
    return true;
  }

  if (action.startsWith('open-external-app:')) {
    hooks.onExternalApp(action.slice('open-external-app:'.length));
    return true;
  }

  if (action === 'manage-external-apps') {
    hooks.onManageExternalApps();
    return true;
  }

  const handler = handlers[action];
  if (!handler) return false;
  hooks.run(handler);
  return true;
}


/** Backend image errors use this prefix when the original file is unavailable. */
export function isOriginalUnavailableActionError(error: unknown): boolean {
  return String(error).startsWith('original_unavailable:');
}

export type MediaKind = 'image' | 'video' | 'mixed' | 'empty';

export function getMediaKind(items: any[]): MediaKind {
  let hasImage = false;
  let hasVideo = false;
  for (const item of items) {
    if (item?.file_type === 1 || item?.file_type === 3) hasImage = true;
    else if (item?.file_type === 2) hasVideo = true;
    if (hasImage && hasVideo) return 'mixed';
  }
  if (hasImage) return 'image';
  if (hasVideo) return 'video';
  return 'empty';
}

export interface ExternalAppLookup {
  externalAppsFor(kind: 'image' | 'video'): any[];
  defaultExternalApp(kind: 'image' | 'video'): any;
}

export type ExternalAppTarget =
  | { kind: 'empty' | 'mixed' | 'no-app' | 'no-files' }
  | { kind: 'image' | 'video'; appPath: string; paths: string[] };

/** Resolve the external app and file paths without emitting UI warnings. */
export function resolveExternalAppTarget(
  items: any[],
  appId: string | undefined,
  lookup: ExternalAppLookup,
): ExternalAppTarget {
  const kind = getMediaKind(items);
  if (kind === 'empty' || kind === 'mixed') return { kind };

  const appPath = appId
    ? String(lookup.externalAppsFor(kind).find((app: any) => app.id === appId)?.path || '')
    : String(lookup.defaultExternalApp(kind)?.path || '');
  if (!appPath) return { kind: 'no-app' };

  const paths = items.map(item => item?.file_path).filter(Boolean);
  if (paths.length === 0) return { kind: 'no-files' };
  return { kind, appPath, paths };
}

export interface ClipboardFilePaths {
  paths: string[];
  filesOnly: boolean;
}

/**
 * Resolve file-menu "copy" targets, de-duplicating RAW/JPEG companions across
 * a selection while respecting the clipboard item limit.
 */
export function getClipboardFilePaths(
  files: any[],
  groupRawJpegPairs: boolean,
  limit = 10,
): ClipboardFilePaths {
  const paths: string[] = [];
  const seenPaths = new Set<string>();
  let filesOnly = false;

  for (const file of files) {
    const groupPaths = getClipboardItemPaths(file, groupRawJpegPairs);
    const uniqueGroupPaths = groupPaths.filter(path => !seenPaths.has(path));
    if (paths.length + uniqueGroupPaths.length > limit) continue;
    if (groupPaths.length === 2) filesOnly = true;
    for (const path of uniqueGroupPaths) {
      seenPaths.add(path);
      paths.push(path);
    }
  }

  return { paths, filesOnly };
}


export interface CopyFilesResult {
  copiedCount: number;
  requestedCount: number;
  pathCount: number;
  filesOnly: boolean;
  offline: boolean;
  cancelled: boolean;
}

export interface CopyFilesOptions {
  groupRawJpegPairs: boolean;
  limit?: number;
}

/** Copy files to the OS clipboard, including companion/accessibility checks. */
export async function copyFilesToClipboard(
  files: any[],
  options: CopyFilesOptions,
): Promise<CopyFilesResult> {
  const requestedCount = files.length;
  if (requestedCount === 0) {
    return { copiedCount: 0, requestedCount, pathCount: 0, filesOnly: false, offline: false, cancelled: true };
  }
  if (files.some(isOriginalStorageUnavailable)) {
    return { copiedCount: 0, requestedCount, pathCount: 0, filesOnly: false, offline: true, cancelled: false };
  }
  const { paths, filesOnly } = getClipboardFilePaths(files, options.groupRawJpegPairs, options.limit);
  if (paths.length === 0) {
    return { copiedCount: 0, requestedCount, pathCount: 0, filesOnly, offline: false, cancelled: true };
  }
  const available = await Promise.all(paths.map(path => checkFileAccessibility(path)));
  if (available.some(value => !value)) {
    return { copiedCount: 0, requestedCount, pathCount: paths.length, filesOnly, offline: true, cancelled: false };
  }
  const copiedCount = Number(await copyImages(paths, filesOnly)) || 0;
  return { copiedCount, requestedCount, pathCount: paths.length, filesOnly, offline: false, cancelled: false };
}


/** Copy the selected files' paths (deduplicated) to the OS clipboard. */
export async function copyFilePathsForFiles(
  files: any[],
  groupRawJpegPairs: boolean,
): Promise<number> {
  return copyFilePaths(files, groupRawJpegPairs);
}


/** Reveal a file in the OS file manager. */
export function revealFile(file: { file_path?: string } | null | undefined): boolean {
  if (!file?.file_path) return false;
  revealPath(file.file_path);
  return true;
}

export interface RenameSingleFileResult {
  fileName: string;
  filePath: string;
  unchanged: boolean;
}

/**
 * Rename one file through the backend. UI state (dialogs, list refresh) stays
 * with the calling window; this function only owns the file-operation rules.
 */
export async function renameSingleFile(
  file: { id: number; name: string; file_path: string },
  requestedName: string,
  extension: string,
): Promise<RenameSingleFileResult | null> {
  const fileName = combineFileName(requestedName, extension);
  if (fileName === file.name) {
    return { fileName, filePath: file.file_path, unchanged: true };
  }

  const filePath = await renameFile(file.id, file.file_path, fileName);
  if (!filePath) return null;
  return { fileName, filePath, unchanged: false };
}

export interface FileDeleteTarget {
  id: number;
  name: string;
  file_path: string;
  album_id?: number;
  thumbnail?: string;
  rawPair: boolean;
  companion?: { id: number; file_path: string };
}

/** Build the backend delete targets, including RAW/JPEG and Live Photo companions. */
export function buildFileDeleteTargets(
  files: any[],
  groupRawJpegPairs: boolean,
): FileDeleteTarget[] {
  return files.map((file): FileDeleteTarget => {
    const rawPair = Boolean(groupRawJpegPairs && file.media_subtype === 'raw_jpeg_pair'
      && file.live_photo_video_id && file.live_photo_video_path);
    return {
      id: file.id,
      name: file.name,
      file_path: file.file_path,
      album_id: file.album_id,
      thumbnail: file.thumbnail,
      rawPair,
      companion: (rawPair || file.media_subtype === 'live_photo')
        && file.live_photo_video_id && file.live_photo_video_path
        ? { id: file.live_photo_video_id, file_path: file.live_photo_video_path }
        : undefined,
    };
  });
}

export interface FileDeletePlan {
  libraryId: string;
  requestId: number;
  files: FileDeleteTarget[];
  trashUnavailable?: boolean;
}

/** A delete needs confirmation when it is permanent or touches paired files. */
/** Build retry targets from only the components that failed, without pair expansion. */
export function buildDeleteRetryTargets(
  files: FileDeleteTarget[],
  failedIds: number[],
): FileDeleteTarget[] {
  const failed = new Set(failedIds);
  const retry: FileDeleteTarget[] = [];
  for (const file of files) {
    if (failed.has(Number(file.id))) {
      retry.push({ ...file, rawPair: false, companion: undefined });
    }
    if (file.companion && failed.has(Number(file.companion.id))) {
      retry.push({
        id: Number(file.companion.id),
        name: file.companion.file_path.split(/[/\\]/).pop() || file.companion.file_path,
        file_path: file.companion.file_path,
        album_id: file.album_id,
        rawPair: false,
      });
    }
  }
  return retry;
}

export function shouldConfirmDelete(plan: FileDeletePlan, permanently: boolean): boolean {
  return permanently || plan.files.length > 1 || plan.files.some(file => Boolean(file.companion));
}

export function countDeleteTargets(
  plan: FileDeletePlan | null,
  rawPairScope: 'both' | 'raw' | 'companion',
): number {
  return (plan?.files ?? []).reduce((count, file) =>
    count + (file.companion && (!file.rawPair || rawPairScope === 'both') ? 2 : 1), 0);
}

export function countRawPairTargets(plan: FileDeletePlan | null): number {
  return plan?.files.filter(file => file.rawPair).length ?? 0;
}

export function countSingleTargets(plan: FileDeletePlan | null): number {
  return (plan?.files.length ?? 0) - countRawPairTargets(plan);
}

export function rawPairCompanionFormat(plan: FileDeletePlan | null): string {
  const formats = new Set((plan?.files ?? []).filter(file => file.rawPair).map(file =>
    /\.jpe?g$/i.test(file.companion?.file_path || '') ? 'JPEG'
      : /\.(heic|heif|hif)$/i.test(file.companion?.file_path || '') ? 'HEIC' : ''));
  return formats.size === 1 ? [...formats][0] : '';
}
