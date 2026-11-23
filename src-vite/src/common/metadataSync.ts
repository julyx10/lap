export type MetadataPane = 'left' | 'right' | 'bottomLeft' | 'bottomRight';

export interface MetadataAdvanceInfo {
  pane: MetadataPane;
  advance: boolean;
  requestId?: number;
}

export interface EnqueueMetadataUpdateOptions {
  fileId: number;
  value: number;
  knownConfirmed: number | null;
  requestId?: number;
  pane: MetadataPane;
  advance: boolean;
  applyOptimistic: (fileId: number, value: number) => void;
  persist: (fileId: number, value: number) => Promise<number | null>;
  onCommit: (
    fileId: number,
    value: number,
    settled: boolean,
    advanceInfo: MetadataAdvanceInfo | null,
  ) => void;
  onError: (fileId: number, rollbackFlag: number | null) => void;
}

interface FileMetadataState {
  confirmed: number | null;
  committedAtLoadSeq: number | null;
  desired: number | null;
  pendingAdvance: MetadataAdvanceInfo | null;
  inFlight: boolean;
  activePromise: Promise<void> | null;
}

export type MetadataField = 'culling_flag' | 'rating' | 'is_favorite';

export function getMatchingPaneFileInfo<T extends { id?: unknown }>(
  info: T | null | undefined,
  expectedFileId: number,
): T | null {
  if (!info || expectedFileId <= 0) return null;
  return Number(info.id) === expectedFileId ? info : null;
}

export function createMetadataSyncTracker(field: MetadataField = 'culling_flag') {
  const max = { culling_flag: 2, rating: 5, is_favorite: 1 }[field];
  const paneRequestIds: Record<MetadataPane, number> = {
    left: 0,
    right: 0,
    bottomLeft: 0,
    bottomRight: 0,
  };
  const stateByFileId = new Map<number, FileMetadataState>();
  let loadSeq = 0;

  function getOrCreateState(fileId: number): FileMetadataState {
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

  function beginFileInfoLoad(pane: MetadataPane): number {
    loadSeq += 1;
    paneRequestIds[pane] = loadSeq;
    return loadSeq;
  }

  function isCurrentFileInfoLoad(
    pane: MetadataPane,
    requestId: number,
    activeFileId: number,
    requestedFileId: number,
  ): boolean {
    return paneRequestIds[pane] === requestId && activeFileId === requestedFileId;
  }

  function applyPendingMetadata<T extends Record<string, any>>(
    fileInfo: T | null,
    loadRequestId?: number,
  ): T | null {
    if (!fileInfo) return fileInfo;
    const id = Number(fileInfo.id);
    if (!Number.isInteger(id) || id <= 0) return fileInfo;

    const state = stateByFileId.get(id);
    if (!state) return fileInfo;

    const loadedFlag = Number(fileInfo[field] ?? 0);
    const isStaleLoad =
      state.committedAtLoadSeq !== null &&
      (loadRequestId === undefined || loadRequestId <= state.committedAtLoadSeq);

    if (isStaleLoad && state.confirmed !== null) {
      (fileInfo as Record<string, any>)[field] = field === 'is_favorite' ? Boolean(state.confirmed) : state.confirmed;
    } else if (!state.inFlight || state.confirmed === null) {
      state.confirmed = loadedFlag;
    }

    if (state.desired !== null) {
      (fileInfo as Record<string, any>)[field] = field === 'is_favorite' ? Boolean(state.desired) : state.desired;
    }
    return fileInfo;
  }

  function recordExternalMetadata(fileId: number, value: number) {
    if (!Number.isInteger(fileId) || fileId <= 0) return;
    const state = getOrCreateState(fileId);
    if (!state.inFlight) {
      state.confirmed = Math.max(0, Math.min(max, value));
      state.committedAtLoadSeq = loadSeq;
    }
  }

  function enqueueMetadataUpdate(options: EnqueueMetadataUpdateOptions): Promise<void> {
    const { fileId } = options;
    const normalized = Math.max(0, Math.min(max, options.value));
    const state = getOrCreateState(fileId);

    if (!state.inFlight && options.knownConfirmed !== null) {
      state.confirmed = Math.max(0, Math.min(max, options.knownConfirmed));
    }

    state.desired = normalized;
    state.pendingAdvance = { pane: options.pane, advance: options.advance };
    if (options.requestId !== undefined) state.pendingAdvance.requestId = options.requestId;
    options.applyOptimistic(fileId, normalized);

    if (state.inFlight && state.activePromise) {
      return state.activePromise;
    }

    state.inFlight = true;
    const run = (async () => {
      try {
        while (state.desired !== null) {
          const target = state.desired;
          const result = await options.persist(fileId, target).catch(() => null);

          if (result !== null) {
            state.confirmed = target;
            state.committedAtLoadSeq = loadSeq;
            const settled = state.desired === target;
            const advanceInfo = settled ? state.pendingAdvance : null;
            if (settled) {
              state.desired = null;
              state.pendingAdvance = null;
            }
            options.onCommit(fileId, target, settled, advanceInfo);
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
    applyPendingMetadata,
    recordExternalMetadata,
    enqueueMetadataUpdate,
  };
}
