//! Read only container-designated XMP payloads; never search image or comment data.
const LIMIT: usize = 16 * 1024 * 1024;

pub(super) fn decode(bytes: &[u8]) -> Option<String> {
    if bytes.len() > LIMIT {
        return None;
    }
    let text = if bytes.starts_with(&[0xff, 0xfe, 0, 0]) || bytes.starts_with(&[0, 0, 0xfe, 0xff]) {
        let le = bytes[0] == 0xff;
        let data = &bytes[4..];
        if data.len() % 4 != 0 {
            return None;
        }
        data.chunks_exact(4)
            .map(|b| {
                let b = b.try_into().ok()?;
                char::from_u32(if le {
                    u32::from_le_bytes(b)
                } else {
                    u32::from_be_bytes(b)
                })
            })
            .collect::<Option<String>>()?
    } else if bytes.starts_with(&[0xff, 0xfe]) || bytes.starts_with(&[0xfe, 0xff]) {
        let le = bytes[0] == 0xff;
        let data = &bytes[2..];
        if data.len() % 2 != 0 {
            return None;
        }
        let words: Vec<_> = data
            .chunks_exact(2)
            .map(|b| {
                if le {
                    u16::from_le_bytes([b[0], b[1]])
                } else {
                    u16::from_be_bytes([b[0], b[1]])
                }
            })
            .collect();
        String::from_utf16(&words).ok()?
    } else {
        std::str::from_utf8(bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes))
            .ok()?
            .to_owned()
    };
    (text.len() <= LIMIT).then_some(text)
}

