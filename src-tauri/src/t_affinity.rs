use std::{
    fs::File,
    io::{Cursor, Read, Seek, SeekFrom},
    path::Path,
};

use image::{DynamicImage, ImageFormat, ImageReader, Limits};

const MAX_PREVIEW_BYTES: u32 = 16 * 1024 * 1024;
const MAX_PREVIEW_DIMENSION: u32 = 4096;

pub(crate) fn is_affinity_path(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            crate::t_common::AFFINITY_IMGS
                .iter()
                .any(|candidate| ext.eq_ignore_ascii_case(candidate))
        })
}

// Container versions 8–12 use the indexed Thmb record documented by Photocraft:
// https://code.danielnagel.at/github-mirror/photocraft/commit/8c7544ed2a16b33dae8e56fcbe7aa36605fdc23c
fn read_preview(path: &str) -> Option<Vec<u8>> {
    let mut file = File::open(path).ok()?;
    let mut header = [0; 32];
    file.read_exact(&mut header).ok()?;
    let version = u16::from_le_bytes([header[4], header[5]]);
    if !(8..=12).contains(&version)
        || header[6] & 3 != 0
        || &header[12..16] != b"#Inf"
        || &header[..4] != b"\x00\xffKA"
        || &header[8..12] != b"nsrP"
    {
        return None;
    }

    let offset = u64::from_le_bytes(header[24..32].try_into().ok()?);
    if offset < 72 {
        return None;
    }
    file.seek(SeekFrom::Start(offset)).ok()?;
    let mut record = [0; 29];
    file.read_exact(&mut record).ok()?;
    let field = |offset| {
        let mut bytes = [0; 4];
        bytes.copy_from_slice(&record[offset..offset + 4]);
        u32::from_le_bytes(bytes)
    };
    let length = field(24);
    if &record[..8] != b"\xff\xff\xff\xffThmb"
        || field(8) != 1
        || field(16) != 29
        || field(20) != 0
        || record[28] != 1
        || !(33..=MAX_PREVIEW_BYTES).contains(&length)
        || field(12) != length + 13
        || offset.checked_add(29 + u64::from(length))? > file.metadata().ok()?.len()
    {
        return None;
    }

    let mut png = vec![0; length as usize];
    file.read_exact(&mut png).ok()?;
    Some(png)
}

fn png_reader(bytes: &[u8]) -> ImageReader<Cursor<&[u8]>> {
    let mut reader = ImageReader::with_format(Cursor::new(bytes), ImageFormat::Png);
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_PREVIEW_DIMENSION);
    limits.max_image_height = Some(MAX_PREVIEW_DIMENSION);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    reader
}

pub(crate) fn decode_preview(path: &str) -> Option<DynamicImage> {
    png_reader(&read_preview(path)?).decode().ok()
}

pub(crate) fn preview_png(path: &str) -> Option<Vec<u8>> {
    let bytes = read_preview(path)?;
    png_reader(&bytes).decode().ok()?;
    Some(bytes)
}
