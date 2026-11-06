//! Existing catalog metadata values and import modes.
use serde::{Deserialize, Serialize};

/// Parsed XMP metadata distinguishing missing properties (`None`) from
/// explicitly empty/cleared properties (`Some(0)`, `Some(vec![])`, `Some("")`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XmpMetadata {
    /// Missing is distinct from explicit false. Independent of ratings and picks.
    pub favorite: Option<bool>,
    /// Star rating (`0..=5`).
    /// - `None`: rating property is omitted from the sidecar.
    /// - `Some(0)`: explicitly unrated (`0` or `""`, or `-1` when mapped to reject).
    /// - `Some(1..=5)`: star rating.
    pub rating: Option<i32>,
    /// Culling flag (`0` = unflagged, `1` = pick, `2` = reject).
    /// - `None`: no culling/pick/reject property is present in the sidecar.
    /// - `Some(0)`: explicitly unflagged.
    /// - `Some(1)`: pick.
    /// - `Some(2)`: reject.
    pub culling_flag: Option<i32>,
    /// Keywords / subject tags (`dc:subject`, `lr:hierarchicalSubject`, `digiKam:TagsList`).
    /// - `None`: no tag container element exists in the sidecar.
    /// - `Some(vec![])`: tag container exists in the sidecar but contains no user tags.
    /// - `Some(tags)`: non-empty list of deduplicated user tags.
    pub tags: Option<Vec<String>>,
    /// Paths explicitly read from hierarchical keyword fields, not inferred from flat text.
    pub hierarchical_tags: Option<Vec<String>>,
    /// User comment / caption (`dc:description`, `exif:UserComment`, `photoshop:Caption`).
    /// - `None`: no description/caption property exists in the sidecar.
    /// - `Some("")`: property exists in the sidecar, but is explicitly empty.
    /// - `Some(text)`: non-empty description text.
    pub description: Option<String>,
}

/// Controls how imported XMP metadata merges with existing database records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XmpImportMode {
    /// Used when a new file is first inserted into `afiles`.
    /// Populates all present non-empty fields from the sidecar.
    InitialScan,
    /// Used during folder rescans or `update_file_info` when a file already exists in `afiles`.
    /// Fills empty fields and merges tags only when the corresponding field has
    /// not been edited locally. Explicit local clears are protected as well.
    RescanNonDestructive,
    /// Explicit refresh bypasses fingerprints, preserving local edits.
    RefreshNonDestructive,
    /// Used when the user explicitly selects "Import from XMP" from the context menu.
    /// Applies all `Some(...)` fields from the sidecar (including `Some(0)` or `Some("")` to clear),
    /// while leaving `None` (missing) fields untouched.
    ManualOverwrite,
}

/// Source text used for display/search; never edited by organization writeback.
#[derive(Clone, Debug, Default)]
pub(crate) struct SourceTitles {
    pub title: Option<String>,
    pub headline: Option<String>,
}
