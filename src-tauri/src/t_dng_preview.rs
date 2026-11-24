//! Extract the RGB JPEG XL preview embedded in a DNG 1.7 file.
//!
//! DNG 1.7 can store both the raw mosaic and preview IFDs with JPEG XL
//! compression. LibRaw without the Adobe DNG SDK cannot decode the raw JXL
//! tiles, but the embedded RGB preview is a self-contained JPEG XL stream.
//! Decoding that stream with `jxl-oxide` keeps DNG previews available on all
//! platforms without pulling in the Adobe SDK.

use std::collections::HashSet;
use std::fs::File;
use std::path::Path;

use memmap2::Mmap;

const TAG_IMAGE_WIDTH: u16 = 256;
const TAG_IMAGE_LENGTH: u16 = 257;
const TAG_COMPRESSION: u16 = 259;
const TAG_PHOTOMETRIC: u16 = 262;
const TAG_STRIP_OFFSETS: u16 = 273;
const TAG_STRIP_BYTE_COUNTS: u16 = 279;
const TAG_TILE_OFFSETS: u16 = 324;
const TAG_TILE_BYTE_COUNTS: u16 = 325;
const TAG_SUB_IFD: u16 = 330;

const COMPRESSION_JXL: u32 = 52546;
const PHOTOMETRIC_RGB: u32 = 2;
const PHOTOMETRIC_YCBCR: u32 = 6;

#[derive(Clone, Copy)]
struct ByteOrder {
    little: bool,
}

impl ByteOrder {
    fn u16(self, bytes: &[u8], offset: usize) -> Option<u16> {
        let value = bytes.get(offset..offset + 2)?;
        Some(if self.little {
            u16::from_le_bytes([value[0], value[1]])
        } else {
            u16::from_be_bytes([value[0], value[1]])
        })
    }

    fn u32(self, bytes: &[u8], offset: usize) -> Option<u32> {
        let value = bytes.get(offset..offset + 4)?;
        Some(if self.little {
            u32::from_le_bytes([value[0], value[1], value[2], value[3]])
        } else {
            u32::from_be_bytes([value[0], value[1], value[2], value[3]])
        })
    }
}

#[derive(Clone, Copy)]
struct Entry {
    tag: u16,
    typ: u16,
    count: u32,
    value_offset: u32,
    inline_offset: usize,
}

impl Entry {
    fn type_size(self) -> Option<usize> {
        match self.typ {
            1 | 2 | 6 | 7 => Some(1),
            3 | 8 => Some(2),
            4 | 9 | 11 | 13 => Some(4),
            5 | 10 | 12 => Some(8),
            _ => None,
        }
    }

    fn total_size(self) -> Option<usize> {
        self.type_size()?.checked_mul(self.count as usize)
    }

    fn values(self, bytes: &[u8], order: ByteOrder) -> Option<Vec<u32>> {
        let size = self.type_size()?;
        let total = self.total_size()?;
        let start = if total <= 4 {
            // The value is stored inline in the 4-byte value field.
            self.inline_offset
        } else {
            self.value_offset as usize
        };
        let data = bytes.get(start..start.checked_add(total)?)?;
        let mut values = Vec::with_capacity(self.count as usize);
        for index in 0..self.count as usize {
            let offset = index.checked_mul(size)?;
            let value = match size {
                1 => data.get(offset).copied()? as u32,
                2 => order.u16(data, offset)? as u32,
                4 => order.u32(data, offset)?,
                _ => continue,
            };
            values.push(value);
        }
        Some(values)
    }
}

struct Ifd {
    entries: Vec<Entry>,
}

impl Ifd {
    fn entry(&self, tag: u16) -> Option<Entry> {
        self.entries.iter().copied().find(|entry| entry.tag == tag)
    }
}

fn parse_ifd(bytes: &[u8], offset: usize, order: ByteOrder) -> Option<Ifd> {
    let count = order.u16(bytes, offset)? as usize;
    let entries_start = offset.checked_add(2)?;
    let entries_end = entries_start.checked_add(count.checked_mul(12)?)?;
    if entries_end > bytes.len() {
        return None;
    }
    let mut entries = Vec::with_capacity(count);
    for index in 0..count {
        let entry_offset = entries_start.checked_add(index.checked_mul(12)?)?;
        let entry = Entry {
            tag: order.u16(bytes, entry_offset)?,
            typ: order.u16(bytes, entry_offset + 2)?,
            count: order.u32(bytes, entry_offset + 4)?,
            value_offset: order.u32(bytes, entry_offset + 8)?,
            inline_offset: entry_offset + 8,
        };
        entries.push(entry);
    }
    Some(Ifd { entries })
}

