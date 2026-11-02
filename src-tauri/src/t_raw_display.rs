//! Per-request RAW display policy. Never stored in global decoder state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawPreviewMode {
    #[default]
    Embedded,
    Rendered,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RawDisplayOptions {
    pub mode: RawPreviewMode,
    pub prefer_pair: bool,
    pub auto_bright: bool,
}

impl RawDisplayOptions {
    pub fn rendered_bright() -> Self {
        Self { mode: RawPreviewMode::Rendered, auto_bright: true, ..Self::default() }
    }

    pub fn embedded(self) -> bool { self.mode == RawPreviewMode::Embedded }
    pub fn cache_tag(self) -> &'static str {
        match (self.mode, self.auto_bright) {
            (RawPreviewMode::Embedded, false) => "embedded-original-v5",
            (RawPreviewMode::Embedded, true) => "embedded-bright-v5",
            // Rendered thumbnails can also fall back to embedded pixels.
            (RawPreviewMode::Rendered, false) => "rendered-original-v4",
            (RawPreviewMode::Rendered, true) => "rendered-bright-v4",
        }
    }
}

pub fn paired_file(file_id: i64, options: RawDisplayOptions) -> Option<crate::t_sqlite::AFile> {
    if !options.prefer_pair { return None; }
    let file = crate::t_sqlite::AFile::get_file_info(file_id).ok()??;
    if file.media_subtype.as_deref() != Some("raw_jpeg_pair") { return None; }
    let companion = crate::t_sqlite::AFile::get_file_info(file.live_photo_video_id?).ok()??;
    if companion.file_type != Some(1) || !std::path::Path::new(companion.file_path.as_deref()?).is_file() { return None; }
    Some(companion)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_display_source_and_brightness_are_independent() {
        let mut tags = std::collections::HashSet::new();
        for mode in ["embedded", "rendered"] {
            for bright in [false, true] {
                let options: RawDisplayOptions = serde_json::from_value(serde_json::json!({
                    "mode": mode, "autoBright": bright, "preferPair": false
                })).unwrap();
                assert_eq!(options.embedded(), mode == "embedded");
                assert_eq!(options.auto_bright, bright);
                tags.insert(options.cache_tag());
            }
        }
        assert_eq!(tags.len(), 4);
        assert!(RawDisplayOptions::default().embedded());
        assert!(!RawDisplayOptions::default().auto_bright);
    }
}
