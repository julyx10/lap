//! Metadata source adapters share field definitions in `schema`.
pub(crate) mod apple_sidecar;
pub(crate) mod embedded;
pub(crate) mod files;
pub(crate) mod keywords;
mod merge;
pub(crate) mod import_count;
mod model;
pub(crate) mod packets;
pub(crate) mod schema;
pub(crate) mod system;
pub(crate) mod xmp;
