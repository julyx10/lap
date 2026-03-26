export type AlbumScanState = 'scanning' | 'queued' | 'paused' | 'complete';
export type AlbumScanIcon = 'update' | 'dot' | 'none';

type AlbumScanStateParams = {
  albumId: number | null | undefined;
  albumQueue: Array<number | string>;
  pausedAlbumIds?: Array<number | string> | null | undefined;
  status?: number | null | undefined;
};

export function getAlbumQueueIndex(
  albumId: number | null | undefined,
  albumQueue: Array<number | string>
): number {
  const normalizedAlbumId = Number(albumId || 0);
  if (normalizedAlbumId <= 0) return -1;
  return albumQueue.findIndex(id => Number(id) === normalizedAlbumId);
}

export function getAlbumScanState({
  albumId,
  albumQueue,
  pausedAlbumIds = [],
  status = 0,
}: AlbumScanStateParams): AlbumScanState {
  const normalizedAlbumId = Number(albumId || 0);
  if (normalizedAlbumId <= 0) return 'complete';

  const queueIndex = getAlbumQueueIndex(albumId, albumQueue);
  if (Number(status || 0) === 2 && queueIndex >= 0) return 'paused';
  if (queueIndex === 0) return 'scanning';
  if (queueIndex > 0) return 'queued';
  if ((pausedAlbumIds || []).some(id => Number(id) === normalizedAlbumId)) return 'paused';

  return 'complete';
}

export function getAlbumScanIcon(state: AlbumScanState): AlbumScanIcon {
  switch (state) {
    case 'scanning':
    case 'queued':
      return 'update';
    case 'paused':
      return 'dot';
    default:
      return 'none';
  }
}

export function shouldAnimateAlbumScanIcon(state: AlbumScanState): boolean {
  return state === 'scanning';
}
