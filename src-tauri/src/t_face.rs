/**
 * Face Recognition module
 * Handles face detection (RetinaFace) and embedding (MobileFaceNet) using ONNX Runtime.
 */
use crate::{t_cluster, t_sqlite};
use image::DynamicImage;
use ndarray::Array;
use ort::{
    inputs,
    session::{Session, builder::GraphOptimizationLevel},
    value::Value,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};

// cancellation token for face indexing
#[derive(Clone)]
pub struct FaceIndexCancellation(pub Arc<Mutex<bool>>);

// detailed status for face indexing
#[derive(Clone)]
pub struct FaceIndexingStatus(pub Arc<Mutex<bool>>);

// face indexing progress
#[derive(Clone, serde::Serialize)]
pub struct FaceIndexProgress {
    pub current: usize,
    pub total: usize,
    pub faces_found: usize,
    pub phase: String,
}

#[derive(Clone)]
pub struct FaceIndexProgressState(pub Arc<Mutex<FaceIndexProgress>>);

/// Detected face bounding box and landmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
    pub landmarks: Option<Vec<(f32, f32)>>, // 5 facial landmarks
}

/// Face with embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceData {
    pub bbox: FaceBox,
    pub embedding: Vec<f32>,
}

struct Anchor {
    cx: f32,
    cy: f32,
}

pub struct FaceEngine {
    detection_model: Option<Session>, // RetinaFace
    embedding_model: Option<Session>, // MobileFaceNet
}

impl FaceEngine {
    pub fn new() -> Self {
        Self {
            detection_model: None,
            embedding_model: None,
        }
    }

    pub fn load_models(&mut self, app: &AppHandle) -> Result<(), String> {
        if self.detection_model.is_some() {
            return Ok(());
        }

        // Resolve paths
        let resource_dir = app
            .path()
            .resolve("resources/models", tauri::path::BaseDirectory::Resource)
            .map_err(|e| format!("Failed to resolve resource path: {}", e))?;

        let detection_model_path = resource_dir.join("det_10g.onnx");
        let embedding_model_path = resource_dir.join("w600k_r50.onnx");

        // Check if models exist
        if !detection_model_path.exists() {
            return Err(format!(
                "Detection model not found at {:?}",
                detection_model_path
            ));
        }
        if !embedding_model_path.exists() {
            return Err(format!(
                "Embedding model not found at {:?}",
                embedding_model_path
            ));
        }

        // Load Detection Model (RetinaFace)
        let detection_model = Session::builder()
            .map_err(|e| e.to_string())?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| e.to_string())?
            .with_intra_threads(4)
            .map_err(|e| e.to_string())?
            .commit_from_file(&detection_model_path)
            .map_err(|e| format!("Failed to load detection model: {}", e))?;

        self.detection_model = Some(detection_model);

