import {
  createIndexMoveSnapshot,
  isFaceIndexingRunning,
  pickLibraryMoveError,
  requiresConnectedMetadataStorage,
  restoreIndexStateAfterLibraryMove,
  shouldResumeDedup,
  shouldResumeFace,
  waitForDedupIdle,
  waitForFaceIndexIdle,
  waitForIndexingIdle,
} from './libraryMove';

describe('libraryMove helpers', () => {
  it('fails closed when indexing activity lookup throws', async () => {
    await expect(
      waitForIndexingIdle(() => {
        throw new Error('bridge down');
      }, { timeoutMs: 50, pollMs: 1 })
    ).rejects.toThrow('bridge down');
  });

  it('fails closed when dedup status lookup throws', async () => {
    await expect(
      waitForDedupIdle(() => {
        throw new Error('dedup unavailable');
      }, { timeoutMs: 50, pollMs: 1 })
    ).rejects.toThrow('dedup unavailable');
  });

  it('fails closed when face indexing lookup throws', async () => {
    await expect(
      waitForFaceIndexIdle(() => {
        throw new Error('face unavailable');
      }, { timeoutMs: 50, pollMs: 1 })
    ).rejects.toThrow('face unavailable');
  });

  it('restores index queue and status after move', () => {
    const indexState = {
      status: 1,
      albumQueue: [11, 22],
      pausedAlbumIds: [33],
    };

    const snapshot = createIndexMoveSnapshot(indexState);
    indexState.status = 2;
    indexState.albumQueue = [];
    indexState.pausedAlbumIds = [11, 22, 33];

    restoreIndexStateAfterLibraryMove(indexState, snapshot);

    expect(indexState).toEqual({
      status: 1,
      albumQueue: [11, 22],
      pausedAlbumIds: [33],
    });
  });

  it('detects disconnected metadata that must reconnect first', () => {
    expect(
      requiresConnectedMetadataStorage({
        storageAvailable: false,
        metadataInitialized: true,
      })
    ).toBe(true);
    expect(
      requiresConnectedMetadataStorage({
        storageAvailable: false,
        metadataInitialized: false,
      })
    ).toBe(false);
  });

  it('only resumes dedup after a clean stop', () => {
    expect(
      shouldResumeDedup({
        dedupWasRunning: true,
        dedupStoppedCleanly: false,
        dedupParams: { path: 'x' },
      })
    ).toBe(false);

    expect(
      shouldResumeDedup({
        dedupWasRunning: true,
        dedupStoppedCleanly: true,
        dedupParams: { path: 'x' },
      })
    ).toBe(true);
  });

  it('only resumes face indexing after a clean stop', () => {
    expect(
      shouldResumeFace({
        wasRunning: true,
        stoppedCleanly: false,
      })
    ).toBe(false);

    expect(
      shouldResumeFace({
        wasRunning: true,
        stoppedCleanly: true,
      })
    ).toBe(true);
  });

  it('keeps the primary move error when cleanup also fails', () => {
    const primaryError = new Error('move failed');
    const cleanupError = new Error('resume failed');

    expect(pickLibraryMoveError(primaryError, cleanupError)).toBe(primaryError);
    expect(pickLibraryMoveError(null, cleanupError)).toBe(cleanupError);
  });

  it('normalizes face indexing responses', () => {
    expect(isFaceIndexingRunning([true, null])).toBe(true);
    expect(isFaceIndexingRunning([false, null])).toBe(false);
    expect(isFaceIndexingRunning(true)).toBe(true);
  });
});