pub(super) fn extract(bytes: &[u8]) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    if bytes.starts_with(&[0xff, 0xd8]) {
        let mut extensions = Vec::new();
        let mut pos = 2;
        while bytes.get(pos) == Some(&0xff) {
            while bytes.get(pos) == Some(&0xff) {
                pos += 1;
            }
            let Some(&marker) = bytes.get(pos) else { break };
            pos += 1;
            if matches!(marker, 0xda | 0xd9) {
                break;
            }
            if matches!(marker, 0x01 | 0xd0..=0xd7) {
                continue;
            }
            let Some(len) = bytes
                .get(pos..pos + 2)
                .map(|b| u16::from_be_bytes([b[0], b[1]]) as usize)
            else {
                break;
            };
            if len < 2 {
                break;
            }
            let Some(data) = bytes.get(pos + 2..pos + len) else {
                break;
            };
            if marker == 0xe1 {
                if let Some(packet) = data.strip_prefix(b"http://ns.adobe.com/xap/1.0/\0") {
                    push(&mut out, packet);
                }
                if let Some(packet) = data.strip_prefix(b"http://ns.adobe.com/xmp/extension/\0") {
                    if packet.len() >= 40 && extensions.len() < 512 {
                        extensions.push(packet);
                    }
                }
            }
            pos += len;
        }
        // Only an extension referenced by the standard packet belongs to this image.
        let guid = out.iter().find_map(|packet| extended_guid(packet));
        if let Some(guid) = guid {
            let mut chunks: Vec<_> = extensions
                .into_iter()
                .filter(|p| p[..32].eq_ignore_ascii_case(guid.as_bytes()))
                .collect();
            chunks.sort_by_key(|p| u32::from_be_bytes(p[36..40].try_into().unwrap()));
            if let Some(first) = chunks.first() {
                let total = u32::from_be_bytes(first[32..36].try_into().unwrap()) as usize;
                if total <= LIMIT {
                    let mut combined = Vec::new();
                    let valid = chunks.into_iter().all(|p| {
                        let len = u32::from_be_bytes(p[32..36].try_into().unwrap()) as usize;
                        let offset = u32::from_be_bytes(p[36..40].try_into().unwrap()) as usize;
                        if len != total
                            || offset != combined.len()
                            || p.len() - 40 > total.saturating_sub(offset)
                        {
                            return false;
                        }
                        combined.extend_from_slice(&p[40..]);
                        true
                    });
                    if valid && combined.len() == total {
                        push(&mut out, &combined);
                    }
                }
            }
        }
    } else if bytes.starts_with(b"II") || bytes.starts_with(b"MM") {
        let le = bytes.starts_with(b"II");
        let u16at = |p: usize| -> Option<u16> {
            let b: [u8; 2] = bytes.get(p..p.checked_add(2)?)?.try_into().ok()?;
            Some(if le {
                u16::from_le_bytes(b)
            } else {
                u16::from_be_bytes(b)
            })
        };
        let u32at = |p: usize| -> Option<u32> {
            let b: [u8; 4] = bytes.get(p..p.checked_add(4)?)?.try_into().ok()?;
            Some(if le {
                u32::from_le_bytes(b)
            } else {
                u32::from_be_bytes(b)
            })
        };
        if matches!(u16at(2), Some(42 | 0x4f52 | 0x5352)) {
            if let Some(offset) = u32at(4).map(|v| v as usize) {
                for i in 0..u16at(offset).unwrap_or(0) as usize {
                    let p = offset + 2 + i * 12;
                    if u16at(p) != Some(700) {
                        continue;
                    }
                    if !matches!(u16at(p + 2), Some(1 | 7)) {
                        continue;
                    }
                    if let Some(len) = u32at(p + 4).map(|v| v as usize).filter(|v| *v <= LIMIT) {
                        let start = if len <= 4 {
                            Some(p + 8)
                        } else {
                            u32at(p + 8).map(|v| v as usize)
                        };
                        if let Some(data) =
                            start.and_then(|start| bytes.get(start..start.checked_add(len)?))
                        {
                            push(&mut out, data);
                        }
                    }
                }
            }
        }
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        let end = bytes
            .get(4..8)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()) as usize + 8)
            .unwrap_or(0);
        if end <= bytes.len() {
            let mut pos = 12;
            while pos + 8 <= end {
                let len = u32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap()) as usize;
                let Some(next) = (pos + 8).checked_add(len) else {
                    break;
                };
                if next > end {
                    break;
                }
                if &bytes[pos..pos + 4] == b"XMP " {
                    push(&mut out, &bytes[pos + 8..next]);
                }
                pos = next + (len & 1);
            }
        }
    } else if matches!(
        bytes.get(4..8),
        Some(b"ftyp" | b"moov" | b"mdat" | b"wide" | b"free" | b"skip" | b"jP  ")
    ) {
        boxes(bytes, 0, &mut out);
    }
    out
}

fn push(out: &mut Vec<Vec<u8>>, packet: &[u8]) {
    if packet.len() <= LIMIT && out.len() < 32 {
        out.push(packet.to_vec());
    }
}

fn boxes(bytes: &[u8], depth: usize, out: &mut Vec<Vec<u8>>) {
    if depth > 8 {
        return;
    }
    let mut pos = 0;
    while pos + 8 <= bytes.len() {
        let size = u32::from_be_bytes(bytes[pos..pos + 4].try_into().unwrap());
        let (len, header) = if size == 1 {
            let Some(b) = bytes.get(pos + 8..pos + 16) else {
                break;
            };
            let Ok(len) = usize::try_from(u64::from_be_bytes(b.try_into().unwrap())) else {
                break;
            };
            (len, 16)
        } else if size == 0 {
            (bytes.len() - pos, 8)
        } else {
            (size as usize, 8)
        };
        if len < header {
            break;
        }
        let Some(end) = pos.checked_add(len).filter(|end| *end <= bytes.len()) else {
            break;
        };
        let data = &bytes[pos + header..end];
        match &bytes[pos + 4..pos + 8] {
            b"uuid"
                if data.starts_with(&[
                    0xbe, 0x7a, 0xcf, 0xcb, 0x97, 0xa9, 0x42, 0xe8, 0x9c, 0x71, 0x99, 0x94, 0x91,
                    0xe3, 0xaf, 0xac,
                ]) =>
            {
                push(out, &data[16..])
            }
            b"XMP_" | b"xml " => push(out, data),
            b"moov" | b"udta" => boxes(data, depth + 1, out),
            _ => {}
        }
        pos = end;
    }
}

