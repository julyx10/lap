# Embedded metadata reader

Lap supplements its existing EXIF, LibRaw and video readers with the pinned
`exiftool-rs 1.0.0` Rust library. No external Perl or ExifTool process is bundled;
the library runs the ExifTool 13.60 code through its own Rust interpreter.
The extraction backend is read-only; it never writes media or sidecars.
Manual **Save Metadata to XMP** writes favorites, ratings, culling flags, comments and tags
using Lap’s independent XML writer. It preserves unrelated properties and never
writes the original media. There is no automatic sidecar export.

This is an internal implementation and validation note, not website content.

## Import policy

- Existing technical fields win. Deep extraction fills missing camera, lens,
  original capture date and complete latitude/longitude pairs before geocoding.
- Automatic organization metadata import uses the existing XMP import setting.
  Favorites, ratings, pick/reject flags, keywords and descriptions may now come from embedded
  XMP/IPTC/EXIF. Sidecar properties take precedence, including explicit empties.
- The existing non-destructive import mode and dirty bits protect local edits
  and clears. This is fill-empty/merge behavior, not bidirectional synchronization.
- Scan and file-info refresh import automatically when enabled. Refresh bypasses
  fingerprints while retaining local-edit protection; the overwrite-import menu
  action has been removed.
- Media and sidecar timestamps/sizes are recorded together. Unchanged scans skip
  organization extraction; an upgrade backfills embedded organization metadata
  once. Technical-field backfills require a file-info refresh/rescan that actually
  re-extracts file info, not just an unchanged folder scan.
- A per-thread, single-file mapped-value cache avoids the duplicate extraction
  between file-info construction and organization import. Cloud-only files are
  not hydrated by this backend. Unsupported formats fall back to existing readers.
- The 1.0 path API reads metadata without the bytes API's full-file copy.
  Namespace-aware XMP parsing still uses a read-only mapping. PSD/HEIF packets
  come from an explicitly requested raw `XMP` tag, restricted to the main document;
  preview-document packets do not supply organization data. Other container
  packet walkers and their size limits are unchanged.
- Cargo defaults PCRE2 to a bundled static build so portable app packages do not
  depend on a build-machine Homebrew/system PCRE2 library. Distribution builders
  may explicitly override `PCRE2_SYS_STATIC` when supplying a system dependency.
- Unknown fields are not persisted. Flat keywords and explicit hierarchical
  keyword paths are distinguished; only the latter create parent/child tags.

## Known limits and upstream cooperation

ORF IIRO header support was fixed in 0.14.0 and is covered by a
26-byte synthetic camera-make regression. All 45 formerly failing local ORF
samples read successfully in the 0.14.0 comparison. Original upstream report:
https://github.com/Le-Syl21/exiftool-rs/issues/24

Local comparisons also found missing video GPS and some lens fields. Existing
values must not be replaced by these missing values. GPS altitude remains with
the established reader pending verified sign/reference handling. Numeric XP tag
display strings must never become keywords; XPKeywords is decoded as UTF-16LE.
Synthetic MakerNote zero ratings are ignored. Source groups are retained during
mapping to avoid name-only duplicate selection for ratings and organization fields.

Historical reports from the 0.14.0 comparison (links retained for traceability;
this is not a statement of their current upstream status):
- XMP numeric GPS normalization: https://github.com/Le-Syl21/exiftool-rs/issues/25
- QuickTime GPS composites: https://github.com/Le-Syl21/exiftool-rs/issues/26
- XPKeywords text decoding: https://github.com/Le-Syl21/exiftool-rs/issues/27
- Samsung lens field mapping: https://github.com/Le-Syl21/exiftool-rs/issues/28

## Validation

`python3 scripts/test_metadata_reader.py` compiles the actual reader and XMP
modules in an isolated harness and runs parsing fixtures plus in-memory SQLite
regressions. The filesystem cloud-state check and global DB opener are stubbed;
the full file scanner and UI are not covered by this harness. The actual platform
`rename_exclusive` function is included using checked source markers.

Set `LAP_TEST_EXIFTOOL=1` to independently read exported values with an installed
ExifTool. Run frontend action tests with `cd src-vite && npm run test:xmp`.

