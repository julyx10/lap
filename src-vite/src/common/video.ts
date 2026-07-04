import { invoke } from '@tauri-apps/api/core';

export interface VideoPrepareResult {
  url: string;
  action: string;
}

export type VideoPrepareMode = 'compatible' | 'process' | null;

export function isWebViewVideoPlaybackDisabled(filePath: string): boolean {
  const extension = filePath.match(/\.([^./\\]+)$/)?.[1]?.trim().toLowerCase() || '';
  return ['mpg', 'mpeg', 'vob'].includes(extension);
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
