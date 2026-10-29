// Serialize backfill attempts; a switch or index completion during an attempt
// schedules one more attempt instead of losing that notification.
export function createRawPairBackfill({ run, libraryId, onSuccess, onError }) {
  let running = false;
  let pending = false;
  let disposed = false;
  let waitingForIndex = false;
  async function request() {
    if (disposed) return;
    pending = true;
    if (running) return;
    running = true;
    try {
      while (pending && !disposed) {
        pending = false;
        const id = libraryId();
        try {
          const complete = await run();
          if (disposed) return;
          if (id !== libraryId()) { pending = true; continue; }
          waitingForIndex = !complete;
          if (complete) await onSuccess();
          // A busy result is retried on index_finished or an explicit request.
        } catch (error) {
          if (!disposed && id === libraryId()) onError(error);
        }
      }
    } finally {
      running = false;
    }
  }
  return {
    request,
    indexFinished() { if (running || waitingForIndex) void request(); },
    dispose() { disposed = true; },
  };
}
