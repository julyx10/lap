//! XMP keyword interoperability. Add formats here, not in scanners or DB code.
//! Match namespace URIs, never user-chosen prefixes. Keep read/write capabilities
//! explicit: importing a vendor field does not imply that Lap can rewrite it.
use crate::t_xmp::{parse_xml, XmlNode, XmpMetadata, NS_DC, NS_RDF};
use exiftool_rs::{Tag, Value};
use std::collections::{BTreeMap, HashSet};

pub const NS_LR: &str = "http://ns.adobe.com/lightroom/1.0/";
pub const NS_DIGIKAM: &str = "http://www.digikam.org/ns/1.0/";
pub const NS_ACDSEE: &str = "http://ns.acdsee.com/iptc/1.0/";

#[derive(Clone, Copy)]
enum Encoding {
    Flat,
    Path(char),
    Categories,
}

#[derive(Clone, Copy, PartialEq)]
enum WriteRule {
    Always,
    IfPresentOrPaths,
    IfPresent,
    Preserve,
}

struct Format {
    uri: &'static str,
    prefix: &'static str,
    property: &'static str,
    backend_name: &'static str,
    encoding: Encoding,
    container: &'static str,
    write: WriteRule,
}

const FORMATS: &[Format] = &[
    // A test-only schema proves that the parser/writer need no per-app changes.
    #[cfg(test)]
    Format {
        uri: "urn:lap:test-keywords",
        prefix: "testtags",
        property: "paths",
        backend_name: "TestPaths",
        encoding: Encoding::Path('>'),
        container: "Bag",
        write: WriteRule::IfPresent,
    },
    Format {
        uri: NS_DC,
        prefix: "dc",
        property: "subject",
        backend_name: "Subject",
        encoding: Encoding::Flat,
        container: "Bag",
        write: WriteRule::Always,
    },
    Format {
        uri: NS_LR,
        prefix: "lr",
        property: "hierarchicalSubject",
        backend_name: "HierarchicalSubject",
        encoding: Encoding::Path('|'),
        container: "Bag",
        write: WriteRule::IfPresentOrPaths,
    },
    Format {
        uri: NS_DIGIKAM,
        prefix: "digiKam",
        property: "TagsList",
        backend_name: "TagsList",
        encoding: Encoding::Path('/'),
        container: "Seq",
        write: WriteRule::IfPresent,
    },
    Format {
        uri: NS_ACDSEE,
        prefix: "acdsee",
        property: "categories",
        backend_name: "Categories",
        encoding: Encoding::Categories,
        container: "",
        write: WriteRule::IfPresent,
    },
];

fn format(uri: &str, name: &str) -> Option<(usize, &'static Format)> {
    FORMATS
        .iter()
        .enumerate()
        .find(|(_, f)| f.uri == uri && f.property == name)
}

fn normalize_path(value: &str, separator: char) -> String {
    // Older Lap/digiKam TagsList files may use pipes instead of slashes.
    let separator = if value.contains(separator) {
        separator
    } else {
        '|'
    };
    value
        .split(separator)
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("|")
}

pub fn push_keyword(value: &str, values: &mut Vec<String>) {
    let value = value.trim();
    if !value.is_empty()
        && !value.contains('\0')
        && !is_internal_system_tag(value)
        && !values.iter().any(|v| v.eq_ignore_ascii_case(value))
    {
        values.push(value.to_owned());
    }
}

fn is_internal_system_tag(value: &str) -> bool {
    let value = value.trim().to_ascii_lowercase();
    value == "darktable" || value.starts_with("darktable|")
}

fn import_values(meta: &mut XmpMetadata, encoding: Encoding, values: Vec<String>) {
    let tags = meta.tags.get_or_insert_with(Vec::new);
    if !matches!(encoding, Encoding::Flat) {
        meta.hierarchical_tags.get_or_insert_with(Vec::new);
    }
    for value in values {
        let value = match encoding {
            Encoding::Path(separator) => normalize_path(&value, separator),
            _ => value,
        };
        push_keyword(&value, tags);
        if let Some(paths) = meta.hierarchical_tags.as_mut() {
            if !matches!(encoding, Encoding::Flat) {
                push_keyword(&value, paths);
            }
        }
    }
}

