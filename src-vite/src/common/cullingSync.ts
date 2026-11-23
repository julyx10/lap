export type CullingPane = 'left' | 'right' | 'bottomLeft' | 'bottomRight';

export interface CullingAdvanceInfo {
  pane: CullingPane;
  advance: boolean;
}

export interface EnqueueCullingUpdateOptions {
  fileId: number;
  cullingFlag: number;
  knownConfirmed: number | null;
  pane: CullingPane;
  advance: boolean;
  applyOptimistic: (fileId: number, cullingFlag: number) => void;
  persist: (fileId: number, cullingFlag: number) => Promise<number | null>;
  onCommit: (
    fileId: number,
    cullingFlag: number,
    settled: boolean,
    advanceInfo: CullingAdvanceInfo | null,
  ) => void;
  onError: (fileId: number, rollbackFlag: number | null) => void;
}

interface FileCullingState {
  confirmed: number | null;
  committedAtLoadSeq: number | null;
  desired: number | null;
  pendingAdvance: CullingAdvanceInfo | null;
  inFlight: boolean;
  activePromise: Promise<void> | null;
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
  const stateByFileId = new Map<number, FileCullingState>();
  let loadSeq = 0;

  function getOrCreateState(fileId: number): FileCullingState {
    let state = stateByFileId.get(fileId);
    if (!state) {
      state = {
        confirmed: null,
        committedAtLoadSeq: null,
        desired: null,
        pendingAdvance: null,
        inFlight: false,
        activePromise: null,
      };
      stateByFileId.set(fileId, state);
    }
    return state;
  }

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

    const state = stateByFileId.get(id);
    if (!state) return fileInfo;

    const loadedFlag = Number(fileInfo.culling_flag ?? fileInfo.cullingFlag ?? 0);
    const isStaleLoad =
      state.committedAtLoadSeq !== null &&
      (loadRequestId === undefined || loadRequestId <= state.committedAtLoadSeq);

    if (isStaleLoad && state.confirmed !== null) {
      fileInfo.culling_flag = state.confirmed;
    } else if (!state.inFlight || state.confirmed === null) {
      state.confirmed = loadedFlag;
    }

    if (state.desired !== null) {
      fileInfo.culling_flag = state.desired;
    }
    return fileInfo;
  }

  function recordExternalCulling(fileId: number, cullingFlag: number) {
    if (!Number.isInteger(fileId) || fileId <= 0) return;
    const state = getOrCreateState(fileId);
    if (!state.inFlight) {
      state.confirmed = normalizeCullingFlag(cullingFlag);
      state.committedAtLoadSeq = loadSeq;
    }
  }

  function enqueueCullingUpdate(options: EnqueueCullingUpdateOptions): Promise<void> {
    const { fileId } = options;
    const normalized = normalizeCullingFlag(options.cullingFlag);
    const state = getOrCreateState(fileId);

    if (state.confirmed === null && options.knownConfirmed !== null) {
      state.confirmed = normalizeCullingFlag(options.knownConfirmed);
    }

    state.desired = normalized;
    state.pendingAdvance = { pane: options.pane, advance: options.advance };
    options.applyOptimistic(fileId, normalized);

    if (state.inFlight && state.activePromise) {
      return state.activePromise;
    }

    state.inFlight = true;
    const run = (async () => {
      try {
        while (state.desired !== null) {
          const target = state.desired;
          const result = await options.persist(fileId, target);

          if (result !== null) {
            state.confirmed = target;
            state.committedAtLoadSeq = loadSeq;
            const settled = state.desired === target;
            const advanceInfo = settled ? state.pendingAdvance : null;
            if (settled) {
              state.desired = null;
              state.pendingAdvance = null;
            }
            options.onCommit(fileId, target, settled, settled ? advanceInfo : null);
            continue;
          }

          if (state.desired === target) {
            state.desired = null;
            state.pendingAdvance = null;
            options.onError(fileId, state.confirmed);
            break;
          }
        }
      } finally {
        state.inFlight = false;
        state.activePromise = null;
      }
    })();

    state.activePromise = run;
    return run;
  }

  return {
    beginFileInfoLoad,
    isCurrentFileInfoLoad,
    applyPendingCulling,
    recordExternalCulling,
    enqueueCullingUpdate,
  };
}
