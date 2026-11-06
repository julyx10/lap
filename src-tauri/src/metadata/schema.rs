//! Field registry. Source names, encodings, priorities and write capabilities live here.
//! Parsers and catalog merging consume these definitions without duplicating aliases.

/// Bump when changed mappings must backfill unchanged catalog entries.
pub(crate) const IMPORT_PREFIX: &str = "embedded-v7:";

pub const NS_LR: &str = "http://ns.adobe.com/lightroom/1.0/";
pub const NS_DIGIKAM: &str = "http://www.digikam.org/ns/1.0/";
pub const NS_ACDSEE: &str = "http://ns.acdsee.com/iptc/1.0/";

#[derive(Clone, Copy)]
pub(super) enum Encoding {
    Flat,
    Path(char),
    Categories,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum WriteRule {
    Always,
    IfPresentOrPaths,
    IfPresent,
    Preserve,
}

pub(super) struct Format {
    pub(super) uri: &'static str,
    pub(super) prefix: &'static str,
    pub(super) property: &'static str,
    pub(super) backend_name: &'static str,
    pub(super) encoding: Encoding,
    pub(super) container: &'static str,
    pub(super) write: WriteRule,
}

pub(super) const FORMATS: &[Format] = &[
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

#[derive(Clone, Copy)]
pub(super) enum SystemEncoding {
    WindowsUtf16,
    FinderTags,
    PlistString,
    Utf8,
    CommaSeparated,
    RatingTen,
}

pub(super) struct SystemMapping {
    pub(super) source: &'static str,
    pub(super) field: CatalogField,
    pub(super) encoding: SystemEncoding,
}

pub(super) const WINDOWS_FIELDS: &[SystemMapping] = &[
    SystemMapping {
        source: "XPKeywords",
        field: CatalogField::Tags,
        encoding: SystemEncoding::WindowsUtf16,
    },
    SystemMapping {
        source: "XPComment",
        field: CatalogField::Description,
        encoding: SystemEncoding::WindowsUtf16,
    },
];

// Kept in one table so source names, decoders and destinations stay together.
pub(super) const MACOS_FIELDS: &[SystemMapping] = &[
    SystemMapping {
        source: "com.apple.metadata:_kMDItemUserTags",
        field: CatalogField::Tags,
        encoding: SystemEncoding::FinderTags,
    },
    SystemMapping {
        source: "com.apple.metadata:kMDItemFinderComment",
        field: CatalogField::Description,
        encoding: SystemEncoding::PlistString,
    },
];
pub(super) const LINUX_FIELDS: &[SystemMapping] = &[
    SystemMapping {
        source: "user.xdg.tags",
        field: CatalogField::Tags,
        encoding: SystemEncoding::CommaSeparated,
    },
    SystemMapping {
        source: "user.xdg.comment",
        field: CatalogField::Description,
        encoding: SystemEncoding::Utf8,
    },
    SystemMapping {
        source: "user.baloo.rating",
        field: CatalogField::Rating,
        encoding: SystemEncoding::RatingTen,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DetailSection {
    Source,
    Capture,
    Location,
}

impl DetailSection {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "source" => Some(Self::Source),
            "capture" => Some(Self::Capture),
            "location" => Some(Self::Location),
            _ => None,
        }
    }
}

/// Detail keys are the existing frontend contract. Original descriptions remain
/// separate from editable catalog descriptions.
pub(super) struct DetailField {
    pub key: &'static str,
    pub backend_names: &'static [&'static str],
    pub xmp: Option<XmpProperty>,
    pub stored: bool,
    pub descriptive: bool,
    pub section: Option<DetailSection>,
}

pub(super) const DETAIL_FIELDS: &[DetailField] = &[
    DetailField {
        key: "title",
        backend_names: &["Title", "ObjectName"],
        xmp: Some(XmpProperty {
            uri: NS_DC,
            prefix: "dc",
            name: "title",
        }),
        stored: true,
        descriptive: true,
        section: Some(DetailSection::Source),
    },
    DetailField {
        key: "headline",
        backend_names: &["Headline"],
        xmp: Some(XmpProperty { uri: NS_PHOTOSHOP, prefix: "photoshop", name: "Headline" }),
        stored: true,
        descriptive: true,
        section: Some(DetailSection::Source),
    },
    DetailField {
        key: "creator",
        backend_names: &["Creator", "By-line", "Artist"],
        xmp: Some(XmpProperty {
            uri: NS_DC,
            prefix: "dc",
            name: "creator",
        }),
        stored: true,
        descriptive: true,
        section: Some(DetailSection::Source),
    },
    DetailField {
        key: "copyright",
        backend_names: &["Rights", "CopyrightNotice", "Copyright"],
        xmp: Some(XmpProperty {
            uri: NS_DC,
            prefix: "dc",
            name: "rights",
        }),
        stored: true,
        descriptive: true,
        section: Some(DetailSection::Source),
    },
    DetailField {
        key: "description",
        backend_names: &["Description", "Caption-Abstract", "ImageDescription"],
        xmp: Some(XmpProperty {
            uri: NS_DC,
            prefix: "dc",
            name: "description",
        }),
        stored: true,
        descriptive: true,
        section: Some(DetailSection::Source),
    },
    DetailField {
        key: "credit",
        backend_names: &["Credit"],
        xmp: Some(XmpProperty {
            uri: NS_PHOTOSHOP,
            prefix: "photoshop",
            name: "Credit",
        }),
        stored: false,
        descriptive: true,
        section: Some(DetailSection::Source),
    },
    DetailField {
        key: "source",
        backend_names: &["Source"],
        xmp: Some(XmpProperty {
            uri: NS_PHOTOSHOP,
            prefix: "photoshop",
            name: "Source",
        }),
        stored: false,
        descriptive: true,
        section: Some(DetailSection::Source),
    },
    DetailField {
        key: "usageTerms",
        backend_names: &["UsageTerms"],
        xmp: Some(XmpProperty {
            uri: "http://ns.adobe.com/xap/1.0/rights/",
            prefix: "xmpRights",
            name: "UsageTerms",
        }),
        stored: false,
        descriptive: true,
        section: Some(DetailSection::Source),
    },
    DetailField {
        key: "rightsUrl",
        backend_names: &["WebStatement"],
        xmp: Some(XmpProperty {
            uri: "http://ns.adobe.com/xap/1.0/rights/",
            prefix: "xmpRights",
            name: "WebStatement",
        }),
        stored: false,
        descriptive: true,
        section: Some(DetailSection::Source),
    },
    DetailField {
        key: "software",
        backend_names: &["CreatorTool", "Software"],
        xmp: Some(XmpProperty {
            uri: NS_XMP,
            prefix: "xmp",
            name: "CreatorTool",
        }),
        stored: true,
        descriptive: true,
        section: Some(DetailSection::Source),
    },
    DetailField {
        key: "flash",
        backend_names: &["Flash"],
        xmp: None,
        stored: false,
        descriptive: false,
        section: Some(DetailSection::Capture),
    },
    DetailField {
        key: "whiteBalance",
        backend_names: &["WhiteBalance"],
        xmp: None,
        stored: false,
        descriptive: false,
        section: Some(DetailSection::Capture),
    },
    DetailField {
        key: "exposureMode",
        backend_names: &["ExposureMode"],
        xmp: None,
        stored: false,
        descriptive: false,
        section: Some(DetailSection::Capture),
    },
    DetailField {
        key: "exposureProgram",
        backend_names: &["ExposureProgram"],
        xmp: None,
        stored: false,
        descriptive: false,
        section: Some(DetailSection::Capture),
    },
    DetailField {
        key: "meteringMode",
        backend_names: &["MeteringMode"],
        xmp: None,
        stored: false,
        descriptive: false,
        section: Some(DetailSection::Capture),
    },
    DetailField {
        key: "focalLength35mm",
        backend_names: &["FocalLengthIn35mmFormat"],
        xmp: None,
        stored: false,
        descriptive: false,
        section: Some(DetailSection::Capture),
    },
    DetailField {
        key: "timeZone",
        backend_names: &["OffsetTimeOriginal"],
        xmp: None,
        stored: false,
        descriptive: false,
        section: Some(DetailSection::Capture),
    },
    DetailField {
        key: "direction",
        backend_names: &["GPSImgDirection"],
        xmp: None,
        stored: false,
        descriptive: false,
        section: Some(DetailSection::Location),
    },
    DetailField {
        key: "directionRef",
        backend_names: &["GPSImgDirectionRef"],
        xmp: None,
        stored: false,
        descriptive: false,
        section: Some(DetailSection::Location),
    },
    DetailField {
        key: "iccProfile",
        backend_names: &["ProfileDescription"],
        xmp: None,
        stored: true,
        descriptive: false,
        section: None,
    },
    DetailField {
        key: "colorSpace",
        backend_names: &["ColorSpace"],
        xmp: None,
        stored: true,
        descriptive: false,
        section: None,
    },
];

// Canonical XMP Namespace URIs
/// Lap-owned schema. URI identity is stable and need not resolve to a web page.
pub const NS_LAP: &str = "https://github.com/julyx10/lap/ns/1.0/";
pub const NS_XMP: &str = "http://ns.adobe.com/xap/1.0/";
pub const NS_XMP_DM: &str = "http://ns.adobe.com/xmp/1.0/DynamicMedia/";
pub const NS_DARKTABLE: &str = "http://darktable.sf.net/";
pub const NS_DC: &str = "http://purl.org/dc/elements/1.1/";
pub const NS_EXIF: &str = "http://ns.adobe.com/exif/1.0/";
pub const NS_PHOTOSHOP: &str = "http://ns.adobe.com/photoshop/1.0/";
pub const NS_RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";

/// Catalog fields have stable dirty bits shared by imports, edit tracking and export.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CatalogField {
    Rating,
    Culling,
    Description,
    Tags,
    Favorite,
}

