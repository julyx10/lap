/**
 * Common constants and shared types
 */

// File Extensions
pub const NORMAL_IMGS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "bmp", "tif", "tiff", "webp", "avif", "heic", "heif",
];
pub const RAW_IMGS: &[&str] = &[
    "cr2", "cr3", "nef", "nrw", "arw", "srf", "sr2", "raf", "rw2", "orf", "pef", "dng",
];
pub const VIDEOS: &[&str] = &[
    "mpg", "mpeg", "mp4", "mkv", "avi", "mov", "webm", "flv", "wmv", "3gp", "m4v", "hevc", "asf",
    "mts", "m2ts", "mod", "tod", "ts",
];
// pub const AUDIOS: &[&str] = &[
//     "mp3", "wav", "flac", "aac", "m4a", "ogg", "wma", "mp2", "mp1", "ape", "alac", "wavpack",
// ];

// AI search
pub const AI_TEXT_MODEL: &str = "text_model.onnx";
pub const AI_VISION_MODEL: &str = "vision_model.onnx";
pub const AI_TOKENIZER: &str = "tokenizer.json";

// Face Recognition Constants

// models
pub const DETECTION_MODEL: &str = "det_10g.onnx"; // RetinaFace
pub const EMBEDDING_MODEL: &str = "w600k_r50.onnx"; // MobileFaceNet

// Quality thresholds - Recommended Values
pub const MIN_CONFIDENCE: f32 = 0.65; // 0.6-0.7 is standard. 0.65 balances precision/recall.
// pub const MIN_FACE_RATIO: f32 = 0.0; // Disabled. Rely on absolute pixel size instead (better for high-res photos).
// pub const MIN_FACE_SIZE: f32 = 90.0; // 80-112px is minimum for good recognition. 90.0 is a safe high-quality baseline.
pub const MIN_BLUR_SCORE: f32 = 100.0; // Standard Laplacian variance threshold. Below 100 is usually blurry.

// Clustering Constants
pub const K_NEIGHBORS: usize = 80; // Prune edges to Top-K (K-NN)
pub const MIN_SAMPLES: usize = 1; // Minimum samples per cluster