`cargo check --locked --offline --manifest-path src-tauri/Cargo.toml` validates
the complete application on the host platform. Windows/Linux builds and installer
execution still require the normal platform CI; host checks do not prove them.

Tests cover embedded JPEG XMP, comma-containing keywords, numeric garbage,
coordinate bounds/signs, unsupported formats, sidecar priority and protection of
local clears, alongside the existing sidecar suite.

The library is GPL-3.0-or-later, matching Lap. Release source archives must include
the pinned dependency and its notices in addition to Lap's build sources.

## Export compatibility

The catalog stores a tag tree. `dc:subject` contains the explicitly assigned tag
names, deduplicated for interoperability; `lr:hierarchicalSubject` contains their
full paths using `|`. Existing `digiKam:TagsList` fields use `/` for the same
paths. Both representations normalize to one tree on import; flat subjects keep
literal separators. Older pipe-separated digiKam paths are also accepted.
`lr:hierarchicalSubject` is created when valid catalog paths are nonempty;
an existing field is also rewritten when empty to clear stale assignments.
An absent field is not created for an empty path set.
ACDSee `categories` is imported from its escaped XML tree (embedded or sidecar).
Only nodes with `Assigned="1"` are linked; unassigned ancestors supply the path.
Matching paths across ACDSee, Lightroom and digiKam share the same nodes.
Malformed ACDSee XML is ignored without discarding other metadata. Existing
ACDSee fields are rewritten with the current assignments, including explicit
clears, so stale categories cannot restore removed tags. No ACDSee field is
created when it was absent. Paths with a literal `/` in a node name are omitted
from digiKam's ambiguous slash encoding; Lightroom and dc retain those tags.

Keyword format compatibility lives in `src-tauri/src/t_xmp_compat.rs`. Its registry
defines namespace URI/property, backend fallback name, encoding and write policy.
Embedded packets and sidecars use the same URI-aware importer; the generic writer
uses the registry to identify and replace writable fields. New keyword schemas
and encodings belong in this module, not in scanner or database branches. The
backend mapping is only a fallback when container packets cannot be extracted.
An additional test-only schema verifies that registering a format here enables
embedded import, sidecar parsing and writing without editing the callers.
Parent nodes organize tags and expand searches, but are not implicitly assigned
to images. Only an existing `darktable:rejected` field is synchronized.

Migration v21 removes tag groups, retaining every tag ID, name and file link as a
root tag. It neither reads images nor writes XMP. Legacy names containing literal
`|` stay intact and export only as flat keywords; they are never guessed into a
hierarchy. Root and sibling names have separate case-insensitive unique indexes.
An upgrade encountering historical case collisions rolls back rather than merging
or renaming user tags silently.

XMP source-path aliases follow local tag renames and subtree moves; deletion
leaves a tombstone so later import cannot recreate the removed source path.
Local subtree edits protect all affected file tag sets. File-info and XMP export
resolve only the ancestors of assigned tags. Parent queries and counters include
descendants, counting each file once. Tree edits reject cycles and name collisions.
Import fingerprint v4 schedules one metadata re-read on the next scan, even when
the media and sidecar are unchanged, to apply hierarchy and vendor-schema parsing
to existing files. Local-edit protection remains in effect; thumbnails are not rebuilt.

The catalog snapshot transaction ends before sidecar I/O begins. Embedded
metadata parsing shares a read-only file mapping with URI-aware XMP parsing;
only mapping failures fall back to a full file read, matching the backend.

Batch exports build one directory index per visited folder, shared by sidecar
lookup and RAW/JPEG stem ambiguity checks. Successful publications update that
index; the index is dropped at the end of the batch. Discovery work is therefore
linear in visited directory entries plus selected files, rather than repeating
whole-directory enumeration for every selected file.

Only direct photo-subject RDF Description properties are imported; nested and
identified resources remain preserved XML rather than becoming photo metadata.
JPEG 2000 UUID XMP packets use the same bounded container extraction as other
supported ISO box formats.

A non-unique `atags(name COLLATE NOCASE)` index supports case-insensitive tag
lookup without merging or rejecting existing differently-cased tag rows. Existing
libraries build this index once during schema initialization.

## File-info refresh contract

