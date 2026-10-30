//! Read-only embedded metadata. Existing EXIF/LibRaw values remain authoritative.
//! Keep the backend boundary here so unsupported formats never prevent import.
use crate::t_xmp_compat::push_keyword;
use exiftool_rs::{ExifTool, Options, Tag, Value};
use std::path::Path;

#[derive(Clone, Debug, Default)]
pub struct Metadata {
    pub make: Option<String>,
    pub model: Option<String>,
    pub artist: Option<String>,
    pub copyright: Option<String>,
    pub description: Option<String>,
    pub software: Option<String>,
    pub icc_profile: Option<String>,
    pub color_space: Option<String>,
    pub bit_depth: Option<u16>,
    pub lens_make: Option<String>,
    pub lens_model: Option<String>,
    pub date: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<f64>,
    pub organization: crate::t_xmp::XmpMetadata,
}

// A single entry per scanning thread avoids parsing twice (file info + DB import).
// Cache only mapped values, not the potentially large full tag collection.
thread_local! {
    static LAST: std::cell::RefCell<Option<(String, Metadata)>> = const { std::cell::RefCell::new(None) };
}

pub fn read(path: &Path) -> Metadata {
    let Ok(info) = std::fs::metadata(path) else {
        return Metadata::default();
    };
    if crate::t_utils::is_cloud_only(&info) {
        return Metadata::default();
    }
    let key = format!(
        "{}:{:?}:{}",
        path.display(),
        info.modified().ok(),
        info.len()
    );
    LAST.with(|cache| {
        if let Some((previous, values)) = cache.borrow().as_ref() {
            if previous == &key {
                return values.clone();
            }
        }
        // Reuse the bytes for URI-aware XMP flags: the backend's Tag drops
        // namespace URIs for unregistered schemas and retains arbitrary prefixes.
        let values = read_uncached(path).unwrap_or_default();
        *cache.borrow_mut() = Some((key, values.clone()));
        values
    })
}

// Keep the mapping scoped to one parse: large video payloads are never copied
// into an owned buffer. Mapping failures retain the backend's read fallback.
fn read_uncached(path: &Path) -> std::io::Result<Metadata> {
    let file = std::fs::File::open(path)?;
    if file.metadata()?.len() == 0 {
        return Ok(Metadata::default());
    }
    // SAFETY: this is a read-only mapping, used synchronously and dropped before
    // returning. As with the backend's existing mmap reader, the source must not
    // be truncated by another process while parsing.
    match unsafe { memmap2::Mmap::map(&file) } {
        Ok(bytes) => Ok(from_bytes(&bytes, path)),
        Err(_) => std::fs::read(path).map(|bytes| from_bytes(&bytes, path)),
    }
}

fn from_bytes(bytes: &[u8], path: &Path) -> Metadata {
    let options = Options {
        duplicates: true,
        ..Default::default()
    };
    let Ok(tags) = ExifTool::with_options(options).extract_info_from_bytes(bytes, path) else {
        return Metadata::default();
    };
    // Empty/unknown XMP properties may produce no backend tags at all.
    // The bounded container walker reads metadata blocks, never pixel payloads.
    let packets = embedded_packets(bytes);
    let mut metadata = map_tags(&tags, packets.is_empty());
    if !packets.is_empty() {
        // One bounded packet extraction and URI-aware parse for every schema.
        let xmp = crate::t_xmp::metadata_from_packets(&packets);
        metadata.organization.culling_flag =
            xmp.culling_flag.or(metadata.organization.culling_flag);
        metadata.organization.favorite = xmp.favorite;
        if xmp.description.is_some() {
            metadata.organization.description = xmp.description;
        }
        if let Some(values) = xmp.tags {
            let tags = metadata.organization.tags.get_or_insert_with(Vec::new);
            for value in values {
                push_keyword(&value, tags);
            }
        }
        if let Some(values) = xmp.hierarchical_tags {
            let paths = metadata
                .organization
                .hierarchical_tags
                .get_or_insert_with(Vec::new);
            for value in values {
                push_keyword(&value, paths);
            }
        }
    }
    metadata
}

pub(crate) fn decode_xmp_text(bytes: &[u8]) -> Option<String> {
    xmp_packets::decode(bytes)
}

/// Resolve flags from their XML namespace URI, never from a user-chosen prefix.
/// Only inspect container-designated metadata. PNG iTXt may compress its packet;
/// other supported containers are handled by the bounded packet extractor.
fn embedded_packets(bytes: &[u8]) -> Vec<String> {
    const LIMIT: usize = 16 * 1024 * 1024;
    let packets = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        let mut packets = Vec::new();
        let mut pos = 8usize;
        while pos.checked_add(12).is_some_and(|end| end <= bytes.len()) {
            let len = u32::from_be_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
            let Some(end) = pos
                .checked_add(12)
                .and_then(|p| p.checked_add(len))
                .filter(|end| *end <= bytes.len())
            else {
                break;
            };
            if &bytes[pos + 4..pos + 8] == b"iTXt" && packets.len() < 32 {
                if let Some(packet) = png_xmp_packet(&bytes[pos + 8..pos + 8 + len], LIMIT) {
                    packets.push(packet);
                }
            }
            if &bytes[pos + 4..pos + 8] == b"IEND" {
                break;
            }
            pos = end;
        }
        packets
    } else {
        xmp_packets::extract(bytes)
    };
    let packets: Vec<_> = packets
        .into_iter()
        .filter_map(|p| xmp_packets::decode(&p))
        .collect();
    packets
}

#[cfg(test)]
fn embedded_flags(bytes: &[u8]) -> (Option<i32>, Option<bool>) {
    crate::t_xmp::flags_from_packets(&embedded_packets(bytes))
}

#[cfg(test)]
fn embedded_culling(bytes: &[u8]) -> Option<i32> {
    embedded_flags(bytes).0
}

#[path = "t_xmp_packets.rs"]
mod xmp_packets;

