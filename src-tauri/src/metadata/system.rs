//! Source mappings into existing catalog fields. No platform attributes are written.
use crate::metadata::keywords::push_keyword;
use crate::metadata::xmp::XmpMetadata;
use exiftool_rs::Tag;
use std::path::Path;

use super::schema::{
    CatalogField as Field, LINUX_FIELDS, MACOS_FIELDS, SystemEncoding as Encoding,
    SystemMapping as Mapping, WINDOWS_FIELDS,
};

fn apply(mapping: &Mapping, values: Vec<String>, meta: &mut XmpMetadata) {
    match mapping.field {
        Field::Tags => {
            let tags = meta.tags.get_or_insert_with(Vec::new);
            for value in values {
                push_keyword(&value, tags);
            }
        }
        Field::Description => {
            if meta
                .description
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
            {
                meta.description = values.into_iter().find(|value| !value.trim().is_empty());
            }
        }
        Field::Favorite | Field::Culling => {
            unreachable!("system sources only map tags, description and rating")
        }
        Field::Rating => {
            if meta.rating.is_none() {
                meta.rating = values
                    .first()
                    .and_then(|value| value.trim().parse::<i32>().ok())
                    .filter(|value| (0..=10).contains(value))
                    .map(|value| (value + 1) / 2);
            }
        }
    }
}

pub fn import_windows_fields(tags: &[Tag], meta: &mut XmpMetadata) {
    for mapping in WINDOWS_FIELDS {
        for tag in tags.iter().filter(|tag| tag.name == mapping.source) {
            let value = match mapping.encoding {
                Encoding::WindowsUtf16 => crate::metadata::embedded::windows_text(&tag.raw_value),
                _ => None,
            };
            if let Some(value) = value {
                let values = match mapping.field {
                    Field::Tags => value.split(';').map(str::to_owned).collect(),
                    _ => vec![value],
                };
                apply(mapping, values, meta);
            }
        }
    }
}

fn decode(mapping: &Mapping, bytes: &[u8]) -> Option<Vec<String>> {
    match mapping.encoding {
        Encoding::FinderTags => {
            let value = plist::Value::from_reader(std::io::Cursor::new(bytes)).ok()?;
            let values = value.as_array()?;
            Some(
                values
                    .iter()
                    .filter_map(|value| {
                        let text = value.as_string()?;
                        // Finder appends a newline and color number (0..7) to the name.
                        let name = match text.rsplit_once('\n') {
                            Some((name, color))
                                if matches!(
                                    color,
                                    "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7"
                                ) =>
                            {
                                name
                            }
                            _ => text,
                        };
                        Some(name.to_owned())
                    })
                    .collect(),
            )
        }
        Encoding::PlistString => {
            let value = plist::Value::from_reader(std::io::Cursor::new(bytes)).ok()?;
            Some(vec![value.as_string()?.to_owned()])
        }
        Encoding::CommaSeparated => Some(
            std::str::from_utf8(bytes)
                .ok()?
                .split(',')
                .map(str::to_owned)
                .collect(),
        ),
        Encoding::Utf8 | Encoding::RatingTen => {
            Some(vec![std::str::from_utf8(bytes).ok()?.to_owned()])
        }
        _ => None,
    }
}

/// Request-local snapshot reused by refresh checks and the subsequent import.
pub(crate) struct SystemMetadata {
    pub metadata: XmpMetadata,
    pub fingerprint: String,
}