fn extended_guid(packet: &[u8]) -> Option<String> {
    use quick_xml::{events::Event, name::ResolveResult, reader::NsReader};
    const NS: &[u8] = b"http://ns.adobe.com/xmp/note/";
    let xml = decode(packet)?;
    let mut reader = NsReader::from_str(&xml);
    loop {
        let event = reader.read_event().ok()?;
        let is_start = matches!(event, Event::Start(_));
        match event {
            Event::Start(e) | Event::Empty(e) => {
                for attr in e.attributes() {
                    let attr = attr.ok()?;
                    let (ns, name) = reader.resolve_attribute(attr.key);
                    if matches!(ns, ResolveResult::Bound(uri) if uri.as_ref() == NS)
                        && name.as_ref() == b"HasExtendedXMP"
                    {
                        let value = attr
                            .decode_and_unescape_value(reader.decoder())
                            .ok()?
                            .into_owned();
                        if value.len() == 32 && value.bytes().all(|b| b.is_ascii_hexdigit()) {
                            return Some(value);
                        }
                    }
                }
                let (ns, name) = reader.resolve_element(e.name());
                if is_start
                    && matches!(ns, ResolveResult::Bound(uri) if uri.as_ref() == NS)
                    && name.as_ref() == b"HasExtendedXMP"
                {
                    let value = reader.read_text(e.name()).ok()?.trim().to_owned();
                    if value.len() == 32 && value.bytes().all(|b| b.is_ascii_hexdigit()) {
                        return Some(value);
                    }
                }
            }
            Event::Eof => return None,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn jpeg_segment(marker: u8, data: &[u8]) -> Vec<u8> {
        let mut out = vec![0xff, marker];
        out.extend(((data.len() + 2) as u16).to_be_bytes());
        out.extend(data);
        out
    }
    #[test]
    fn extended_packets_require_reference_and_complete_nonoverlapping_chunks() {
        let guid = "0123456789ABCDEF0123456789ABCDEF";
        let standard = format!(
            "<rdf:RDF xmlns:rdf=\"{}\"><rdf:Description xmlns:n=\"http://ns.adobe.com/xmp/note/\" n:HasExtendedXMP=\"{guid}\"/></rdf:RDF>",
            crate::t_xmp::NS_RDF
        );
        let packet = b"<extended/>";
        let mut main = b"http://ns.adobe.com/xap/1.0/\0".to_vec();
        main.extend(standard.as_bytes());
        let extension = |offset: u32, data: &[u8]| {
            let mut out = b"http://ns.adobe.com/xmp/extension/\0".to_vec();
            out.extend(guid.as_bytes());
            out.extend((packet.len() as u32).to_be_bytes());
            out.extend(offset.to_be_bytes());
            out.extend(data);
            jpeg_segment(0xe1, &out)
        };
        let mut jpeg = vec![0xff, 0xd8];
        jpeg.extend(jpeg_segment(0xe1, &main));
        jpeg.extend(extension(5, &packet[5..]));
        jpeg.extend(extension(0, &packet[..5]));
        assert_eq!(
            extract(&jpeg),
            vec![standard.as_bytes().to_vec(), packet.to_vec()]
        );
        jpeg.extend(extension(0, &packet[..5]));
        assert_eq!(extract(&jpeg).len(), 1); // overlapping duplicate invalidates extension
        let mut orphan = vec![0xff, 0xd8];
        orphan.extend(extension(0, packet));
        assert!(extract(&orphan).is_empty());
    }
    #[test]
    fn webp_uses_only_xmp_chunk_within_declared_riff_boundary() {
        let chunk = |kind: &[u8], data: &[u8]| {
            let mut v = kind.to_vec();
            v.extend((data.len() as u32).to_le_bytes());
            v.extend(data);
            if data.len() % 2 != 0 {
                v.push(0);
            }
            v
        };
        let mut body = b"WEBP".to_vec();
        body.extend(chunk(b"JUNK", b"<fake/>"));
        body.extend(chunk(b"XMP ", b"<real/>"));
        let mut bytes = b"RIFF".to_vec();
        bytes.extend((body.len() as u32).to_le_bytes());
        bytes.extend(body);
        bytes.extend(chunk(b"XMP ", b"<trailer/>"));
        assert_eq!(extract(&bytes), vec![b"<real/>".to_vec()]);
    }
    #[test]
    fn tiff_ignores_thumbnail_ifd_and_arbitrary_xml() {
        let mut bytes = b"II\x2a\0\x08\0\0\0".to_vec();
        bytes.extend(1u16.to_le_bytes());
        bytes.extend(700u16.to_le_bytes());
        bytes.extend(1u16.to_le_bytes());
        bytes.extend(7u32.to_le_bytes());
        bytes.extend(26u32.to_le_bytes());
        bytes.extend(33u32.to_le_bytes());
        bytes.extend(b"<real/>");
        bytes.extend(1u16.to_le_bytes());
        bytes.extend(700u16.to_le_bytes());
        bytes.extend(1u16.to_le_bytes());
        bytes.extend(7u32.to_le_bytes());
        bytes.extend(51u32.to_le_bytes());
        bytes.extend(0u32.to_le_bytes());
        bytes.extend(b"<fake/>");
        assert_eq!(extract(&bytes), vec![b"<real/>".to_vec()]);
    }
    #[test]
    fn decoding_accepts_unicode_and_rejects_truncated_or_invalid_codepoints() {
        let text = "<caption>照片 📷</caption>";
        for le in [true, false] {
            let mut utf16 = if le {
                vec![0xff, 0xfe]
            } else {
                vec![0xfe, 0xff]
            };
            for word in text.encode_utf16() {
                utf16.extend(if le {
                    word.to_le_bytes()
                } else {
                    word.to_be_bytes()
                });
            }
            assert_eq!(decode(&utf16).as_deref(), Some(text));
            utf16.pop();
            assert!(decode(&utf16).is_none());
            let mut utf32 = if le {
                vec![0xff, 0xfe, 0, 0]
            } else {
                vec![0, 0, 0xfe, 0xff]
            };
            for c in text.chars() {
                utf32.extend(if le {
                    (c as u32).to_le_bytes()
                } else {
                    (c as u32).to_be_bytes()
                });
            }
            assert_eq!(decode(&utf32).as_deref(), Some(text));
        }
        assert!(decode(&[0xff, 0xfe, 0, 0, 0, 0xd8, 0, 0]).is_none());
    }
    #[test]
    fn malformed_container_lengths_do_not_panic() {
        let seeds = [
            b"\xff\xd8\xff\xe1\xff\xff".as_slice(),
            b"II\x2a\0\xff\xff\xff\xff",
            b"RIFF\xff\xff\xff\xffWEBP",
            b"\0\0\0\x01ftyp\xff\xff\xff\xff\xff\xff\xff\xff",
        ];
        for seed in seeds {
            for end in 0..=seed.len() {
                assert!(extract(&seed[..end]).is_empty());
            }
        }
    }
}

#[cfg(test)]
mod guid_regressions {
    use super::*;
    #[test]
    fn empty_guid_element_does_not_consume_following_property() {
        let guid = "0123456789ABCDEF0123456789ABCDEF";
        let xml = format!(
            "<rdf:RDF xmlns:rdf=\"{}\"><rdf:Description xmlns:n=\"http://ns.adobe.com/xmp/note/\"><n:HasExtendedXMP/><n:HasExtendedXMP>{guid}</n:HasExtendedXMP></rdf:Description></rdf:RDF>",
            crate::t_xmp::NS_RDF
        );
        assert_eq!(extended_guid(xml.as_bytes()).as_deref(), Some(guid));
    }
}