fn png_xmp_packet(chunk: &[u8], limit: usize) -> Option<Vec<u8>> {
    use std::io::Read;
    let keyword_end = chunk.iter().position(|b| *b == 0)?;
    if &chunk[..keyword_end] != b"XML:com.adobe.xmp" {
        return None;
    }
    let compressed = *chunk.get(keyword_end + 1)?;
    if *chunk.get(keyword_end + 2)? != 0 {
        return None;
    }
    let mut pos = keyword_end + 3;
    for _ in 0..2 {
        pos += chunk.get(pos..)?.iter().position(|b| *b == 0)? + 1;
    }
    let data = chunk.get(pos..)?;
    let packet = match compressed {
        0 if data.len() <= limit => data.to_vec(),
        1 => {
            let mut packet = Vec::new();
            flate2::read::ZlibDecoder::new(data)
                .take(limit as u64 + 1)
                .read_to_end(&mut packet)
                .ok()?;
            if packet.len() > limit {
                return None;
            }
            packet
        }
        _ => return None,
    };
    Some(packet)
}

fn text(tag: &Tag) -> Option<String> {
    match &tag.raw_value {
        Value::String(value) if !value.trim().is_empty() && !value.contains('\0') => {
            Some(value.trim().to_owned())
        }
        _ => None,
    }
}

fn source_rank(tag: &Tag) -> u8 {
    match tag.group.family0.as_str() {
        "XMP" => 0,
        "IPTC" => 1,
        "EXIF" => 2,
        _ => 3,
    }
}

#[cfg(test)]
fn from_tags(tags: &[Tag]) -> Metadata {
    map_tags(tags, true)
}

fn map_tags(tags: &[Tag], fallback_xmp_keywords: bool) -> Metadata {
    let string = |name: &str| {
        tags.iter()
            .filter(|t| t.name == name)
            .filter_map(|t| text(t).map(|value| (source_rank(t), value)))
            .min_by_key(|(rank, _)| *rank)
            .map(|(_, value)| value)
    };
    // Only accept complete coordinate pairs from the same metadata directory.
    let mut gps = None;
    for lat in tags.iter().filter(|t| t.name == "GPSLatitude") {
        let Some(lon) = tags
            .iter()
            .find(|t| t.name == "GPSLongitude" && t.group.family1 == lat.group.family1)
        else {
            continue;
        };
        let reference = |name: &str| {
            tags.iter()
                .find(|t| t.name == name && t.group.family1 == lat.group.family1)
                .and_then(text)
        };
        if let (Some(a), Some(b)) = (
            coordinate(&lat.raw_value, reference("GPSLatitudeRef").as_deref(), 90.0),
            coordinate(
                &lon.raw_value,
                reference("GPSLongitudeRef").as_deref(),
                180.0,
            ),
        ) {
            gps = Some((a, b));
            break;
        }
    }
    let mut organization = crate::t_xmp::XmpMetadata::default();
    // Ignore synthetic MakerNote zero ratings: absence must not become a clear.
    let rating = tags
        .iter()
        .filter(|t| t.name == "Rating")
        .filter_map(|tag| {
            let value = tag.raw_value.to_display_string().parse::<i32>().ok()?;
            let trusted = matches!(tag.group.family0.as_str(), "XMP" | "EXIF")
                || (tag.group.family0 == "MakerNotes" && value > 0);
            (trusted && (-1..=5).contains(&value)).then_some((source_rank(tag), value))
        })
        .min_by_key(|(rank, _)| *rank);
    if let Some((_, value)) = rating {
        organization.rating = Some(value.max(0));
    }
    // URI-aware pick/good/rejected/label values are supplied by from_bytes.
    // A legacy negative rating remains meaningful without an explicit flag.
    organization.culling_flag = (rating.map(|(_, value)| value) == Some(-1)).then_some(2);
    if fallback_xmp_keywords {
        crate::t_xmp_compat::import_backend(tags, &mut organization);
    }
    let mut keywords = organization.tags.take().unwrap_or_default();
    for tag in tags.iter().filter(|t| t.group.family0 == "IPTC" && t.name == "Keywords") {
        collect_strings(&tag.raw_value, &mut keywords);
    }
    // XPKeywords is UTF-16LE, never the backend's numeric display string.
    for tag in tags.iter().filter(|t| t.name == "XPKeywords") {
        if let Some(value) = xp_keywords(&tag.raw_value) {
            for word in value.split(';') {
                push_keyword(word, &mut keywords);
            }
        }
    }
    if !keywords.is_empty() {
        organization.tags = Some(keywords);
    }
    organization.description = string("Description")
        .or_else(|| string("Caption-Abstract"))
        .or_else(|| string("UserComment"));
    let source = detail_values_for(tags, true);
    let detail = |key: &str| source.get(key).map(|d| d.values.join("; "));
    Metadata {
        artist: detail("creator"),
        copyright: detail("copyright"),
        description: detail("description"),
        software: detail("software"),
        icc_profile: detail("iccProfile"),
        color_space: detail("colorSpace"),
        bit_depth: bit_depth_from_tags(tags),
        make: string("Make"),
        model: string("Model"),
        lens_make: string("LensMake"),
        lens_model: string("LensModel"),
        date: string("DateTimeOriginal"),
        latitude: gps.map(|(a, _)| a),
        longitude: gps.map(|(_, b)| b),
        // Altitude reference differs between EXIF and composite output; leave it
        // to the established reader until source-aware conversion is verified.
        altitude: None,
        organization,
    }
}

// Conflicting main/preview depths are not a reliable per-channel value.
// QuickTime BitDepth describes the sample entry, not bits per color channel.
fn bit_depth_from_tags(tags: &[Tag]) -> Option<u16> {
    let mut depth = None;
    for tag in tags.iter().filter(|tag| {
        matches!(tag.name.as_str(), "BitsPerSample" | "BitDepth")
            && matches!(tag.group.family0.as_str(), "EXIF" | "PNG" | "File")
    }) {
        let raw = tag.raw_value.to_display_string();
        for token in raw
            .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
            .filter(|v| !v.is_empty())
        {
            let value: u16 = token.parse().ok()?;
            if !(1..=64).contains(&value) || depth.is_some_and(|old| old != value) {
                return None;
            }
            depth = Some(value);
        }
    }
    depth
}

fn collect_strings(value: &Value, words: &mut Vec<String>) {
    match value {
        Value::String(word) => push_keyword(word, words),
        Value::List(values) => {
            for value in values {
                collect_strings(value, words);
            }
        }
        _ => {}
    }
}

