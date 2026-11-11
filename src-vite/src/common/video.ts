import { invoke } from '@tauri-apps/api/core';
import { matchesShortcut, type ShortcutPlatform } from './shortcuts';

export function handleVideoEscapeShortcut(event: KeyboardEvent, platform: ShortcutPlatform, closePreview: () => void) {
  if (event.defaultPrevented || !matchesShortcut('view.close', event, platform)) return;
  const target = event.target as HTMLElement | null;
  // Only bypass video.js controls; leave other controls and dialogs to their owners.
  if (!target?.closest?.('.video-js')
    || target.closest('input, textarea, [contenteditable]:not([contenteditable="false"])')) return;
  // The player's own fullscreen must exit before the surrounding preview closes.
  if (document.fullscreenElement || target.closest('.vjs-fullscreen')) return;
  if (!event.repeat) closePreview();
  event.preventDefault();
  event.stopPropagation();
}

export function handleVideoPlaybackShortcut(event: KeyboardEvent, platform: ShortcutPlatform, togglePlay: () => void) {
  if (!matchesShortcut('view.togglePlayback', event, platform)) return;
  const target = event.target as HTMLElement | null;
  if (target?.closest?.('input, textarea, [contenteditable]:not([contenteditable="false"])')) return;
  // Intercept before video.js controls, so one press performs one action.
  event.preventDefault();
  event.stopPropagation();
  if (!event.repeat) togglePlay();
}

export interface VideoPrepareResult {
  url: string;
  action: string;
}

export type VideoPrepareMode = 'compatible' | 'process' | null;

export async function getVideoStreamUrl(filePath: string): Promise<string> {
  return invoke<string>('get_video_stream_url', { filePath });
}

export function isWebViewVideoPlaybackDisabled(filePath: string): boolean {
  const extension = filePath.match(/\.([^./\\]+)$/)?.[1]?.trim().toLowerCase() || '';
  return ['mpg', 'mpeg', 'vob'].includes(extension);
}

// Share one probe across hover previews. Await it before loading video (#313);
// unknown failures preserve playback, while confirmed missing elements disable it.
let gstreamerAvailability: Promise<boolean> | null = null;

export function getGStreamerAvailability(): Promise<boolean> {
  if (!gstreamerAvailability) {
    gstreamerAvailability = invoke<boolean>('check_gstreamer_available').catch(() => true);
  }
  return gstreamerAvailability;
}

export async function prepareVideo(
  filePath: string,
  playerId: string = 'default',
  force: VideoPrepareMode = null,
  requestId: number | null = null
): Promise<VideoPrepareResult> {
  return invoke<VideoPrepareResult>('prepare_video', { filePath, playerId, force, requestId });
}

export async function cancelVideoPrepare(
  playerId: string = 'default',
  requestId: number | null = null
): Promise<void> {
  return invoke('cancel_video_prepare', { playerId, requestId });
}

export async function clearVideoCache(): Promise<void> {
  return invoke('clear_video_cache');
}