        // Load Embedding Model (MobileFaceNet)
        let embedding_model = Session::builder()
            .map_err(|e| e.to_string())?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| e.to_string())?
            .with_intra_threads(4)
            .map_err(|e| e.to_string())?
            .commit_from_file(&embedding_model_path)
            .map_err(|e| format!("Failed to load embedding model: {}", e))?;

        self.embedding_model = Some(embedding_model);

        Ok(())
    }

    pub fn is_loaded(&self) -> bool {
        self.detection_model.is_some() && self.embedding_model.is_some()
    }

    /// Detect faces implementation (from DynamicImage)
    fn detect_faces(&mut self, img: &DynamicImage) -> Result<Vec<FaceBox>, String> {
        let original_width = img.width() as f32;
        let original_height = img.height() as f32;

        // RetinaFace typically expects 640x640 input
        let target_size = 640;
        // Resize preserving aspect ratio (Letterbox)
        // Use max dimension to fit within target
        let scale = (target_size as f32) / original_width.max(original_height);
        // Use round() to minimize truncation error
        let new_w = (original_width * scale).round() as u32;
        let new_h = (original_height * scale).round() as u32;

        let img_resized = img.resize_exact(new_w, new_h, image::imageops::FilterType::Triangle);
        let rgb_img = img_resized.to_rgb8();

        // Standard InsightFace/RetinaFace preprocessing aligns to Top-Left (0,0)

        // Normalize: (pixel - 127.5) / 128.0
        // Initialize with zeros (padding)
        let mut array = Array::zeros((1, 3, target_size as usize, target_size as usize));
        for (x, y, pixel) in rgb_img.enumerate_pixels() {
            let r = (pixel[0] as f32 - 127.5) / 128.0;
            let g = (pixel[1] as f32 - 127.5) / 128.0;
            let b = (pixel[2] as f32 - 127.5) / 128.0;

            // Place image at top-left (0,0)
            // RetinaFace (InsightFace) models typically expect BGR input
            array[[0, 0, y as usize, x as usize]] = b; // Blue
            array[[0, 1, y as usize, x as usize]] = g; // Green
            array[[0, 2, y as usize, x as usize]] = r; // Red
        }

        let input_value = Value::from_array(array).map_err(|e| e.to_string())?;

        // Use block scope to ensure outputs is dropped before calling nms
        let mut faces = {
            let outputs = self
                .detection_model
                .as_mut()
                .unwrap()
                .run(inputs!["input.1" => input_value])
                .map_err(|e| format!("Detection inference error: {}", e))?;

            let mut all_detections = Vec::new();
            let strides = [8, 16, 32];
            let min_sizes = [[16, 32], [64, 128], [256, 512]]; // Standard RetinaFace config

            // Map output indices based on observation
            // Scores, Boxes, Landmarks indices per stride
            let indices = [
                (0, 3, 6), // Stride 8
                (1, 4, 7), // Stride 16
                (2, 5, 8), // Stride 32
            ];

            let confidence_threshold = 0.6;

            for (i, &stride) in strides.iter().enumerate() {
                let (score_idx, box_idx, _) = indices[i];

                let scores_tensor = &outputs[score_idx];
                let boxes_tensor = &outputs[box_idx];

                let (_, scores_data) = scores_tensor
                    .try_extract_tensor::<f32>()
                    .map_err(|e| format!("Failed stride {} scores: {}", stride, e))?;
                let (_, boxes_data) = boxes_tensor
                    .try_extract_tensor::<f32>()
                    .map_err(|e| format!("Failed stride {} boxes: {}", stride, e))?;

                let feature_map_w = target_size / stride;
                let feature_map_h = target_size / stride;
                let anchors =
                    Self::generate_anchors(stride, &min_sizes[i], feature_map_w, feature_map_h);

                for (j, anchor) in anchors.iter().enumerate() {
                    let score = scores_data[j];
                    if score < confidence_threshold {
                        continue;
                    }

                    // Decode box: [l, t, r, b] (distances from center, normalized by stride)
                    // This assumes SCRFD model (det_10g.onnx) which outputs distances
                    let l = boxes_data[j * 4];
                    let t = boxes_data[j * 4 + 1];
                    let r = boxes_data[j * 4 + 2];
                    let b = boxes_data[j * 4 + 3];

                    // SCRFD uses stride-scaled distances
                    // x1 = cx - l * stride
                    // y1 = cy - t * stride
                    // x2 = cx + r * stride
                    // y2 = cy + b * stride

                    let x1 = anchor.cx - l * stride as f32;
                    let y1 = anchor.cy - t * stride as f32;
                    let x2 = anchor.cx + r * stride as f32;
                    let y2 = anchor.cy + b * stride as f32;

                    // Scale back to original image
                    // Use effective scale factors derived from actual resized dimensions
                    let inv_scale_x = original_width / new_w as f32;
                    let inv_scale_y = original_height / new_h as f32;

                    // Scale directly (no padding offset)
                    let original_x1 = x1 * inv_scale_x;
                    let original_y1 = y1 * inv_scale_y;
                    let original_x2 = x2 * inv_scale_x;
                    let original_y2 = y2 * inv_scale_y;

                    all_detections.push(FaceBox {
                        x: original_x1,
                        y: original_y1,
                        width: original_x2 - original_x1,
                        height: original_y2 - original_y1,
                        confidence: score,
                        landmarks: None,
                    });
                }
            }

            all_detections
        };

        // Non-maximum suppression
        faces = self.nms(faces, 0.4);

        if faces.is_empty() {
            // No faces found after NMS
        }

        Ok(faces)
    }

    /// Generate anchors for a specific stride
    fn generate_anchors(
        stride: u32,
        min_sizes: &[u32],
        feature_w: u32,
        feature_h: u32,
    ) -> Vec<Anchor> {
        let mut anchors =
            Vec::with_capacity((feature_w * feature_h * min_sizes.len() as u32) as usize);

        for y in 0..feature_h {
            for x in 0..feature_w {
                for &_min_size in min_sizes {
                    // Dense anchor centers
                    // Adjusted to 0.0 (top-left) from 0.5 (center) to fix systematic bottom-right shift
                    let cx = (x as f32) * stride as f32;
                    let cy = (y as f32) * stride as f32;

                    anchors.push(Anchor { cx, cy });
                }
            }
        }
        anchors
    }

    /// Get face embedding implementation (from DynamicImage)
    fn get_face_embedding(
        &mut self,
        img: &DynamicImage,
        bbox: &FaceBox,
    ) -> Result<Vec<f32>, String> {
        // Crop face region with some padding
        let padding = 0.2;
        let x = (bbox.x - bbox.width * padding).max(0.0) as u32;
        let y = (bbox.y - bbox.height * padding).max(0.0) as u32;
        let w = (bbox.width * (1.0 + 2.0 * padding)) as u32;
        let h = (bbox.height * (1.0 + 2.0 * padding)) as u32;

        let max_x = (x + w).min(img.width());
        let max_y = (y + h).min(img.height());

        let face_crop = img.crop_imm(x, y, max_x - x, max_y - y);

        // Resize to MobileFaceNet input size (112x112)
        let face_resized = face_crop.resize_exact(112, 112, image::imageops::FilterType::Triangle);
        let rgb_face = face_resized.to_rgb8();

        // Normalize: (pixel - 127.5) / 128.0
        let mut array = Array::zeros((1, 3, 112, 112));

        for (fx, fy, pixel) in rgb_face.enumerate_pixels() {
            let r = (pixel[0] as f32 - 127.5) / 128.0;
            let g = (pixel[1] as f32 - 127.5) / 128.0;
            let b = (pixel[2] as f32 - 127.5) / 128.0;

            array[[0, 0, fy as usize, fx as usize]] = r;
            array[[0, 1, fy as usize, fx as usize]] = g;
            array[[0, 2, fy as usize, fx as usize]] = b;
        }

        let input_value = Value::from_array(array).map_err(|e| e.to_string())?;

        let outputs = self
            .embedding_model
            .as_mut()
            .unwrap()
            .run(inputs!["input.1" => input_value])
            .map_err(|e| format!("Embedding inference error: {}", e))?;

        let embedding = &outputs[0];
        let (_, embedding_data) = embedding
            .try_extract_tensor::<f32>()
            .map_err(|e| format!("Failed to extract embedding: {}", e))?;

        // Normalize embedding to unit vector
        let emb_vec: Vec<f32> = embedding_data.iter().copied().collect();
        let norm: f32 = emb_vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        let normalized: Vec<f32> = emb_vec.iter().map(|x| x / norm).collect();

        Ok(normalized)
    }

    /// Compute cosine similarity between two embeddings
    #[allow(dead_code)]
    pub fn compare_faces(emb1: &[f32], emb2: &[f32]) -> f32 {
        if emb1.len() != emb2.len() {
            return 0.0;
        }
        // Embeddings are already normalized, so dot product = cosine similarity
        emb1.iter().zip(emb2.iter()).map(|(a, b)| a * b).sum()
    }

    /// Process image: detect all faces and get embeddings
    /// Filters out low-quality faces (low confidence, small size, blurry)
    pub fn process_image(&mut self, image_path: &str) -> Result<Vec<FaceData>, String> {
        // Get image dimensions
        let img = image::open(image_path).map_err(|e| format!("Failed to open image: {}", e))?;

        // Quality thresholds
        // Quality thresholds - Recommended Values
        const MIN_CONFIDENCE: f32 = 0.65; // 0.6-0.7 is standard. 0.65 balances precision/recall.
        const MIN_FACE_RATIO: f32 = 0.0; // Disabled. Rely on absolute pixel size instead (better for high-res photos).
        const MIN_FACE_SIZE: f32 = 90.0; // 80-112px is minimum for good recognition. 90.0 is a safe high-quality baseline.
        // const MIN_BLUR_SCORE: f32 = 100.0; // Standard Laplacian variance threshold. Below 100 is usually blurry.

        let img_width = img.width() as f32;
        let img_height = img.height() as f32;
        let img_area = img_width * img_height;

        let faces = self.detect_faces(&img)?;

        let mut results = Vec::new();
        for face in faces {
            // Filter 1: Skip low confidence faces
            if face.confidence < MIN_CONFIDENCE {
                continue;
            }

            // Filter 2: Skip very small faces (likely background people)
            let face_area = face.width * face.height;
            if face_area / img_area < MIN_FACE_RATIO {
                continue;
            }

            // Filter 3: Skip faces smaller than minimum pixel size
            if face.width < MIN_FACE_SIZE || face.height < MIN_FACE_SIZE {
                continue;
            }

            // Filter 4: Skip blurry faces
            // let blur_score = self.calculate_blur_score(&img, &face);
            // if blur_score < MIN_BLUR_SCORE {
            //     continue;
            // }

            // Get embedding for quality face
            let embedding = self.get_face_embedding(&img, &face)?;
            results.push(FaceData {
                bbox: face,
                embedding,
            });
        }

        Ok(results)
    }

    /// Calculate blur score using Variance of Laplacian
    // fn calculate_blur_score(&self, img: &DynamicImage, bbox: &FaceBox) -> f32 {
    //     let x = bbox.x.max(0.0) as u32;
    //     let y = bbox.y.max(0.0) as u32;
    //     // Check bounds to ensure we don't crash on cropping
    //     let w = bbox.width.min(img.width() as f32 - bbox.x) as u32;
    //     let h = bbox.height.min(img.height() as f32 - bbox.y) as u32;

    //     if w < 3 || h < 3 {
    //         return 0.0;
    //     }

    //     let crop = img.crop_imm(x, y, w, h).to_luma8();
    //     let (width, height) = crop.dimensions();

    //     let mut mean = 0.0;
    //     let mut count = 0;
    //     let mut laplacian_values = Vec::with_capacity((width * height) as usize);

    //     // Simple Laplacian kernel:
    //     //  0  1  0
    //     //  1 -4  1
    //     //  0  1  0

    //     for y in 1..height - 1 {
    //         for x in 1..width - 1 {
    //             let p = crop.get_pixel(x, y).0[0] as i16;
    //             let top = crop.get_pixel(x, y - 1).0[0] as i16;
    //             let bottom = crop.get_pixel(x, y + 1).0[0] as i16;
    //             let left = crop.get_pixel(x - 1, y).0[0] as i16;
    //             let right = crop.get_pixel(x + 1, y).0[0] as i16;

    //             let sum = top + bottom + left + right - 4 * p;
    //             let val = sum as f32;

    //             laplacian_values.push(val);
    //             mean += val;
    //             count += 1;
    //         }
    //     }

    //     if count == 0 {
    //         return 0.0;
    //     }

    //     mean /= count as f32;

    //     let variance = laplacian_values
    //         .iter()
    //         .fold(0.0, |acc, &val| acc + (val - mean).powi(2));
    //     variance / count as f32
    // }

    /// Non-maximum suppression
    fn nms(&self, mut boxes: Vec<FaceBox>, iou_threshold: f32) -> Vec<FaceBox> {
        boxes.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        let mut keep = Vec::new();
        let mut suppressed = vec![false; boxes.len()];

        for i in 0..boxes.len() {
            if suppressed[i] {
                continue;
            }
            keep.push(boxes[i].clone());

            for j in (i + 1)..boxes.len() {
                if suppressed[j] {
                    continue;
                }
                if self.iou(&boxes[i], &boxes[j]) > iou_threshold {
                    suppressed[j] = true;
                }
            }
        }

        keep
    }

    /// Intersection over Union
    fn iou(&self, a: &FaceBox, b: &FaceBox) -> f32 {
        let x1 = a.x.max(b.x);
        let y1 = a.y.max(b.y);
        let x2 = (a.x + a.width).min(b.x + b.width);
        let y2 = (a.y + a.height).min(b.y + b.height);

        let inter_area = (x2 - x1).max(0.0) * (y2 - y1).max(0.0);
        let a_area = a.width * a.height;
        let b_area = b.width * b.height;

        inter_area / (a_area + b_area - inter_area)
    }
}