impl CatalogField {
    pub const fn mask(self) -> i32 {
        match self {
            Self::Rating => 1,
            Self::Culling => 2,
            Self::Description => 4,
            Self::Tags => 8,
            Self::Favorite => 16,
        }
    }

    pub const fn column(self) -> Option<&'static str> {
        match self {
            Self::Rating => Some("rating"),
            Self::Culling => Some("culling_flag"),
            Self::Description => Some("comments"),
            Self::Favorite => Some("is_favorite"),
            Self::Tags => None,
        }
    }
}

pub(super) const CATALOG_FIELDS: &[CatalogField] = &[
    CatalogField::Rating,
    CatalogField::Culling,
    CatalogField::Description,
    CatalogField::Tags,
    CatalogField::Favorite,
];

#[derive(Clone, Copy)]
pub(super) struct XmpProperty {
    pub uri: &'static str,
    pub prefix: &'static str,
    pub name: &'static str,
}

impl XmpProperty {
    pub fn matches(self, uri: &str, name: &str) -> bool {
        self.uri == uri && self.name == name
    }

    pub fn qualified_name(self) -> String {
        format!("{}:{}", self.prefix, self.name)
    }

    pub fn text_xml(self, escaped_value: &str) -> String {
        let name = self.qualified_name();
        format!("<{name}>{escaped_value}</{name}>")
    }
}