pub fn import_properties(properties: &[&XmlNode], meta: &mut XmpMetadata) {
    for node in properties {
        let Some((_, format)) = format(&node.uri, &node.name) else {
            continue;
        };
        let values = match format.encoding {
            // Invalid vendor XML must not discard other valid XMP properties.
            Encoding::Categories => match acdsee_category_paths(&node.value()) {
                Ok(values) => values,
                Err(_) => continue,
            },
            _ => node
                .children
                .iter()
                .filter(|n| n.uri == NS_RDF && matches!(n.name.as_str(), "Bag" | "Seq"))
                .flat_map(|n| &n.children)
                .filter(|n| n.uri == NS_RDF && n.name == "li")
                .map(|n| n.text.clone())
                .collect(),
        };
        import_values(meta, format.encoding, values);
    }
}

/// Fallback for embedded containers whose XMP packet extractor is unavailable.
/// URI-aware packet parsing remains authoritative for aliases and new schemas.
pub fn import_backend(tags: &[Tag], meta: &mut XmpMetadata) {
    fn strings(value: &Value, out: &mut Vec<String>) {
        match value {
            Value::String(value) => out.push(value.clone()),
            Value::List(values) => {
                for value in values {
                    strings(value, out);
                }
            }
            _ => {}
        }
    }
    for tag in tags.iter().filter(|tag| tag.group.family0 == "XMP") {
        let Some(format) = FORMATS.iter().find(|f| {
            tag.name == f.backend_name
                && tag
                    .group
                    .family1
                    .strip_prefix("XMP-")
                    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(f.prefix))
        }) else {
            continue;
        };
        let mut values = Vec::new();
        strings(&tag.raw_value, &mut values);
        if matches!(format.encoding, Encoding::Categories) {
            values = values
                .iter()
                .filter_map(|v| acdsee_category_paths(v).ok())
                .flatten()
                .collect();
            if values.is_empty() {
                continue;
            }
        }
        // Keep backend absence/empty semantics; the XML reader handles explicit clears.
        if !values.is_empty() {
            import_values(meta, format.encoding, values);
        }
    }
}

pub fn writable_property(uri: &str, name: &[u8]) -> bool {
    std::str::from_utf8(name)
        .ok()
        .and_then(|name| format(uri, name))
        .is_some_and(|(_, f)| f.write != WriteRule::Preserve)
}

#[derive(Default)]
pub struct TagWriter {
    present: HashSet<usize>,
}

impl TagWriter {
    pub fn observe(&mut self, uri: &str, name: &str) {
        if let Some((id, _)) = format(uri, name) {
            self.present.insert(id);
        }
    }

    pub fn render(&self, tags: &[String], paths: Option<&[String]>) -> String {
        let mut output = String::new();
        for (id, f) in FORMATS.iter().enumerate() {
            let write = match f.write {
                WriteRule::Always => true,
                WriteRule::IfPresentOrPaths => {
                    self.present.contains(&id) || paths.is_some_and(|paths| !paths.is_empty())
                }
                WriteRule::IfPresent => self.present.contains(&id),
                WriteRule::Preserve => false,
            };
            if !write {
                continue;
            }
            let (prefix, property, uri, container) = (f.prefix, f.property, f.uri, f.container);
            if matches!(f.encoding, Encoding::Categories) {
                let categories = categories_xml(paths.unwrap_or(tags), paths.is_some());
                output += &format!(
                    "<{prefix}:{property} xmlns:{prefix}=\"{uri}\">{}</{prefix}:{property}>",
                    quick_xml::escape::escape(&categories)
                );
                continue;
            }
            output += &format!("<{prefix}:{property} xmlns:{prefix}=\"{uri}\"><rdf:{container}>");
            let values = if matches!(f.encoding, Encoding::Flat) {
                tags
            } else {
                paths.unwrap_or(tags)
            };
            for value in values {
                let value = match f.encoding {
                    Encoding::Path(separator) if paths.is_some() => {
                        // A literal slash in a node name is ambiguous in digiKam.
                        // Omit that representation; LR and dc retain the value.
                        if separator != '|' && value.contains(separator) {
                            continue;
                        }
                        value.replace('|', &separator.to_string())
                    }
                    _ => value.clone(),
                };
                output += &format!("<rdf:li>{}</rdf:li>", quick_xml::escape::escape(&value));
            }
            output += &format!("</rdf:{container}></{prefix}:{property}>");
        }
        output
    }
}

