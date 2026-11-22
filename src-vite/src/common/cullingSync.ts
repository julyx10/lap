export type CullingPane = 'left' | 'right' | 'bottomLeft' | 'bottomRight';

interface PendingCullingEntry {
  seq: number;
  value: number;
  previous: number | null;
}

interface CommittedCullingEntry {
  value: number;
  committedAtLoadSeq: number;
}

export function normalizeCullingFlag(flag: number): number {
  return Math.max(0, Math.min(2, flag));
}

export function getMatchingPaneFileInfo<T extends { id?: unknown }>(
  info: T | null | undefined,
  expectedFileId: number,
): T | null {
  if (!info || expectedFileId <= 0) return null;
  return Number(info.id) === expectedFileId ? info : null;
}

export function createCullingSyncTracker() {
  const paneRequestIds: Record<CullingPane, number> = {
    left: 0,
    right: 0,
    bottomLeft: 0,
    bottomRight: 0,
  };
  const pendingByFileId = new Map<number, PendingCullingEntry>();
  const committedByFileId = new Map<number, CommittedCullingEntry>();
  let loadSeq = 0;
  let cullingSeq = 0;

  function beginFileInfoLoad(pane: CullingPane): number {
    loadSeq += 1;
    paneRequestIds[pane] = loadSeq;
    return loadSeq;
  }

  function isCurrentFileInfoLoad(
    pane: CullingPane,
    requestId: number,
    activeFileId: number,
    requestedFileId: number,
  ): boolean {
    return paneRequestIds[pane] === requestId && activeFileId === requestedFileId;
  }

  function applyPendingCulling<T extends Record<string, any>>(
    fileInfo: T | null,
    loadRequestId?: number,
  ): T | null {
    if (!fileInfo) return fileInfo;
    const id = Number(fileInfo.id);
    if (!Number.isInteger(id) || id <= 0) return fileInfo;

    const committed = committedByFileId.get(id);
    if (committed && (loadRequestId === undefined || loadRequestId <= committed.committedAtLoadSeq)) {
      fileInfo.culling_flag = committed.value;
    }

    const pending = pendingByFileId.get(id);
    if (!pending) return fileInfo;
    if (pending.previous === null) {
      pending.previous = Number(fileInfo.culling_flag ?? fileInfo.cullingFlag ?? 0);
    }
    fileInfo.culling_flag = pending.value;
    return fileInfo;
  }

  function beginCullingUpdate(
    fileId: number,
    cullingFlag: number,
    knownPrevious: number | null,
  ): { seq: number; normalized: number } {
    const seq = ++cullingSeq;
    const normalized = normalizeCullingFlag(cullingFlag);
    const existing = pendingByFileId.get(fileId);
    const previous = existing ? existing.previous : knownPrevious;
    pendingByFileId.set(fileId, { seq, value: normalized, previous });
    return { seq, normalized };
  }

  function finishCullingUpdate(
    fileId: number,
    seq: number,
    succeeded = true,
  ): { isLatest: boolean; previous: number | null } {
    const entry = pendingByFileId.get(fileId);
    if (!entry || entry.seq !== seq) {
      return { isLatest: false, previous: null };
    }
    pendingByFileId.delete(fileId);
    if (succeeded) {
      committedByFileId.set(fileId, { value: entry.value, committedAtLoadSeq: loadSeq });
    }
    return { isLatest: true, previous: entry.previous };
  }

  return {
    beginFileInfoLoad,
    isCurrentFileInfoLoad,
    applyPendingCulling,
    beginCullingUpdate,
    finishCullingUpdate,
  };
}