`check_selected_file_info` resolves each file's current catalog path under the
library-switch lock and checks the catalog record and preview cache. The check
uses catalog values, directory discovery and filesystem metadata only. It never
reads original or sidecar contents, extracts metadata or downloads online-only
files. A file needs updating when any of these conditions holds:

- `metadata_pending` is set and the original is available locally.
- `last_scan_time` is missing or nonpositive.
- The original's modification time differs from the successfully indexed version.
- The original's current size differs from its catalog size.
- Automatic XMP import is enabled and the selected sidecar's modification time is
  later than `last_scan_time`.

Missing technical fields alone do not trigger an update. Missing or inaccessible
files and failed checks are reported as errors, not classified as up to date.
Disabling automatic XMP import excludes sidecars from this decision.

A normal multi-selection dialog uses two separate library-bound index sessions:
one for checking the selection, closed before confirmation, and a new one for
refreshing the confirmed files. Each stage releases its session in `finally`,
including cancellation and partial failures. Commands share their stage's index
across background worker threads; expired sessions are reclaimed on subsequent
creation. Download/index/free-space mode goes directly to a refresh session.

Scan and selection refresh update changed originals and repair missing/stale
previews. Scan covers the album; selection refresh covers the selection. Missing
previews alone do not trigger metadata extraction. Metadata backfill and
sidecar-only imports retain valid previews; there is no separate rebuild option.
The existence check uses database metadata and filesystem attributes without
loading or decoding preview bytes.

Browsing checks the viewport and bounded prefetch once per visit. It only fills
missing previews (including absent/empty cache files), never refreshes file info,
imports XMP, consumes pending work or regenerates an existing preview because its
original changed. Cache reads always reuse existing previews without validating
source timestamps or RAW options, and preserve invalidation markers for
scan/refresh. Display decode failures request one repair; successful repair
invalidates the frontend's old cache without reloading file info or the original.
Explicit RAW display-setting changes still request their own preview replacement.

Generation shares a four-task budget and per-file locking. Workers recheck after
acquiring the lock, preserve old cache until replacement succeeds, and reject
results if the original changes during extraction/generation. Automatic decode
failures are suppressed for the same source/options for 30 seconds, with at most
1024 entries; explicit scan/refresh can retry. Online-only or unavailable
originals keep their cached previews and are never downloaded by these operations.
Checking online-only attributes during metadata refresh advances the check
timestamp but retains pending extraction work.

Automatic XMP import failures have different reporting contracts:

- Scan/import preserves the successful primary file operation, logs the XMP
  failure and sets `metadata_pending` for retry. This applies only when automatic
  import is enabled; database failures can still propagate normally.
- Explicit refresh keeps successfully extracted catalog metadata and returns the
  XMP failure in `FileRefreshResult.warning`. Local files remain pending for retry.
  The caller keeps the returned file even when a later stage reports an error.
- An unchanged online-only original also returns sidecar import errors in its
  skipped result and advances `last_scan_time`, avoiding repeated offers for the
  same unchanged broken sidecar without downloading the original.

The dialog reports XMP-only results in a separate warnings count.
Thumbnail-generation and release errors remain failures. If both a warning and an
error occur for one file, it is counted once as failed and both messages remain
visible. Successfully updated metadata is retained in all returned results.
Single-file refresh likewise distinguishes warning and error notifications.

## Lap Favorite schema

Namespace URI: `https://github.com/julyx10/lap/ns/1.0/` (stable identifier).
Property: `Favorite`, Boolean, emitted as `True` or `False`. Prefixes are arbitrary;
readers match the URI and local property name. Attribute and element forms are
accepted. Missing or invalid values remain unknown, not explicit false. A sidecar
value overrides an embedded value. This is a Lap extension, not a cross-application
Favorite standard; ratings, pick flags and keyword/label text never imply it.

Automatic imports preserve existing favorites and locally edited/cleared states.
Local favorite edits use dirty bit 16, independent of rating and culling. Manual
sidecar export writes the current Boolean value, including false, without clearing
local-edit protection. The embedded import fingerprint is versioned so unchanged libraries
backfill this field once after upgrading. Both flags share one XML parsing pass.


## On-demand file details

