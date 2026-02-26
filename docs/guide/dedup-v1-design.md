# Dedup V1 Design (Exact Duplicates)

## Scope
- Goal: detect and review **exact duplicate files** in current library.
- Non-goal: near/similar duplicate matching (pHash/embeddings) in V1.

## Product Behavior
- New sidebar entry: `Duplicates`.
- Show duplicate groups only (group size >= 2).
- Safe action model:
  - `Keep selected` (marks one kept, others selected for delete)
  - `Delete selected` (move to trash, not hard delete)
  - `Mark reviewed` / `Unreviewed`
- Compare workflow:
  - open current group in split image viewer.
  - independent navigation remains, linked zoom optional.

## Matching Rule (V1)
- Duplicate key = `blake3 + file_size`.
- Rationale: high precision, low false positives, faster hashing for large libraries.

## Data Model

### New table: `file_hashes`
- `file_id INTEGER PRIMARY KEY` (FK to files)
- `hash TEXT NOT NULL` (blake3 hex)
- `file_size INTEGER NOT NULL`
- `mtime INTEGER NOT NULL`
- `computed_at INTEGER NOT NULL`
- Indexes:
  - `idx_file_hashes_hash_size (hash, file_size)`
  - `idx_file_hashes_mtime (mtime)`

### New table: `duplicate_groups`
- `id INTEGER PRIMARY KEY`
- `hash TEXT NOT NULL`
- `file_size INTEGER NOT NULL`
- `file_count INTEGER NOT NULL`
- `total_size INTEGER NOT NULL`
- `reviewed INTEGER NOT NULL DEFAULT 0`
- `updated_at INTEGER NOT NULL`
- Unique index: `(hash, file_size)`

### New table: `duplicate_group_items`
- `group_id INTEGER NOT NULL`
- `file_id INTEGER NOT NULL`
- `is_keep INTEGER NOT NULL DEFAULT 0`
- `is_selected INTEGER NOT NULL DEFAULT 0`
- `score REAL NOT NULL DEFAULT 0`
- PK: `(group_id, file_id)`
- Indexes:
  - `idx_dup_items_group (group_id)`
  - `idx_dup_items_file (file_id)`

## Keep Candidate Scoring
Pick best file by deterministic score (higher wins):
1. higher resolution (`width * height`)
2. larger size
3. richer metadata (taken_date, camera fields, gps)
4. edited/favorited/tagged preferred
5. newest mtime

## Backend Pipeline
1. Hash task scans files that need refresh:
   - new file
   - changed mtime/size
   - missing hash
2. Upsert `file_hashes`.
3. Rebuild or incrementally update groups:
   - `GROUP BY hash, file_size HAVING COUNT(*) > 1`
4. For each group, refresh `duplicate_groups` + `duplicate_group_items`.
5. Recompute `is_keep` from score unless user manually overrides.

## API Draft (Tauri commands)

### Scan/Index
- `dedup_start_scan(album_id?: number)` -> `{ job_id }`
- `dedup_get_scan_status(job_id)` -> `{ state, processed, total, groups }`
- `dedup_cancel_scan(job_id)` -> `{ ok }`

### Query
- `dedup_list_groups(params)`
  - params: `{ reviewed?: boolean, offset, limit, sort }`
  - return: `{ total, groups: [{ id, file_count, total_size, reviewed, updated_at, keep_file_id }] }`
- `dedup_get_group(group_id)`
  - return: `{ group, items: [fileInfo...] }`

### Mutations
- `dedup_set_keep(group_id, file_id)`
- `dedup_set_selected(group_id, file_ids, selected)`
- `dedup_mark_reviewed(group_id, reviewed)`
- `dedup_delete_selected(group_id)` -> move selected files to trash and refresh group
- `dedup_delete_selected_bulk(group_ids)`

### Events
- `dedup-scan-progress` `{ job_id, processed, total, groups }`
- `dedup-groups-updated` `{ changed_group_ids }`

## Frontend Flow
1. Enter `Duplicates` view -> call `dedup_list_groups`.
2. Click group -> call `dedup_get_group` and render compare strip/grid.
3. Open compare -> launch split viewer with two files from group.
4. User actions:
   - choose keep
   - select delete candidates
   - delete selected (trash)
   - mark reviewed
5. On delete success:
   - refresh group
   - auto-advance to next unreviewed group.

## UX Defaults
- Default filter: `unreviewed only`.
- Default sort: `largest reclaimable size`.
- Confirm dialog before delete:
  - count, reclaim size, undo hint (trash).

## Performance Notes
- Hashing in background worker, bounded concurrency.
- Batch DB writes inside transaction.
- Cache scan cursor by library.

## V2 Extensions
- Near duplicates (pHash/embedding distance)
- Cross-library dedup
- Import-time duplicate prevention
