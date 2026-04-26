# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - 2026-04-26

### Added

- **Batch Rename Feature** (`feature/batch-rename-resize`)
  - Added `BatchOperation.vue` component for batch file operations
  - Implemented `batch_rename` Tauri command for renaming multiple files with patterns
  - Added pattern variables: `{name}`, `{index}`, `{date}`, `{time}`
  - Real-time preview of rename results before execution
  - Support for sequential numbering with zero-padded indices
  - Progress indicator during batch operations

- **Batch Resize Feature** (`feature/batch-rename-resize`)
  - Implemented `batch_resize` Tauri command for batch image resizing
  - Support for pixel-based resizing with width/height inputs
  - Support for percentage-based resizing
  - Aspect ratio preservation option
  - Configurable JPEG quality (60%, 80%, 90%)
  - Multiple output formats: Original, JPEG, PNG, WebP
  - Custom output folder selection
  - Progress tracking during resize operations

- **Internationalization (i18n)**
  - Added English translations for batch operations in `en.json`
  - Added Chinese (Simplified) translations in `zh.json`
  - Translation keys:
    - `batch_rename.*`: Batch rename UI strings
    - `batch_resize.*`: Batch resize UI strings
    - `batch_operation.*`: Shared operation UI strings

### Changed

- **Backend (Rust/Tauri)**
  - Extended `t_cmds.rs` with `batch_rename` and `batch_resize` commands
  - Enhanced `api.js` with `batchRename()` and `batchResize()` frontend functions
  - Added `std::fs` import for file system operations in batch resize

### Technical Details

#### Frontend Components
- `BatchOperation.vue`: Vue 3 component with TypeScript
  - Modal dialog with tabbed operation selection
  - Real-time preview generation
  - Progress tracking and cancellation support

#### Backend Commands
- `batch_rename`: Accepts `{ file_id, source_path, new_name }`
  - Updates database with new name and pinyin transliteration
  - Returns boolean indicating success/failure

- `batch_resize`: Accepts `{ file_id, file_path, width, height, percentage, keep_aspect_ratio, quality, output_format, output_folder }`
  - Supports both pixel and percentage modes
  - Automatically calculates aspect-ratio-preserved dimensions
  - Outputs resized images with configurable quality

#### Translation Support
Available in all supported languages:
- English (en)
- Chinese Simplified (zh)
- German (de)
- Spanish (es)
- French (fr)
- Japanese (ja)
- Korean (ko)
- Portuguese (pt)
- Russian (ru)

## Related Issues

- Closes #25: [FEAT] Support Batch Rename/Resize Files

## Contributors

- Implementation by MiniMax Agent for julyx10/lap