fn read_strips(bytes: &[u8], ifd: &Ifd, order: ByteOrder) -> Option<Vec<u8>> {
    let (offsets, counts) = if let (Some(offsets), Some(counts)) = (
        ifd.entry(TAG_STRIP_OFFSETS).and_then(|entry| entry.values(bytes, order)),
        ifd.entry(TAG_STRIP_BYTE_COUNTS).and_then(|entry| entry.values(bytes, order)),
    ) {
        (offsets, counts)
    } else {
        (
            ifd.entry(TAG_TILE_OFFSETS).and_then(|entry| entry.values(bytes, order))?,
            ifd.entry(TAG_TILE_BYTE_COUNTS).and_then(|entry| entry.values(bytes, order))?,
        )
    };
    if offsets.is_empty() || offsets.len() != counts.len() {
        return None;
    }
    let mut result = Vec::new();
    for (offset, count) in offsets.into_iter().zip(counts) {
        let start = offset as usize;
        let length = count as usize;
        let end = start.checked_add(length)?;
        result.extend_from_slice(bytes.get(start..end)?);
    }
    Some(result)
}

fn is_rgb_preview(ifd: &Ifd, bytes: &[u8], order: ByteOrder) -> bool {
    let compression = ifd.entry(TAG_COMPRESSION)
        .and_then(|entry| entry.values(bytes, order))
        .and_then(|values| values.first().copied());
    if compression != Some(COMPRESSION_JXL) {
        return false;
    }
    let photometric = ifd.entry(TAG_PHOTOMETRIC)
        .and_then(|entry| entry.values(bytes, order))
        .and_then(|values| values.first().copied());
    matches!(photometric, Some(PHOTOMETRIC_RGB | PHOTOMETRIC_YCBCR))
}

/// Return the largest embedded RGB JPEG XL preview from a DNG file.
///
/// The returned bytes are a complete JPEG XL codestream. Callers should decode
/// them with `jxl-oxide` and cache the result as a normal embedded RAW preview.
pub fn get_embedded_jxl_preview(path: &str) -> Option<Vec<u8>> {
    if !Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("dng"))
    {
        return None;
    }
    let file = File::open(path).ok()?;
    let bytes = unsafe { Mmap::map(&file).ok()? };
    if bytes.len() < 8 {
        return None;
    }
    let order = if bytes.starts_with(b"II") {
        ByteOrder { little: true }
    } else if bytes.starts_with(b"MM") {
        ByteOrder { little: false }
    } else {
        return None;
    };
    let first_ifd = order.u32(&bytes, 4)? as usize;
    let mut queue = vec![first_ifd];
    let mut seen = HashSet::new();
    let mut best: Option<(u64, Vec<u8>)> = None;
    while let Some(offset) = queue.pop() {
        if offset == 0 || !seen.insert(offset) {
            continue;
        }
        let Some(ifd) = parse_ifd(&bytes, offset, order) else {
            continue;
        };
        if let Some(sub_ifds) = ifd.entry(TAG_SUB_IFD)
            .and_then(|entry| entry.values(&bytes, order))
        {
            queue.extend(sub_ifds.into_iter().map(|value| value as usize));
        }
        if !is_rgb_preview(&ifd, &bytes, order) {
            continue;
        }
        let Some(width) = ifd.entry(TAG_IMAGE_WIDTH)
            .and_then(|entry| entry.values(&bytes, order))
            .and_then(|values| values.first().copied())
        else { continue };
        let Some(height) = ifd.entry(TAG_IMAGE_LENGTH)
            .and_then(|entry| entry.values(&bytes, order))
            .and_then(|values| values.first().copied())
        else { continue };
        let Some(strips) = read_strips(&bytes, &ifd, order) else {
            continue;
        };
        if strips.is_empty() {
            continue;
        }
        let area = u64::from(width) * u64::from(height);
        if best.as_ref().is_none_or(|(best_area, _)| area > *best_area) {
            best = Some((area, strips));
        }
    }
    best.map(|(_, data)| data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_dng_input() {
        assert!(get_embedded_jxl_preview("not-a-dng.jpg").is_none());
    }
}