fn xp_keywords(value: &Value) -> Option<String> {
    let bytes = match value {
        Value::Binary(v) | Value::Undefined(v) => v.clone(),
        Value::List(v) => v
            .iter()
            .map(|v| match v {
                Value::U8(b) => Some(*b),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?,
        Value::String(v) => return Some(v.trim_end_matches('\0').to_owned()),
        _ => return None,
    };
    if bytes.len() % 2 != 0 {
        return None;
    }
    let utf16: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .take_while(|c| *c != 0)
        .collect();
    String::from_utf16(&utf16).ok()
}

fn coordinate(value: &Value, reference: Option<&str>, limit: f64) -> Option<f64> {
    let number = |v: &Value| v.to_display_string().parse::<f64>().ok();
    let mut value = match value {
        Value::List(parts) if parts.len() == 3 => {
            number(&parts[0])? + number(&parts[1])? / 60.0 + number(&parts[2])? / 3600.0
        }
        _ => number(value)?,
    };
    if reference.is_some_and(|r| matches!(r, "S" | "W" | "South" | "West")) {
        value = -value.abs();
    }
    (value.is_finite() && value.abs() <= limit).then_some(value)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    #[test]
    fn keywords_preserve_commas_and_ignore_numeric_garbage() {
        let mut words = Vec::new();
        collect_strings(
            &Value::List(vec![
                Value::String("Paris, France".into()),
                Value::String("darktable|internal".into()),
                Value::U8(0),
            ]),
            &mut words,
        );
        assert_eq!(words, ["Paris, France"]);
        assert_eq!(
            xp_keywords(&Value::Binary(vec![0; 10])).as_deref(),
            Some("")
        );
    }
    #[test]
    fn coordinates_are_validated_and_signed() {
        assert_eq!(coordinate(&Value::F64(181.0), None, 180.0), None);
        assert_eq!(coordinate(&Value::F64(f64::NAN), None, 90.0), None);
        assert_eq!(
            coordinate(
                &Value::List(vec![Value::U8(30), Value::U8(30), Value::U8(0)]),
                Some("S"),
                90.0
            ),
            Some(-30.5)
        );
    }

    #[test]
    fn digikam_and_lightroom_paths_merge_without_splitting_flat_keywords() {
        for lightroom in ["", "<lr:hierarchicalSubject><rdf:Bag><rdf:li>Places|France|Paris</rdf:li></rdf:Bag></lr:hierarchicalSubject>"] {
            let xml = format!(r#"<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"><rdf:Description xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:lr="http://ns.adobe.com/lightroom/1.0/" xmlns:digiKam="http://www.digikam.org/ns/1.0/">
                <dc:subject><rdf:Bag><rdf:li>Paris</rdf:li><rdf:li>literal/slash</rdf:li></rdf:Bag></dc:subject>
                <digiKam:TagsList><rdf:Seq><rdf:li>Places/France/Paris</rdf:li></rdf:Seq></digiKam:TagsList>{lightroom}
                </rdf:Description></rdf:RDF>"#);
            let bytes = jpeg_with_xmp_packet(xml.as_bytes());
            let meta = from_bytes(&bytes, Path::new("paths.jpg")).organization;
            let parsed = crate::t_xmp::parse_xmp_str(&xml).unwrap();
            for metadata in [meta, parsed] {
                assert_eq!(metadata.hierarchical_tags, Some(vec!["Places|France|Paris".into()]));
                let tags = metadata.tags.unwrap();
                assert_eq!(tags.len(), 3);
                assert!(tags.contains(&"literal/slash".to_string()));
                assert!(!tags.contains(&"Places/France/Paris".to_string()));
            }
        }
    }

    #[test]
    fn acdsee_categories_import_assigned_paths_from_embedded_and_sidecar_xmp() {
        let categories = r#"<Categories><Category Assigned="0">Places<Category Assigned="1">Paris &amp; France</Category><Category Assigned="0">Unused</Category></Category><Category Assigned="1">People<Category Assigned="1">Paris</Category></Category></Categories>"#;
        let escaped = quick_xml::escape::escape(categories);
        for property in [format!("a:categories=\"{escaped}\""), String::new()] {
            let body = if property.is_empty() {
                format!("<a:categories><![CDATA[{categories}]]></a:categories>")
            } else {
                String::new()
            };
            let xml = format!(
                r#"<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"><rdf:Description xmlns:a="http://ns.acdsee.com/iptc/1.0/" {property}>{body}</rdf:Description></rdf:RDF>"#
            );
            let bytes = jpeg_with_xmp_packet(xml.as_bytes());
            let meta = from_bytes(&bytes, Path::new("acdsee.jpg")).organization;
            let parsed = crate::t_xmp::parse_xmp_str(&xml).unwrap();
            for metadata in [meta, parsed] {
                assert_eq!(
                    metadata.hierarchical_tags,
                    Some(vec![
                        "Places|Paris & France".into(),
                        "People".into(),
                        "People|Paris".into()
                    ])
                );
                assert_eq!(metadata.tags, metadata.hierarchical_tags);
            }
        }
        assert!(crate::t_xmp_compat::acdsee_category_paths("<Categories><Category>").is_err());
        assert!(crate::t_xmp_compat::acdsee_category_paths("<!DOCTYPE Categories><Categories/>").is_err());
    }

    pub(crate) fn jpeg_with_xmp() -> Vec<u8> {
        let xmp = br#"<x:xmpmeta xmlns:x="adobe:ns:meta/"><rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"><rdf:Description xmlns:xmp="http://ns.adobe.com/xap/1.0/" xmlns:dc="http://purl.org/dc/elements/1.1/" xmp:Rating="4"><dc:subject><rdf:Bag><rdf:li>Paris, France</rdf:li><rdf:li>Travel</rdf:li></rdf:Bag></dc:subject><dc:description><rdf:Alt><rdf:li xml:lang="x-default">Embedded caption</rdf:li></rdf:Alt></dc:description></rdf:Description></rdf:RDF></x:xmpmeta>"#;
        jpeg_with_xmp_packet(xmp)
    }

    pub(crate) fn jpeg_with_xmp_packet(xmp: &[u8]) -> Vec<u8> {
        let mut payload = b"http://ns.adobe.com/xap/1.0/\0".to_vec();
        payload.extend(xmp);
        let mut jpeg = vec![0xff, 0xd8, 0xff, 0xe1];
        jpeg.extend(((payload.len() + 2) as u16).to_be_bytes());
        jpeg.extend(payload);
        jpeg.extend([0xff, 0xd9]);
        jpeg
    }

    #[test]
    fn reads_embedded_xmp_without_splitting_keyword_commas() {
        let tags = ExifTool::new()
            .extract_info_from_bytes(&jpeg_with_xmp(), Path::new("synthetic.jpg"))
            .unwrap();
        let metadata = from_tags(&tags);
        assert_eq!(metadata.organization.rating, Some(4));
        assert_eq!(
            metadata.organization.tags,
            Some(vec!["Paris, France".into(), "Travel".into()])
        );
        assert_eq!(
            metadata.organization.description.as_deref(),
            Some("Embedded caption")
        );
    }

    #[test]
    fn unsupported_format_is_nonfatal() {
        let path = std::env::temp_dir().join(format!("lap-unsupported-{}.bin", std::process::id()));
        std::fs::write(&path, b"not a supported media container").unwrap();
        assert_eq!(
            read(&path).organization,
            crate::t_xmp::XmpMetadata::default()
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn reads_orf_camera_make() {
        // Regression for upstream #24: ORF uses 0x4f52 instead of TIFF's 42.
        let data = [
            b'I', b'I', 0x52, 0x4f, 8, 0, 0, 0, 1, 0, 0x0f, 1, 2, 0, 4, 0, 0, 0, b'O', b'M', 0, 0,
            0, 0, 0, 0,
        ];
        let tags = ExifTool::new()
            .extract_info_from_bytes(&data, Path::new("synthetic.orf"))
            .unwrap();
        assert_eq!(from_tags(&tags).make.as_deref(), Some("OM"));
    }

    #[test]
    fn reads_iptc_keywords_and_caption() {
        let mut iim = Vec::new();
        for (dataset, value) in [
            (25u8, "Paris, France"),
            (25, "Travel"),
            (120, "IPTC caption"),
        ] {
            iim.extend([0x1c, 2, dataset]);
            iim.extend((value.len() as u16).to_be_bytes());
            iim.extend(value.as_bytes());
        }
        let mut payload = b"Photoshop 3.0\0".to_vec();
        payload.extend(b"8BIM\x04\x04\0\0");
        payload.extend((iim.len() as u32).to_be_bytes());
        payload.extend(&iim);
        if iim.len() % 2 != 0 {
            payload.push(0);
        }
        let mut jpeg = vec![0xff, 0xd8, 0xff, 0xed];
        jpeg.extend(((payload.len() + 2) as u16).to_be_bytes());
        jpeg.extend(payload);
        jpeg.extend([0xff, 0xd9]);
        let tags = ExifTool::new()
            .extract_info_from_bytes(&jpeg, Path::new("iptc.jpg"))
            .unwrap();
        let meta = from_tags(&tags).organization;
        assert_eq!(
            meta.tags,
            Some(vec!["Paris, France".into(), "Travel".into()])
        );
        assert_eq!(meta.description.as_deref(), Some("IPTC caption"));
    }

    #[test]
    fn decodes_utf16_xp_keywords() {
        let bytes = "旅行;Paris\0"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        assert_eq!(
            xp_keywords(&Value::Binary(bytes)).as_deref(),
            Some("旅行;Paris")
        );
        assert_eq!(xp_keywords(&Value::Binary(vec![0])), None);
    }

    #[test]
    fn maker_note_zero_does_not_clear_a_rating() {
        let tags = ExifTool::new()
            .extract_info_from_bytes(&jpeg_with_xmp(), Path::new("rating.jpg"))
            .unwrap();
        let mut rating = tags.into_iter().find(|t| t.name == "Rating").unwrap();
        rating.group.family0 = "MakerNotes".into();
        rating.raw_value = Value::U8(0);
        assert_eq!(from_tags(&[rating.clone()]).organization.rating, None);
        rating.raw_value = Value::U8(3);
        assert_eq!(from_tags(&[rating.clone()]).organization.rating, Some(3));
        rating.raw_value = Value::U8(6);
        assert_eq!(from_tags(&[rating]).organization.rating, None);
    }

    #[test]
    fn embedded_culling_matches_sidecar_for_all_supported_states() {
        for (properties, expected) in [
            ("<xmpDM:pick>1</xmpDM:pick>", Some(1)),
            ("<xmpDM:pick>-1</xmpDM:pick>", Some(2)),
            (
                "<xmpDM:pick>0</xmpDM:pick><xmpDM:good>False</xmpDM:good><xmp:Rating>-1</xmp:Rating>",
                Some(0),
            ),
            ("<xmpDM:good>True</xmpDM:good>", Some(1)),
            ("<xmpDM:good>False</xmpDM:good>", Some(2)),
            ("<darktable:rejected>1</darktable:rejected>", Some(2)),
            ("<darktable:rejected>0</darktable:rejected>", Some(0)),
            ("<xmp:Label>Pick</xmp:Label>", Some(1)),
            ("<xmp:Label>Rejected</xmp:Label>", Some(2)),
            ("<xmp:Rating>-1</xmp:Rating>", Some(2)),
            ("<xmp:Label>Red</xmp:Label>", None),
            ("<fake:pick>1</fake:pick>", None),
        ] {
            let xml = format!(
                "<rdf:RDF xmlns:rdf=\"{}\"><rdf:Description xmlns:xmpDM=\"{}\" xmlns:xmp=\"{}\" xmlns:darktable=\"{}\" xmlns:fake=\"urn:not-a-culling-schema\">{properties}</rdf:Description></rdf:RDF>",
                crate::t_xmp::NS_RDF,
                crate::t_xmp::NS_XMP_DM,
                crate::t_xmp::NS_XMP,
                crate::t_xmp::NS_DARKTABLE
            );
            let tags = ExifTool::with_options(Options {
                duplicates: true,
                ..Default::default()
            })
            .extract_info_from_bytes(
                &jpeg_with_xmp_packet(xml.as_bytes()),
                Path::new("flags.jpg"),
            )
            .unwrap();
            assert_eq!(
                from_bytes(
                    &jpeg_with_xmp_packet(xml.as_bytes()),
                    Path::new("flags.jpg")
                )
                .organization
                .culling_flag,
                expected,
                "{properties}; tags={tags:?}"
            );
            assert_eq!(
                crate::t_xmp::parse_xmp_str(&xml).unwrap().culling_flag,
                expected,
                "{properties}"
            );
        }
    }
    #[test]
    fn embedded_culling_resolves_uris_and_rejects_spoofed_prefixes() {
        for (declarations, properties, expected) in [
            (
                format!("xmlns:dm=\"{}\"", crate::t_xmp::NS_XMP_DM),
                "dm:pick=\"1\"",
                Some(1),
            ),
            (
                format!("xmlns:dm=\"{}\"", crate::t_xmp::NS_XMP_DM),
                "dm:good=\"False\"",
                Some(2),
            ),
            (
                format!("xmlns:dt=\"{}\"", crate::t_xmp::NS_DARKTABLE),
                "dt:rejected=\"1\"",
                Some(2),
            ),
            (
                format!("xmlns:r=\"{}\"", crate::t_xmp::NS_XMP),
                "r:Label=\"Pick\"",
                Some(1),
            ),
            (
                "xmlns:xmpDM=\"urn:unrelated\"".into(),
                "xmpDM:pick=\"1\"",
                None,
            ),
            (
                "xmlns:darktable=\"urn:unrelated\"".into(),
                "darktable:rejected=\"1\"",
                None,
            ),
        ] {
            let xml = format!(
                "<meta:xmpmeta xmlns:meta=\"adobe:ns:meta/\"><rdf:RDF xmlns:rdf=\"{}\"><rdf:Description {declarations} {properties}/></rdf:RDF></meta:xmpmeta>",
                crate::t_xmp::NS_RDF
            );
            let bytes = jpeg_with_xmp_packet(xml.as_bytes());
            let meta = from_bytes(&bytes, Path::new("flags.jpg"));
            assert_eq!(meta.organization.culling_flag, expected, "{xml}");
        }
    }

    #[test]
    fn compressed_png_culling_uses_namespace_uri() {
        use std::io::Write;
        let xml = format!(
            "<rdf:RDF xmlns:rdf=\"{}\"><rdf:Description xmlns:dm=\"{}\" dm:pick=\"-1\"/></rdf:RDF>",
            crate::t_xmp::NS_RDF,
            crate::t_xmp::NS_XMP_DM
        );
        for compressed in [false, true] {
            let mut packet = b"XML:com.adobe.xmp\0".to_vec();
            packet.extend([u8::from(compressed), 0, 0, 0]);
            if compressed {
                let mut encoder =
                    flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
                encoder.write_all(xml.as_bytes()).unwrap();
                packet.extend(encoder.finish().unwrap());
            } else {
                packet.extend(xml.as_bytes());
            }
            let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
            bytes.extend((packet.len() as u32).to_be_bytes());
            bytes.extend(b"iTXt");
            bytes.extend(packet);
            bytes.extend([0; 4]);
            assert_eq!(embedded_culling(&bytes), Some(2));
            assert_eq!(
                from_bytes(&bytes, Path::new("flags.png"))
                    .organization
                    .culling_flag,
                Some(2)
            );
        }
        let oversized = vec![b'x'; 65];
        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&oversized).unwrap();
        let mut chunk = b"XML:com.adobe.xmp\0\x01\0\0\0".to_vec();
        chunk.extend(encoder.finish().unwrap());
        assert!(png_xmp_packet(&chunk, 64).is_none());
    }
}

#[cfg(test)]
mod final_review {
    use super::*;
    #[test]
    fn jpeg_comment_is_not_an_xmp_metadata_source() {
        let packet = |flag| {
            format!(
                "<rdf:RDF xmlns:rdf=\"{}\"><rdf:Description xmlns:dm=\"{}\" dm:pick=\"{flag}\"/></rdf:RDF>",
                crate::t_xmp::NS_RDF,
                crate::t_xmp::NS_XMP_DM
            )
        };
        let comment = packet(-1);
        let real = tests::jpeg_with_xmp_packet(packet(1).as_bytes());
        let mut jpeg = vec![0xff, 0xd8, 0xff, 0xfe];
        jpeg.extend(((comment.len() + 2) as u16).to_be_bytes());
        jpeg.extend(comment.as_bytes());
        jpeg.extend(&real[2..]);
        let tags = ExifTool::with_options(Options {
            duplicates: true,
            ..Default::default()
        })
        .extract_info_from_bytes(&jpeg, Path::new("comment.jpg"))
        .unwrap();
        assert!(
            tags.iter()
                .any(|t| t.name == "Pick" && t.raw_value.to_display_string() == "1")
        );
        assert!(
            !tags
                .iter()
                .any(|t| t.name == "Pick" && t.raw_value.to_display_string() == "-1")
        );
        assert_eq!(
            from_bytes(&jpeg, Path::new("comment.jpg"))
                .organization
                .culling_flag,
            Some(1)
        );
    }
}

#[cfg(test)]
mod final_review_encoding {
    use super::*;
    #[test]
    fn utf16_embedded_pick_is_not_lost() {
        let xml = format!(
            "<rdf:RDF xmlns:rdf=\"{}\"><rdf:Description xmlns:xmpDM=\"{}\" xmpDM:pick=\"1\"/></rdf:RDF>",
            crate::t_xmp::NS_RDF,
            crate::t_xmp::NS_XMP_DM
        );
        let mut packet = vec![0xff, 0xfe];
        for unit in xml.encode_utf16() {
            packet.extend(unit.to_le_bytes());
        }
        let jpeg = tests::jpeg_with_xmp_packet(&packet);
        let tags = ExifTool::with_options(Options {
            duplicates: true,
            ..Default::default()
        })
        .extract_info_from_bytes(&jpeg, Path::new("utf16.jpg"))
        .unwrap();
        assert!(tags.iter().any(|t| t.name == "Pick"
            && t.group.family1 == "XMP-xmpDM"
            && t.raw_value.to_display_string() == "1"));
        assert_eq!(
            from_bytes(&jpeg, Path::new("utf16.jpg"))
                .organization
                .culling_flag,
            Some(1)
        );
    }
}

#[cfg(test)]
mod staged_review {
    use super::*;
    #[test]
    fn extended_pick_takes_precedence_over_standard_good() {
        let guid = "0123456789ABCDEF0123456789ABCDEF";
        let packet = |props: &str| {
            format!(
                "<rdf:RDF xmlns:rdf=\"{}\"><rdf:Description xmlns:dm=\"{}\" xmlns:n=\"http://ns.adobe.com/xmp/note/\" {props}/></rdf:RDF>",
                crate::t_xmp::NS_RDF,
                crate::t_xmp::NS_XMP_DM
            )
        };
        let main = packet(&format!("dm:good=\"False\" n:HasExtendedXMP=\"{guid}\""));
        let extended = packet("dm:pick=\"1\"");
        let mut jpeg = tests::jpeg_with_xmp_packet(main.as_bytes());
        jpeg.truncate(jpeg.len() - 2);
        let mut data = b"http://ns.adobe.com/xmp/extension/\0".to_vec();
        data.extend(guid.as_bytes());
        data.extend((extended.len() as u32).to_be_bytes());
        data.extend(0u32.to_be_bytes());
        data.extend(extended.as_bytes());
        jpeg.extend([0xff, 0xe1]);
        jpeg.extend(((data.len() + 2) as u16).to_be_bytes());
        jpeg.extend(data);
        jpeg.extend([0xff, 0xd9]);
        let tags = ExifTool::with_options(Options {
            duplicates: true,
            ..Default::default()
        })
        .extract_info_from_bytes(&jpeg, Path::new("extended.jpg"))
        .unwrap();
        assert!(
            tags.iter()
                .any(|t| t.name == "Pick" && t.raw_value.to_display_string() == "1"),
            "{tags:?}"
        );
        assert_eq!(
            from_bytes(&jpeg, Path::new("extended.jpg"))
                .organization
                .culling_flag,
            Some(1)
        );
    }
}

#[cfg(test)]
mod staged_review_mov {
    use super::*;
    #[test]
    fn mov_without_ftyp_keeps_embedded_pick() {
        let xml = format!(
            "<rdf:RDF xmlns:rdf=\"{}\"><rdf:Description xmlns:dm=\"{}\" dm:pick=\"1\"/></rdf:RDF>",
            crate::t_xmp::NS_RDF,
            crate::t_xmp::NS_XMP_DM
        );
        let atom = |kind: &[u8], data: &[u8]| {
            let mut v = ((data.len() + 8) as u32).to_be_bytes().to_vec();
            v.extend(kind);
            v.extend(data);
            v
        };
        let bytes = atom(b"moov", &atom(b"udta", &atom(b"XMP_", xml.as_bytes())));
        let tags = ExifTool::with_options(Options {
            duplicates: true,
            ..Default::default()
        })
        .extract_info_from_bytes(&bytes, Path::new("flags.mov"))
        .unwrap();
        assert!(
            tags.iter()
                .any(|t| t.name == "Pick" && t.raw_value.to_display_string() == "1"),
            "{tags:?}"
        );
        assert_eq!(
            from_bytes(&bytes, Path::new("flags.mov"))
                .organization
                .culling_flag,
            Some(1)
        );
    }
}

#[cfg(test)]
mod mapped_read_tests {
    use super::*;
    #[test]
    fn mapped_read_matches_bytes_and_handles_empty_and_missing_files() {
        let path = std::env::temp_dir().join(format!(
            "lap-mapped-{}-{}.jpg",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let bytes = tests::jpeg_with_xmp();
        std::fs::write(&path, &bytes).unwrap();
        assert_eq!(read_uncached(&path).unwrap().organization.rating, Some(4));
        std::fs::write(&path, []).unwrap();
        assert_eq!(read_uncached(&path).unwrap().organization.rating, None);
        std::fs::remove_file(&path).unwrap();
        assert!(read_uncached(&path).is_err());
    }
}

#[cfg(test)]
mod deep_review {
    use super::*;
    #[test]
    fn jp2_retains_culling() {
        let xml = format!(
            "<rdf:RDF xmlns:rdf=\"{}\"><rdf:Description xmlns:dm=\"{}\" dm:pick=\"1\"/></rdf:RDF>",
            crate::t_xmp::NS_RDF,
            crate::t_xmp::NS_XMP_DM
        );
        let atom = |kind: &[u8], data: &[u8]| {
            let mut v = ((data.len() + 8) as u32).to_be_bytes().to_vec();
            v.extend(kind);
            v.extend(data);
            v
        };
        let mut bytes = atom(b"jP  ", b"\x0d\x0a\x87\x0a");
        bytes.extend(atom(b"ftyp", b"jp2 \0\0\0\0jp2 "));
        let mut payload = vec![
            0xbe, 0x7a, 0xcf, 0xcb, 0x97, 0xa9, 0x42, 0xe8, 0x9c, 0x71, 0x99, 0x94, 0x91, 0xe3,
            0xaf, 0xac,
        ];
        payload.extend(xml.as_bytes());
        bytes.extend(atom(b"uuid", &payload));
        let tags = ExifTool::with_options(Options {
            duplicates: true,
            ..Default::default()
        })
        .extract_info_from_bytes(&bytes, Path::new("flags.jp2"))
        .unwrap();
        assert!(
            tags.iter()
                .any(|t| t.name == "Pick" && t.raw_value.to_display_string() == "1"),
            "{tags:?}"
        );
        assert_eq!(
            from_bytes(&bytes, Path::new("flags.jp2"))
                .organization
                .culling_flag,
            Some(1)
        );
    }
}

#[cfg(test)]
mod favorite_tests {
    use super::*;
    #[test]
    fn embedded_favorite_is_read_by_uri_in_jpeg_png_and_utf16() {
        use std::io::Write;
        for favorite in [true, false] {
            let xml = format!(
                "<rdf:RDF xmlns:rdf=\"{}\"><rdf:Description xmlns:custom=\"{}\" custom:Favorite=\"{}\"/></rdf:RDF>",
                crate::t_xmp::NS_RDF,
                crate::t_xmp::NS_LAP,
                if favorite { "True" } else { "False" }
            );
            let jpeg = tests::jpeg_with_xmp_packet(xml.as_bytes());
            assert_eq!(
                from_bytes(&jpeg, Path::new("favorite.jpg"))
                    .organization
                    .favorite,
                Some(favorite)
            );
            let mut utf16 = vec![0xff, 0xfe];
            for word in xml.encode_utf16() {
                utf16.extend(word.to_le_bytes());
            }
            assert_eq!(
                from_bytes(
                    &tests::jpeg_with_xmp_packet(&utf16),
                    Path::new("favorite.jpg")
                )
                .organization
                .favorite,
                Some(favorite)
            );
            let mut encoder =
                flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(xml.as_bytes()).unwrap();
            let mut chunk = b"XML:com.adobe.xmp\0\x01\0\0\0".to_vec();
            chunk.extend(encoder.finish().unwrap());
            let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
            png.extend((chunk.len() as u32).to_be_bytes());
            png.extend(b"iTXt");
            png.extend(chunk);
            png.extend([0; 4]);
            assert_eq!(
                from_bytes(&png, Path::new("favorite.png"))
                    .organization
                    .favorite,
                Some(favorite)
            );
        }
    }
}

/// Read-only detail values, deliberately excluded from catalog rows and search indexes.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailValue {
    pub values: Vec<String>,
    pub source: String,
}
pub type Details = std::collections::BTreeMap<String, DetailValue>;

#[derive(Default)]
struct DetailCacheEntry {
    embedded: Option<Details>,
    refresh_key: Option<String>,
    sidecar: Option<(String, Option<String>, Details)>,
}
type SharedDetails = std::sync::Arc<std::sync::Mutex<DetailCacheEntry>>;
fn detail_cache() -> &'static std::sync::Mutex<std::collections::VecDeque<(String, SharedDetails)>>
{
    static CACHE: std::sync::OnceLock<
        std::sync::Mutex<std::collections::VecDeque<(String, SharedDetails)>>,
    > = std::sync::OnceLock::new();
    CACHE.get_or_init(Default::default)
}
fn detail_bytes(details: &Details) -> usize {
    details
        .values()
        .map(|value| value.values.iter().map(String::len).sum::<usize>())
        .sum()
}
fn trim_detail_cache() {
    if let Ok(mut cache) = detail_cache().lock() {
        let mut bytes = cache
            .iter()
            .filter_map(|(_, entry)| entry.try_lock().ok())
            .map(|entry| {
                entry.embedded.as_ref().map(detail_bytes).unwrap_or(0)
                    + entry
                        .sidecar
                        .as_ref()
                        .map(|(_, _, data)| detail_bytes(data))
                        .unwrap_or(0)
            })
            .sum::<usize>();
        let mut index = 0;
        while index < cache.len() && (cache.len() > 16 || bytes > 4 * 1024 * 1024) {
            // Never evict an extraction that another request is using or waiting for.
            if std::sync::Arc::strong_count(&cache[index].1) == 1 {
                let (_, entry) = cache.remove(index).unwrap();
                if let Ok(entry) = entry.lock() {
                    bytes = bytes.saturating_sub(
                        entry.embedded.as_ref().map(detail_bytes).unwrap_or(0)
                            + entry
                                .sidecar
                                .as_ref()
                                .map(|(_, _, data)| detail_bytes(data))
                                .unwrap_or(0),
                    );
                }
            } else {
                index += 1;
            }
        }
    }
}
fn bound_details(details: &mut Details) {
    for value in details.values_mut() {
        value.values = value
            .values
            .iter()
            .take(64)
            .map(|v| v.chars().take(32768).collect())
            .collect();
    }
}
fn extract_detail_values(path: &Path, info: &std::fs::Metadata) -> Result<Details, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    // SAFETY: read-only mapping, scoped to this synchronous extraction, as in read_uncached.
    let mapping = unsafe { memmap2::Mmap::map(&file) };
    let fallback;
    let bytes: &[u8] = match &mapping {
        Ok(mapped) => mapped,
        Err(_) => {
            fallback = std::fs::read(path).map_err(|e| e.to_string())?;
            &fallback
        }
    };
    // requested_tags is a post-extraction filter in the backend. Extract once and
    // keep only our small mapped values, never the complete tag collection.
    let tags = ExifTool::with_options(Options {
        duplicates: true,
        ..Default::default()
    })
    .extract_info_from_bytes(bytes, path)
    .map_err(|e| e.to_string())?;
    let mut details = detail_values(&tags);
    for packet in embedded_packets(bytes) {
        if let Ok(mut source) = crate::t_xmp::source_details(&packet) {
            for value in source.values_mut() {
                value.source = value.source.replace("Sidecar XMP", "Embedded XMP");
            }
            details.extend(source);
        }
    }
    let after = std::fs::metadata(path).map_err(|e| e.to_string())?;
    if after.len() != info.len() || after.modified().ok() != info.modified().ok() {
        return Err("Original changed while reading metadata".into());
    }
    bound_details(&mut details);
    Ok(details)
}
pub fn read_details(
    path: &Path,
    force: bool,
    section: &str,
    refresh_key: Option<&str>,
    cancelled: Option<&std::sync::atomic::AtomicBool>,
) -> Result<Details, String> {
    if !matches!(section, "source" | "capture" | "location") {
        return Err("Unknown metadata section".into());
    }
    let info = std::fs::metadata(path).map_err(|e| e.to_string())?;
    if crate::t_utils::is_cloud_only(&info) {
        return Err("Original is not available locally".into());
    }
    let key = format!(
        "{}:{:?}:{}",
        path.display(),
        info.modified().ok(),
        info.len()
    );
    let shared = {
        let mut cache = detail_cache().lock().map_err(|e| e.to_string())?;
        if let Some(index) = cache.iter().position(|(k, _)| k == &key) {
            let entry = cache.remove(index).unwrap();
            let shared = entry.1.clone();
            cache.push_back(entry);
            shared
        } else {
            let shared = std::sync::Arc::new(std::sync::Mutex::new(DetailCacheEntry::default()));
            cache.push_back((key, shared.clone()));
            shared
        }
    };
    // A per-file lock coalesces simultaneous section requests, including refreshes.
    let result = (|| {
        let mut entry = shared.lock().map_err(|e| e.to_string())?;
        if cancelled.is_some_and(|flag| flag.load(std::sync::atomic::Ordering::Relaxed)) {
            return Err("Metadata request cancelled".into());
        }
        if entry.embedded.is_none()
            || (force && (refresh_key.is_none() || entry.refresh_key.as_deref() != refresh_key))
        {
            entry.embedded = Some(extract_detail_values(path, &info)?);
            entry.refresh_key = refresh_key.map(str::to_owned);
            entry.sidecar = None;
        }
        let mut details = entry.embedded.as_ref().unwrap().clone();
        // Folded source sections never cause sidecar discovery or reading.
        if section == "source" {
            if let Some(path) = crate::t_xmp::find_xmp_sidecar(path) {
                let sidecar_info = std::fs::metadata(&path).map_err(|e| e.to_string())?;
                if !crate::t_utils::is_cloud_only(&sidecar_info) {
                    let key = format!(
                        "{}:{:?}:{}",
                        path.display(),
                        sidecar_info.modified().ok(),
                        sidecar_info.len()
                    );
                    let cached = entry.sidecar.as_ref().is_some_and(|(old, epoch, _)| {
                        old == &key
                            && (!force
                                || (refresh_key.is_some() && epoch.as_deref() == refresh_key))
                    });
                    if !cached {
                        use std::io::Read;
                        let mut xml = String::new();
                        std::fs::File::open(&path)
                            .map_err(|e| e.to_string())?
                            .take(16 * 1024 * 1024 + 1)
                            .read_to_string(&mut xml)
                            .map_err(|e| e.to_string())?;
                        if xml.len() > 16 * 1024 * 1024 {
                            return Err("XMP exceeds 16 MiB size limit".into());
                        }
                        let mut source = crate::t_xmp::source_details(&xml)?;
                        bound_details(&mut source);
                        entry.sidecar = Some((key, refresh_key.map(str::to_owned), source));
                    }
                    details.extend(entry.sidecar.as_ref().unwrap().2.clone());
                }
            }
        }
        details.retain(|key, _| match section {
            "source" => matches!(
                key.as_str(),
                "title"
                    | "creator"
                    | "copyright"
                    | "description"
                    | "credit"
                    | "source"
                    | "usageTerms"
                    | "rightsUrl"
                    | "software"
            ),
            "capture" => matches!(
                key.as_str(),
                "flash"
                    | "whiteBalance"
                    | "exposureMode"
                    | "exposureProgram"
                    | "meteringMode"
                    | "focalLength35mm"
                    | "timeZone"
            ),
            "location" => matches!(key.as_str(), "direction" | "directionRef"),
            _ => false,
        });
        Ok(details)
    })();
    drop(shared);
    trim_detail_cache();
    result
}

fn detail_values(tags: &[Tag]) -> Details {
    detail_values_for(tags, false)
}

fn detail_values_for(tags: &[Tag], stored_only: bool) -> Details {
    let mut result = Details::new();
    let mappings: &[(&str, &[&str])] = &[
        ("title", &["Title", "ObjectName"]),
        ("creator", &["Creator", "By-line", "Artist"]),
        ("copyright", &["Rights", "CopyrightNotice", "Copyright"]),
        (
            "description",
            &["Description", "Caption-Abstract", "ImageDescription"],
        ),
        ("credit", &["Credit"]),
        ("source", &["Source"]),
        ("usageTerms", &["UsageTerms"]),
        ("rightsUrl", &["WebStatement"]),
        ("software", &["CreatorTool", "Software"]),
        ("flash", &["Flash"]),
        ("whiteBalance", &["WhiteBalance"]),
        ("exposureMode", &["ExposureMode"]),
        ("exposureProgram", &["ExposureProgram"]),
        ("meteringMode", &["MeteringMode"]),
        ("focalLength35mm", &["FocalLengthIn35mmFormat"]),
        ("timeZone", &["OffsetTimeOriginal"]),
        ("direction", &["GPSImgDirection"]),
        ("directionRef", &["GPSImgDirectionRef"]),
        ("iccProfile", &["ProfileDescription"]),
        ("colorSpace", &["ColorSpace"]),
    ];
    for (key, names) in mappings {
        if !stored_only && matches!(*key, "iccProfile" | "colorSpace") {
            continue;
        }
        if stored_only
            && !matches!(
                *key,
                "creator" | "copyright" | "description" | "software" | "iccProfile" | "colorSpace"
            )
        {
            continue;
        }
        let descriptive = matches!(
            *key,
            "title"
                | "creator"
                | "copyright"
                | "description"
                | "credit"
                | "source"
                | "usageTerms"
                | "rightsUrl"
                | "software"
        );
        let candidate = tags
            .iter()
            .filter(|tag| names.contains(&tag.name.as_str()))
            .filter(|tag| {
                if descriptive {
                    matches!(tag.group.family0.as_str(), "XMP" | "IPTC" | "EXIF")
                } else {
                    matches!(
                        tag.group.family0.as_str(),
                        "EXIF" | "XMP" | "ICC_Profile" | "PNG" | "File" | "QuickTime"
                    )
                }
            })
            .filter(|tag| !matches!(tag.raw_value, Value::Binary(_) | Value::Undefined(_)))
            .min_by_key(|tag| {
                if descriptive {
                    source_rank(tag)
                } else {
                    if tag.group.family0 == "XMP" { 1 } else { 0 }
                }
            });
        if let Some(tag) = candidate {
            let values = match &tag.raw_value {
                Value::List(items) => items.iter().map(|v| v.to_display_string()).collect(),
                _ => vec![tag.print_value.clone()],
            };
            let values: Vec<_> = values
                .into_iter()
                .filter(|v| !v.trim().is_empty() && !v.contains('\0'))
                .map(|v| v.chars().take(32768).collect())
                .take(64)
                .collect();
            if !values.is_empty() {
                result.insert(
                    (*key).into(),
                    DetailValue {
                        values,
                        source: format!("{}:{}", tag.group.family0, tag.name),
                    },
                );
            }
        }
    }
    result
}

// Cancellation also covers tokens cancelled before their extraction command arrives.
type DetailRequestFlag = std::sync::Arc<std::sync::atomic::AtomicBool>;
type DetailRequestMap = std::collections::HashMap<String, (DetailRequestFlag, std::time::Instant)>;
fn detail_requests() -> &'static std::sync::Mutex<DetailRequestMap> {
    static REQUESTS: std::sync::OnceLock<std::sync::Mutex<DetailRequestMap>> =
        std::sync::OnceLock::new();
    REQUESTS.get_or_init(Default::default)
}
fn prune_detail_requests(requests: &mut DetailRequestMap) {
    requests.retain(|_, (flag, time)| {
        time.elapsed().as_secs() < 300 || std::sync::Arc::strong_count(flag) > 1
    });
}
pub fn begin_detail_request(id: &str) -> Result<DetailRequestFlag, String> {
    if id.is_empty() || id.len() > 128 {
        return Err("Invalid metadata request token".into());
    }
    let mut requests = detail_requests().lock().map_err(|e| e.to_string())?;
    prune_detail_requests(&mut requests);
    if requests.len() >= 1024 && !requests.contains_key(id) {
        return Err("Too many metadata requests".into());
    }
    Ok(requests
        .entry(id.into())
        .or_insert_with(|| (Default::default(), std::time::Instant::now()))
        .0
        .clone())
}
pub fn cancel_detail_requests(ids: &[String]) -> Result<(), String> {
    if ids.len() > 32 {
        return Err("Too many metadata cancellation tokens".into());
    }
    for id in ids {
        begin_detail_request(id)?.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    Ok(())
}
pub struct DetailRequestGuard(pub String);
impl Drop for DetailRequestGuard {
    fn drop(&mut self) {
        if let Ok(mut requests) = detail_requests().lock() {
            requests.remove(&self.0);
        }
    }
}