/// Read attributes independently of the embedded-metadata cache. Attribute edits
/// need not change the media's size or modification time.
pub fn read_system(path: &Path) -> SystemMetadata {
    let mut meta = XmpMetadata::default();
    let mut fingerprint = blake3::Hasher::new();
    let mappings = if cfg!(target_os = "macos") {
        MACOS_FIELDS
    } else if cfg!(target_os = "linux") {
        LINUX_FIELDS
    } else {
        &[]
    };
    for mapping in mappings {
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        let bytes = xattr::get_deref(path, mapping.source).ok().flatten();
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        let bytes: Option<Vec<u8>> = {
            let _ = path;
            None
        };
        fingerprint.update(mapping.source.as_bytes());
        if let Some(bytes) = bytes {
            fingerprint.update(&(bytes.len() as u64).to_le_bytes());
            fingerprint.update(&bytes);
            if let Some(values) = decode(mapping, &bytes) {
                apply(mapping, values, &mut meta);
            }
        }
    }
    SystemMetadata {
        metadata: meta,
        fingerprint: fingerprint.finalize().to_hex().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::merge::merge_system;
    use super::*;

    #[test]
    fn windows_utf16_fields_map_only_supported_catalog_fields() {
        let tag = |name: &str, text: &str| Tag {
            id: exiftool_rs::TagId::Numeric(0),
            name: name.into(),
            description: String::new(),
            group: exiftool_rs::TagGroup {
                family0: "EXIF".into(),
                family1: "IFD0".into(),
                family2: "Image".into(),
                family3: "Main".into(),
            },
            raw_value: exiftool_rs::Value::Binary(
                text.encode_utf16()
                    .chain([0])
                    .flat_map(u16::to_le_bytes)
                    .collect(),
            ),
            print_value: String::new(),
            priority: 1,
        };
        let mut meta = XmpMetadata::default();
        import_windows_fields(
            &[
                tag("XPKeywords", "旅行;Paris;paris"),
                tag("XPComment", "中文备注; keep punctuation"),
                tag("XPTitle", "Ignored title"),
                tag("XPSubject", "Ignored subject"),
            ],
            &mut meta,
        );
        assert_eq!(meta.tags.unwrap(), ["旅行", "Paris"]);
        assert_eq!(
            meta.description.as_deref(),
            Some("中文备注; keep punctuation")
        );
    }

    #[test]
    fn windows_comment_is_read_through_the_real_file_reader() {
        let text = "Windows 备注; punctuation";
        let utf16: Vec<u8> = text
            .encode_utf16()
            .chain([0])
            .flat_map(u16::to_le_bytes)
            .collect();
        // Little-endian TIFF IFD containing the Windows XPComment tag (40092).
        let mut exif = b"Exif\0\0II\x2a\0\x08\0\0\0".to_vec();
        exif.extend_from_slice(&1u16.to_le_bytes());
        exif.extend_from_slice(&40092u16.to_le_bytes());
        exif.extend_from_slice(&1u16.to_le_bytes());
        exif.extend_from_slice(&(utf16.len() as u32).to_le_bytes());
        exif.extend_from_slice(&26u32.to_le_bytes());
        exif.extend_from_slice(&0u32.to_le_bytes());
        exif.extend_from_slice(&utf16);
        let mut jpeg = vec![0xff, 0xd8, 0xff, 0xe1];
        jpeg.extend_from_slice(&((exif.len() + 2) as u16).to_be_bytes());
        jpeg.extend_from_slice(&exif);
        jpeg.extend_from_slice(&[0xff, 0xd9]);
        let path = std::env::temp_dir().join(format!("lap-xpcomment-{}.jpg", uuid::Uuid::new_v4()));
        std::fs::write(&path, jpeg).unwrap();
        let meta = crate::metadata::embedded::read(&path);
        std::fs::remove_file(path).unwrap();
        assert_eq!(meta.organization.description.as_deref(), Some(text));
    }

    #[test]
    fn finder_binary_plists_strip_only_color_suffixes() {
        let values = plist::Value::Array(vec![
            plist::Value::String("旅行\n6".into()),
            plist::Value::String("literal\nname".into()),
        ]);
        let mut bytes = Vec::new();
        values.to_writer_binary(&mut bytes).unwrap();
        assert_eq!(
            decode(&MACOS_FIELDS[0], &bytes).unwrap(),
            ["旅行", "literal\nname"]
        );
        bytes.clear();
        plist::Value::String("Finder comment".into())
            .to_writer_binary(&mut bytes)
            .unwrap();
        assert_eq!(
            decode(&MACOS_FIELDS[1], &bytes).unwrap(),
            ["Finder comment"]
        );
        assert!(decode(&MACOS_FIELDS[0], b"invalid plist").is_none());
    }

    #[test]
    fn dolphin_fields_decode_and_preserve_embedded_values() {
        let mut system = XmpMetadata::default();
        for (mapping, bytes) in
            LINUX_FIELDS
                .iter()
                .zip([b"Travel,Paris,travel".as_slice(), b"Dolphin comment", b"9"])
        {
            apply(mapping, decode(mapping, bytes).unwrap(), &mut system);
        }
        assert_eq!(system.tags.as_ref().unwrap(), &["Travel", "Paris"]);
        assert_eq!(system.rating, Some(5));
        assert_eq!(system.description.as_deref(), Some("Dolphin comment"));
        let mut embedded = XmpMetadata {
            tags: Some(vec!["Paris".into()]),
            rating: Some(2),
            description: Some("Embedded comment".into()),
            ..Default::default()
        };
        merge_system(&mut embedded, system);
        assert_eq!(embedded.tags.unwrap(), ["Paris", "Travel"]);
        assert_eq!(embedded.rating, Some(2));
        assert_eq!(embedded.description.as_deref(), Some("Embedded comment"));
        for value in ["-1", "11", "invalid"] {
            let mut meta = XmpMetadata::default();
            apply(&LINUX_FIELDS[2], vec![value.into()], &mut meta);
            assert!(meta.rating.is_none());
        }
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[test]
    fn attribute_only_edits_change_import_fingerprint() {
        let path = std::env::temp_dir().join(format!(
            "lap-system-metadata-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        std::fs::write(&path, b"unchanged media").unwrap();
        let mapping = if cfg!(target_os = "macos") {
            &MACOS_FIELDS[1]
        } else {
            &LINUX_FIELDS[1]
        };
        let before = std::fs::metadata(&path).unwrap();
        let mut previous = read_system(&path).fingerprint;
        for comment in ["first", "second"] {
            let bytes = if cfg!(target_os = "macos") {
                let mut bytes = Vec::new();
                plist::Value::String(comment.into())
                    .to_writer_binary(&mut bytes)
                    .unwrap();
                bytes
            } else {
                comment.as_bytes().to_vec()
            };
            xattr::set_deref(&path, mapping.source, &bytes).unwrap();
            let snapshot = read_system(&path);
            assert_eq!(snapshot.metadata.description.as_deref(), Some(comment));
            assert_ne!(snapshot.fingerprint, previous);
            previous = snapshot.fingerprint;
            assert_eq!(
                std::fs::metadata(&path).unwrap().modified().unwrap(),
                before.modified().unwrap()
            );
        }
        std::fs::remove_file(path).unwrap();
    }
}