pub(super) const FAVORITE: XmpProperty = XmpProperty {
    uri: NS_LAP,
    prefix: "lap",
    name: "Favorite",
};
pub(super) const RATING: XmpProperty = XmpProperty {
    uri: NS_XMP,
    prefix: "xmp",
    name: "Rating",
};
pub(super) const PICK: XmpProperty = XmpProperty {
    uri: NS_XMP_DM,
    prefix: "xmpDM",
    name: "pick",
};
pub(super) const GOOD: XmpProperty = XmpProperty {
    uri: NS_XMP_DM,
    prefix: "xmpDM",
    name: "good",
};
pub(super) const REJECTED: XmpProperty = XmpProperty {
    uri: NS_DARKTABLE,
    prefix: "darktable",
    name: "rejected",
};
pub(super) const DIGIKAM_PICK: XmpProperty = XmpProperty {
    uri: NS_DIGIKAM,
    prefix: "digiKam",
    name: "PickLabel",
};
pub(super) const LABEL: XmpProperty = XmpProperty {
    uri: NS_XMP,
    prefix: "xmp",
    name: "Label",
};
// First present value wins, including explicit clears.
pub(super) const DESCRIPTION: &[XmpProperty] = &[
    XmpProperty {
        uri: NS_DC,
        prefix: "dc",
        name: "description",
    },
    XmpProperty {
        uri: NS_EXIF,
        prefix: "exif",
        name: "UserComment",
    },
    XmpProperty {
        uri: NS_PHOTOSHOP,
        prefix: "photoshop",
        name: "Caption",
    },
];