fn categories_xml(values: &[String], hierarchical: bool) -> String {
    #[derive(Default)]
    struct Category {
        assigned: bool,
        children: BTreeMap<String, Category>,
    }
    let mut roots = BTreeMap::<String, Category>::new();
    for value in values {
        let parts: Vec<_> = if hierarchical {
            value.split('|').collect()
        } else {
            vec![value.as_str()]
        };
        let mut children = &mut roots;
        for (index, part) in parts.iter().enumerate() {
            let node = children.entry((*part).to_string()).or_default();
            node.assigned |= index + 1 == parts.len();
            children = &mut node.children;
        }
    }
    fn append(nodes: &BTreeMap<String, Category>, xml: &mut String) {
        for (name, node) in nodes {
            *xml += &format!(
                "<Category Assigned=\"{}\">{}",
                u8::from(node.assigned),
                quick_xml::escape::escape(name)
            );
            append(&node.children, xml);
            *xml += "</Category>";
        }
    }
    let mut xml = String::from("<Categories>");
    append(&roots, &mut xml);
    xml += "</Categories>";
    xml
}

/// The common bounded, DTD-free parser also validates ACDSee's escaped XML.
pub(crate) fn acdsee_category_paths(content: &str) -> Result<Vec<String>, String> {
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }
    let document = parse_xml(content)?;
    let root = &document.children[0];
    if !root.uri.is_empty() || root.name != "Categories" {
        return Err("Invalid ACDSee Categories root".into());
    }
    fn visit(node: &XmlNode, ancestors: &mut Vec<String>, paths: &mut Vec<String>) {
        if !node.uri.is_empty() || node.name != "Category" {
            return;
        }
        let name = node.text.trim();
        if name.is_empty() || name.contains('|') {
            return;
        }
        ancestors.push(name.to_string());
        if node
            .attributes
            .iter()
            .any(|a| a.uri.is_empty() && a.name == "Assigned" && a.text.trim() == "1")
        {
            push_keyword(&ancestors.join("|"), paths);
        }
        for child in &node.children {
            visit(child, ancestors, paths);
        }
        ancestors.pop();
    }
    let mut paths = Vec::new();
    for node in &root.children {
        visit(node, &mut Vec::new(), &mut paths);
    }
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nul_keywords_are_rejected_without_losing_valid_values() {
        let mut values = Vec::new();
        for value in ["valid", "bad\0keyword", "Places|bad\0child", "another"] {
            push_keyword(value, &mut values);
        }
        assert_eq!(values, vec!["valid", "another"]);
    }

    #[test]
    fn lightroom_paths_are_created_only_for_nonempty_paths_and_existing_fields_clear() {
        let mut writer = TagWriter::default();
        assert!(!writer.render(&[], Some(&[])).contains("hierarchicalSubject"));
        assert!(!writer.render(&["literal|text".into()], Some(&[])).contains("hierarchicalSubject"));
        assert!(writer.render(&["Paris".into()], Some(&["Places|Paris".into()])).contains("hierarchicalSubject"));
        writer.observe(NS_LR, "hierarchicalSubject");
        let output = writer.render(&[], Some(&[]));
        assert!(output.contains("hierarchicalSubject"));
        assert_eq!(crate::t_xmp::parse_xmp_str(&packet(&output)).unwrap().hierarchical_tags, Some(vec![]));
    }

    fn packet(body: &str) -> String {
        format!(
            "<rdf:RDF xmlns:rdf=\"{NS_RDF}\"><rdf:Description>{body}</rdf:Description></rdf:RDF>"
        )
    }

    fn embedded(xml: &str) -> XmpMetadata {
        let file =
            std::env::temp_dir().join(format!("lap-xmp-compat-{}.jpg", uuid::Uuid::new_v4()));
        std::fs::write(
            &file,
            crate::t_metadata::tests::jpeg_with_xmp_packet(xml.as_bytes()),
        )
        .unwrap();
        let metadata = crate::t_metadata::read(&file).organization;
        std::fs::remove_file(file).unwrap();
        metadata
    }

    #[test]
    fn registered_schema_uses_generic_import_and_export_with_any_prefix() {
        let xml = packet("<alias:paths xmlns:alias=\"urn:lap:test-keywords\"><rdf:Bag><rdf:li>Places&gt;Paris</rdf:li></rdf:Bag></alias:paths>");
        let parsed = crate::t_xmp::parse_xmp_str(&xml).unwrap();
        let paths = vec!["Places|Paris".into()];
        assert_eq!(parsed.hierarchical_tags, Some(paths.clone()));
        assert_eq!(embedded(&xml).hierarchical_tags, Some(paths.clone()));
        assert!(writable_property("urn:lap:test-keywords", b"paths"));
        let mut writer = TagWriter::default();
        writer.observe("urn:lap:test-keywords", "paths");
        let output = writer.render(&["Paris".into()], Some(&paths));
        assert!(output.contains("Places&gt;Paris"));
        let reread = crate::t_xmp::parse_xmp_str(&packet(&output)).unwrap();
        assert_eq!(reread.hierarchical_tags, Some(paths));
    }

    #[test]
    fn embedded_keywords_use_namespace_identity_instead_of_backend_prefixes() {
        let spoofed = packet("<lr:hierarchicalSubject xmlns:lr=\"urn:unrelated\"><rdf:Bag><rdf:li>Places|Wrong</rdf:li></rdf:Bag></lr:hierarchicalSubject>");
        assert!(embedded(&spoofed).tags.is_none());
    }

    #[test]
    fn empty_missing_and_existing_category_writeback_remain_distinct() {
        assert!(crate::t_xmp::parse_xmp_str(&packet(""))
            .unwrap()
            .tags
            .is_none());
        let empty = crate::t_xmp::parse_xmp_str(&packet(
            "<a:paths xmlns:a=\"urn:lap:test-keywords\"><rdf:Bag/></a:paths>",
        ))
        .unwrap();
        assert_eq!(empty.tags, Some(vec![]));
        assert_eq!(empty.hierarchical_tags, Some(vec![]));
        assert!(writable_property(NS_ACDSEE, b"categories"));
        let mut writer = TagWriter::default();
        assert!(!writer.render(&[], Some(&[])).contains("categories"));
        writer.observe(NS_ACDSEE, "categories");
        let empty = crate::t_xmp::parse_xmp_str(&packet(&writer.render(&[], Some(&[])))).unwrap();
        assert_eq!(empty.tags, Some(vec![]));
    }

    #[test]
    fn registry_has_unique_properties_and_complete_writable_formats() {
        let mut keys = HashSet::new();
        for format in FORMATS {
            assert!(keys.insert((format.uri, format.property)));
            if format.write != WriteRule::Preserve
                && !matches!(format.encoding, Encoding::Categories)
            {
                assert!(!format.container.is_empty());
            }
        }
    }

    #[test]
    fn category_writeback_preserves_assignments_and_escapes_both_xml_layers() {
        let paths = vec![
            "Places|Paris & France".into(),
            "People".into(),
            "People|Paris".into(),
        ];
        let xml = categories_xml(&paths, true);
        let mut reread = acdsee_category_paths(&xml).unwrap();
        reread.sort();
        let mut expected = paths.clone();
        expected.sort();
        assert_eq!(reread, expected);
        assert!(xml.contains("<Category Assigned=\"0\">Places"));
        assert!(xml.contains("<Category Assigned=\"1\">People"));
        let mut writer = TagWriter::default();
        writer.observe(NS_ACDSEE, "categories");
        let output = writer.render(
            &["Paris & France".into(), "People".into(), "Paris".into()],
            Some(&paths),
        );
        assert!(output.contains("Paris &amp;amp; France"));
        let parsed = crate::t_xmp::parse_xmp_str(&packet(&output)).unwrap();
        let mut actual = parsed.hierarchical_tags.unwrap();
        actual.sort();
        assert_eq!(actual, expected);
    }

    #[test]
    fn separator_in_node_name_does_not_create_a_second_hierarchy() {
        let paths = vec!["Trips/2024|Paris".into(), "Places|Rome".into()];
        let mut writer = TagWriter::default();
        writer.observe(NS_DIGIKAM, "TagsList");
        let output = writer.render(&["Paris".into(), "Rome".into()], Some(&paths));
        assert!(output.contains("<rdf:li>Places/Rome</rdf:li>"));
        assert!(!output.contains("<rdf:li>Trips/2024/Paris</rdf:li>"));
        let parsed = crate::t_xmp::parse_xmp_str(&packet(&output)).unwrap();
        assert_eq!(parsed.hierarchical_tags, Some(paths));
    }
}