#[derive(Clone)]
pub struct FaceState(pub std::sync::Arc<Mutex<FaceEngine>>);

pub fn run_face_indexing(
    app_handle: AppHandle,
    face_state: FaceState,
    cancel_token_struct: FaceIndexCancellation,
    status_token_struct: FaceIndexingStatus,
    progress_token_struct: FaceIndexProgressState,
    cluster_epsilon: Option<f32>,
) -> Result<(), String> {
    let cancel_token = cancel_token_struct.0.clone();
    let status_token = status_token_struct.0.clone();
    let progress_token = progress_token_struct.0.clone();
    // Use provided epsilon or default to 0.42
    let epsilon = cluster_epsilon.unwrap_or(0.42);

    // Check if already running
    {
        let mut running = status_token.lock().unwrap();
        if *running {
            return Err("Face indexing is already running".to_string());
        }
        *running = true;
    }

    // Reset cancellation flag
    *cancel_token.lock().unwrap() = false;

    // Reset progress
    {
        let mut progress = progress_token.lock().unwrap();
        progress.current = 0;
        progress.total = 0;
        progress.faces_found = 0;
        progress.phase = "indexing".to_string();
    }

    tauri::async_runtime::spawn(async move {
        // 1. Initialization
        let reset_status = || {
            if let Ok(mut running) = status_token.lock() {
                *running = false;
            }
        };

        // Load models if not already loaded
        {
            let mut engine = face_state.0.lock().unwrap();
            if !engine.is_loaded() {
                if let Err(e) = engine.load_models(&app_handle) {
                    eprintln!("Failed to load face models: {}", e);
                    let _ = app_handle.emit(
                        "face_index_finished",
                        serde_json::json!({
                            "total_faces": 0,
                            "total_persons": 0,
                            "cancelled": false,
                            "error": e.to_string()
                        }),
                    );
                    reset_status();
                    return;
                }
            }
        }

        // 2. Preparation (Get files and stats)
        let (processed_count, existing_faces_count) = match t_sqlite::Face::get_stats() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to get stats: {}", e);
                (0, 0)
            }
        };

        let files = match t_sqlite::Face::get_unprocessed_image_files() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to get unprocessed files: {}", e);
                let _ = app_handle.emit(
                    "face_index_finished",
                    serde_json::json!({
                        "total_faces": 0,
                        "total_persons": 0,
                        "cancelled": false,
                        "error": e
                    }),
                );
                reset_status();
                return;
            }
        };

        let total_files = processed_count + files.len();
        let mut total_faces = existing_faces_count;
        let mut current = processed_count;

        // Init progress
        {
            let mut progress = progress_token.lock().unwrap();
            progress.total = total_files;
            progress.current = current;
            progress.faces_found = total_faces;
            progress.phase = "indexing".to_string();
        }

        let _ = app_handle.emit(
            "face_index_progress",
            serde_json::json!({
                "current": current,
                "total": total_files,
                "faces_found": total_faces,
                "phase": "indexing"
            }),
        );

        // 3. Image Processing Loop
        let mut cancelled = false;

        for (file_id, file_path) in files {
            if *cancel_token.lock().unwrap() {
                cancelled = true;
                break;
            }

            current += 1;

            let mut engine = face_state.0.lock().unwrap();

            let process_result = engine.process_image(&file_path);

            match process_result {
                Ok(faces) => {
                    let has_faces = !faces.is_empty();
                    let status = if has_faces { 1 } else { 2 };

                    if let Err(e) = t_sqlite::Face::mark_scanned(file_id, status) {
                        eprintln!("Failed to mark file {} as scanned: {}", file_id, e);
                    }

                    if has_faces {
                        for face_data in &faces {
                            let bbox_json = serde_json::json!({
                                "x": face_data.bbox.x,
                                "y": face_data.bbox.y,
                                "width": face_data.bbox.width,
                                "height": face_data.bbox.height,
                                "confidence": face_data.bbox.confidence,
                            })
                            .to_string();

                            match t_sqlite::Face::add(file_id, &bbox_json, &face_data.embedding) {
                                Ok(_) => total_faces += 1,
                                Err(e) => eprintln!("Failed to store face: {}", e),
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to process image {}: {}", file_path, e);
                }
            }

            // Periodic progress update (every 10 files or at end)
            if current % 10 == 0 || current == total_files {
                {
                    let mut progress = progress_token.lock().unwrap();
                    progress.current = current;
                    progress.faces_found = total_faces;
                }

                let _ = app_handle.emit(
                    "face_index_progress",
                    serde_json::json!({
                        "current": current,
                        "total": total_files,
                        "faces_found": total_faces,
                        "phase": "indexing"
                    }),
                );
            }
        }

        if cancelled {
            let _ = app_handle.emit(
                "face_index_finished",
                serde_json::json!({
                    "total_faces": total_faces,
                    "total_persons": 0,
                    "cancelled": true
                }),
            );
            reset_status();
            return;
        }

        // 4. Clustering
        {
            let mut progress = progress_token.lock().unwrap();
            progress.phase = "clustering".to_string();
        }

        let _ = app_handle.emit(
            "face_index_progress",
            serde_json::json!({
                "current": total_files,
                "total": total_files,
                "faces_found": total_faces,
                "phase": "clustering"
            }),
        );

        let cancel_token_cluster = cancel_token.clone();
        let total_persons = match t_cluster::cluster_faces(
            epsilon,
            |progress| {
                let _ = app_handle.emit(
                    "cluster_progress",
                    serde_json::json!({
                        "phase": progress.phase,
                        "current": progress.current,
                        "total": progress.total,
                    }),
                );
            },
            || {
                // Check if user has cancelled
                *cancel_token_cluster.lock().unwrap()
            },
        ) {
            Ok(count) => count,
            Err(e) => {
                eprintln!("Clustering failed: {}", e);
                0
            }
        };

        // 5. Finished
        let _ = app_handle.emit(
            "face_index_finished",
            serde_json::json!({
                "total_faces": total_faces,
                "total_persons": total_persons,
                "cancelled": false
            }),
        );
        reset_status();
    });

    Ok(())
}