pub(super) const EMBEDDED_DESCRIPTION: &[&str] =
    &["Description", "Caption-Abstract", "UserComment"];
pub(super) const SOURCE_PRIORITY: &[&str] = &["XMP", "IPTC", "EXIF"];
pub(super) const IPTC_KEYWORDS: (&str, &str) = ("IPTC", "Keywords");

/// Native EXIF selectors and backend aliases share the same identity.
#[derive(Clone, Copy)]
pub(crate) enum CaptureField {
    Make,
    Model,
    DateTimeOriginal,
    Software,
    Artist,
    Copyright,
    ImageDescription,
    LensMake,
    LensModel,
    ExposureBiasValue,
    ExposureTime,
    FNumber,
    FocalLength,
    PhotographicSensitivity,
    Flash,
    Orientation,
    GPSLatitude,
    GPSLongitude,
    GPSLatitudeRef,
    GPSLongitudeRef,
    GPSAltitude,
}

impl CaptureField {
    fn definition(self) -> (exif::Tag, &'static str) {
        match self {
            Self::Make => (exif::Tag::Make, "Make"),
            Self::Model => (exif::Tag::Model, "Model"),
            Self::DateTimeOriginal => (exif::Tag::DateTimeOriginal, "DateTimeOriginal"),
            Self::Software => (exif::Tag::Software, "Software"),
            Self::Artist => (exif::Tag::Artist, "Artist"),
            Self::Copyright => (exif::Tag::Copyright, "Copyright"),
            Self::ImageDescription => (exif::Tag::ImageDescription, "ImageDescription"),
            Self::LensMake => (exif::Tag::LensMake, "LensMake"),
            Self::LensModel => (exif::Tag::LensModel, "LensModel"),
            Self::ExposureBiasValue => (exif::Tag::ExposureBiasValue, "ExposureBiasValue"),
            Self::ExposureTime => (exif::Tag::ExposureTime, "ExposureTime"),
            Self::FNumber => (exif::Tag::FNumber, "FNumber"),
            Self::FocalLength => (exif::Tag::FocalLength, "FocalLength"),
            Self::PhotographicSensitivity => (
                exif::Tag::PhotographicSensitivity,
                "PhotographicSensitivity",
            ),
            Self::Flash => (exif::Tag::Flash, "Flash"),
            Self::Orientation => (exif::Tag::Orientation, "Orientation"),
            Self::GPSLatitude => (exif::Tag::GPSLatitude, "GPSLatitude"),
            Self::GPSLongitude => (exif::Tag::GPSLongitude, "GPSLongitude"),
            Self::GPSLatitudeRef => (exif::Tag::GPSLatitudeRef, "GPSLatitudeRef"),
            Self::GPSLongitudeRef => (exif::Tag::GPSLongitudeRef, "GPSLongitudeRef"),
            Self::GPSAltitude => (exif::Tag::GPSAltitude, "GPSAltitude"),
        }
    }

    pub fn exif(self) -> exif::Tag {
        self.definition().0
    }
    pub fn backend_name(self) -> &'static str {
        self.definition().1
    }
}

pub(super) const TECHNICAL_GROUPS: &[&str] =
    &["EXIF", "XMP", "ICC_Profile", "PNG", "File", "QuickTime"];
pub(super) const BIT_DEPTH_NAMES: &[&str] = &["BitsPerSample", "BitDepth"];
pub(super) const BIT_DEPTH_GROUPS: &[&str] = &["EXIF", "PNG", "File"];
pub(super) const LENS_ID: (&str, &str) = ("Composite", "LensID");

/// Apple image marker aliases, in discovery order.
pub(crate) const APPLE_CONTENT_IDENTIFIER_MARKERS: &[&str] = &[
    "com.apple.quicktime.content.identifier",
    "ContentIdentifier",
];

/// ffprobe format and stream tag aliases, in discovery order.
pub(crate) const APPLE_VIDEO_CONTENT_IDENTIFIER_KEYS: &[&str] = &[
    "com.apple.quicktime.content.identifier",
    "content.identifier",
    "live-photo-info",
];
