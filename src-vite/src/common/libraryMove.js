export const LIBRARY_MOVE_CANCELLED_ERROR = 'Library storage move cancelled.';

export const sleep = (ms) => new Promise((resolve) => window.setTimeout(resolve, ms));

export function isFaceIndexingRunning(result) {
  return Array.isArray(result) ? Boolean(result[0]) : Boolean(result);
}

export async function waitForActivityIdle(
  fetchStatus,
  isRunning,
  timeoutMessage,
  options = {}
) {
  const timeoutMs = Number(options.timeoutMs || 10000);
  const pollMs = Number(options.pollMs || 200);
  const sleepFn = options.sleepFn || sleep;
  const startedAt = Date.now();

  while (Date.now() - startedAt < timeoutMs) {
    const status = await fetchStatus();
    if (!isRunning(status)) {
      return status;
    }
    await sleepFn(pollMs);
  }

  throw new Error(timeoutMessage);
}

export async function waitForIndexingIdle(getIndexingActivity, options = {}) {
  return waitForActivityIdle(
    () => getIndexingActivity({ strict: true }),
    (activity) => Boolean(activity?.running),
    'Timed out while waiting for indexing to stop.',
    options
  );
}

export async function waitForDedupIdle(dedupGetScanStatus, options = {}) {
  return waitForActivityIdle(
    () => dedupGetScanStatus({ strict: true }),
    (status) => status?.state === 'running',
    'Timed out while waiting for dedup scan to stop.',
    options
  );
}

export async function waitForFaceIndexIdle(isFaceIndexing, options = {}) {
  return waitForActivityIdle(
    () => isFaceIndexing({ strict: true }),
    isFaceIndexingRunning,
    'Timed out while waiting for face indexing to stop.',
    options
  );
}

export function createIndexMoveSnapshot(indexState) {
  const queueSnapshot = Array.isArray(indexState?.albumQueue)
    ? [...indexState.albumQueue]
    : [];
  const pausedSnapshot = Array.isArray(indexState?.pausedAlbumIds)
    ? [...indexState.pausedAlbumIds]
    : [];

  return {
    indexQueue: queueSnapshot,
    pausedAlbumIds: pausedSnapshot,
    indexStatus: Number(indexState?.status || 0),
    activeAlbumId: Number(queueSnapshot[0] || 0),
  };
}

export function pauseIndexStateForLibraryMove(indexState, snapshot = createIndexMoveSnapshot(indexState)) {
  if (!indexState || !snapshot.indexQueue.length) {
    return snapshot;
  }

  indexState.albumQueue = [];
  indexState.pausedAlbumIds = Array.from(
    new Set([...snapshot.pausedAlbumIds, ...snapshot.indexQueue].map((id) => Number(id)).filter((id) => id > 0))
  );
  indexState.status = 2;
  return snapshot;
}

export function restoreIndexStateAfterLibraryMove(indexState, snapshot) {
  if (!indexState || !snapshot || typeof snapshot !== 'object') {
    return;
  }

  indexState.pausedAlbumIds = Array.isArray(snapshot.pausedAlbumIds)
    ? [...snapshot.pausedAlbumIds]
    : [];
  indexState.albumQueue = Array.isArray(snapshot.indexQueue)
    ? [...snapshot.indexQueue]
    : [];

  if (indexState.albumQueue.length > 0) {
    indexState.status = Number(snapshot.indexStatus || 0) === 1 ? 1 : 2;
  } else {
    indexState.status = Number(snapshot.indexStatus || 0);
  }
}

export function shouldResumeDedup(snapshot) {
  return Boolean(
    snapshot &&
    snapshot.dedupWasRunning &&
    snapshot.dedupStoppedCleanly &&
    snapshot.dedupParams
  );
}

export function shouldResumeFace(faceMoveState) {
  return Boolean(
    faceMoveState &&
    faceMoveState.wasRunning &&
    faceMoveState.stoppedCleanly
  );
}

export function pickLibraryMoveError(primaryError, cleanupError) {
  return primaryError || cleanupError || null;
}

export function requiresConnectedMetadataStorage(libraryInfo) {
  return Boolean(
    libraryInfo &&
    libraryInfo.storageAvailable === false &&
    libraryInfo.metadataInitialized
  );
}