File Info loads extended source, capture and GPS direction fields only when their
section is expanded. These read-only values have no new catalog columns or indexes.
Existing artist/copyright/description/software columns are supplemented during
scan/refresh; existing nonempty reader values win. Artist has a NOCASE index.

Source fields prefer sidecar XMP, embedded XMP, IPTC and EXIF. Namespace-aware XML
preserves multiple creators, language alternatives and explicit empty properties.
The editable catalog description remains independent. Inspection never writes
sidecars, downloads cloud-only files or depends on the organization import switch.
Time-zone offsets are displayed without changing catalog timestamp semantics.

Concurrent sections share one embedded extraction per media path/size/mtime.
Parsing runs off the UI thread with at most two concurrent requests. The cache
retains up to 16 idle file entries and 4 MiB of value text; active requests may
briefly exceed these bounds. Only requested fields are returned. Source-sidecar
discovery/parsing is lazy and separately cached by path/size/mtime. A shared
client/generation key coalesces forced refreshes across sections.

The panel keeps target-file snapshots while refreshing, replaces section values
atomically, and discards stale file/library responses. Only a successful empty
result hides a section; unread/error sections remain accessible with retry.
Collapse, file/library changes, refresh and unmount cancel pending requests.
Cancellation is checked after the concurrency permit and per-file lock; bounded,
expiring tombstones handle cancellation arriving before registration.


## Stored image properties

`afiles.icc_profile`, `color_space` and nullable integer `bit_depth` are populated
by the existing scan/file-refresh extraction. Migration v20 adds empty
columns and marks historical images/RAW as `metadata_pending`, without reading
media during migration. Scan or explicit refresh extracts those fields once and
clears the marker on successful catalog update, even when values remain empty.
Online-only files keep their marker until available locally. Metadata-only
backfill preserves thumbnails and embeddings. File
Info renders only nonempty catalog values, including the selected RAW companion's
values. Opening or expanding File never requests these properties; the former
image-detail request group and its separate insertion animation are removed.
Conflicting bit depths (for example original versus embedded preview) remain NULL.
QuickTime sample-entry BitDepth is excluded because it is not per-channel depth.
Scan mapping selects only persisted fields, without constructing the extended
detail rows. No search indexes are added for these three fields.


## XMP file lifecycle

Single and batch rename, move (including outside the library), copy, delete and
file/date-organized import carry XMP independently of the import setting. Folder
operations already carry their contents. XMP bytes are never rewritten.

Both `photo.xmp` and `photo.raw.xmp` are supported; only the extension is folded.
A shared stem is retained while another owner remains. A separated file receives
a full-name sidecar; paired files moved together can retain a common stem. After
all original owners move, unchanged orphaned source sidecars are cleaned up.
Full-name metadata takes precedence over another file's shared stem.

Metadata is staged with exclusive destinations and rolls back if the primary
operation or catalog update fails. Existing destination XMP is never overwritten,
even with media Replace; keep-both transfers search for a conflict-free name.
Unclaimed target XMP is also treated as a conflict, preventing accidental adoption.
Sidecar symlinks are rejected. Unknown sibling formats conservatively retain
shared metadata. Deletion removes XMP only after the final owner is gone, using
the same Trash/permanent choice as the media operation.

Batch planning and date-organized imports use operation-local directory indexes.
Multi-file move/copy actions (including internal drops) share an explicit transfer
batch index between conflict checks and publication. Successful transfers update
ownership and sidecar entries in place, including shared-stem cleanup; failures
discard the index. Before reusing a directory, a metadata check detects outside
entry changes and rebuilds only that directory. Unix identity/ctime also detects
replacement and changes with restored mtime. Normal batch mutations update the
saved directory stamp without enumerating it again. A batch serializes its
transfers and ends in a frontend `finally` block. Single-file commands use one index for planning and execution.
Date-organized duplicate detection compares XMP as well as media/AAE content,
using the same sidecar selection as the copy operation. Publication, rollback
and deletion invalidate scan/refresh lookup caches across threads. Small updates
patch the current scan index; removals rebuild it lazily when necessary.
The isolated metadata harness exercises ownership, conflicts, rollback, paired
files, naming, symlinks and permanent deletion on temporary files; native Trash,
cross-volume moves and full desktop/database integration need platform QA.
