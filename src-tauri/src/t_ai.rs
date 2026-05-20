/**
 * AI Engine module
 * Handles ONNX Runtime sessions and model inference.
 */
use crate::t_common;
use image::DynamicImage;
use ndarray::{Array, Array4};
use ort::{
    inputs,
    session::{Session, builder::GraphOptimizationLevel},
    value::Value,
};
use reqwest::header::{CONTENT_RANGE, RANGE, USER_AGENT};
use serde::Serialize;
use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::AsyncWriteExt;
use tokenizers::Tokenizer;

pub struct AiEngine {
    text_model: Option<Session>,
    vision_model: Option<Session>,
    tokenizer: Option<Tokenizer>,
    text_model_kind: ImageSearchTextModel,
}

const AI_INTRA_THREADS: usize = 2;
const MULTILINGUAL_TEXT_MODEL_URL: &str =
    "https://github.com/julyx10/lap-binaries/releases/download/models/text_model.onnx";
const MULTILINGUAL_TOKENIZER_URL: &str =
    "https://github.com/julyx10/lap-binaries/releases/download/models/tokenizer.json";
const MULTILINGUAL_RELEASE_API_URL: &str =
    "https://api.github.com/repos/julyx10/lap-binaries/releases/tags/models";
static MULTILINGUAL_MODEL_DOWNLOAD_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ImageSearchTextModel {
    Default,
    Multilingual,
}

impl ImageSearchTextModel {
    pub fn from_i64(value: i64) -> Self {
        match value {
            1 => Self::Multilingual,
            _ => Self::Default,
        }
    }

    pub fn as_i64(self) -> i64 {
        match self {
            Self::Default => 0,
            Self::Multilingual => 1,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageSearchModelStatus {
    pub active_model: i64,
    pub multilingual_available: bool,
}

#[derive(Debug, Clone)]
struct TextModelPaths {
    model: PathBuf,
    tokenizer: PathBuf,
}

impl AiEngine {
    pub fn new() -> Self {
        Self {
            text_model: None,
            vision_model: None,
            tokenizer: None,
            text_model_kind: ImageSearchTextModel::Default,
        }
    }

    pub fn load_models(&mut self, app: &AppHandle) -> Result<(), String> {
        if self.text_model.is_some() && self.vision_model.is_some() {
            return Ok(());
        }

        println!("Loading AI Models...");

        let resource_dir = Self::resource_model_dir(app)?;
        let vision_model_path = resource_dir.join(t_common::AI_VISION_MODEL);

        // Load Vision Model
        if self.vision_model.is_none() {
            let vision_model = Self::load_session(&vision_model_path, "vision")?;
            self.vision_model = Some(vision_model);
        }

        if self.text_model.is_none() {
            self.set_text_model(app, ImageSearchTextModel::Default)?;
        }

        println!("AI Models Loaded Successfully!");
        Ok(())
    }

    fn load_session(path: &Path, model_name: &str) -> Result<Session, String> {
        Session::builder()
            .map_err(|e| e.to_string())?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| e.to_string())?
            .with_intra_threads(AI_INTRA_THREADS)
            .map_err(|e| e.to_string())?
            .commit_from_file(path)
            .map_err(|e| format!("Failed to load {} model from {:?}: {}", model_name, path, e))
    }

    fn resource_model_dir(app: &AppHandle) -> Result<PathBuf, String> {
        #[cfg(debug_assertions)]
        {
            let manifest_dir = env!("CARGO_MANIFEST_DIR");
            let dev_path = std::path::PathBuf::from(manifest_dir).join("resources/models");
            if dev_path.exists() {
                return Ok(dev_path);
            }
        }

        app.path()
            .resolve("models", tauri::path::BaseDirectory::Resource)
            .map_err(|e| format!("Failed to resolve resource path: {}", e))
    }

    fn multilingual_model_dir(_app: &AppHandle) -> Result<PathBuf, String> {
        crate::t_config::get_app_data_dir().map(|dir| dir.join("models").join("multilingual"))
    }

    fn text_model_paths(app: &AppHandle, model: ImageSearchTextModel) -> Result<TextModelPaths, String> {
        let model_dir = match model {
            ImageSearchTextModel::Default => Self::resource_model_dir(app)?,
            ImageSearchTextModel::Multilingual => Self::multilingual_model_dir(app)?,
        };

        Ok(TextModelPaths {
            model: model_dir.join(t_common::AI_TEXT_MODEL),
            tokenizer: model_dir.join(t_common::AI_TOKENIZER),
        })
    }

    pub fn is_multilingual_model_available(app: &AppHandle) -> bool {
        Self::text_model_paths(app, ImageSearchTextModel::Multilingual)
            .map(|paths| paths.model.exists() && paths.tokenizer.exists())
            .unwrap_or(false)
    }

    pub fn model_status(&self, app: &AppHandle) -> ImageSearchModelStatus {
        ImageSearchModelStatus {
            active_model: self.text_model_kind.as_i64(),
            multilingual_available: Self::is_multilingual_model_available(app),
        }
    }

    pub fn set_text_model(
        &mut self,
        app: &AppHandle,
        model: ImageSearchTextModel,
    ) -> Result<(), String> {
        if self.text_model.is_some() && self.text_model_kind == model {
            return Ok(());
        }

        let paths = Self::text_model_paths(app, model)?;
        if !paths.model.exists() || !paths.tokenizer.exists() {
            return Err(format!("Image search model files are missing for {:?}", model));
        }

        let tokenizer = Tokenizer::from_file(&paths.tokenizer)
            .map_err(|e| format!("Failed to load tokenizer from {:?}: {}", paths.tokenizer, e))?;
        let text_model = Self::load_session(&paths.model, "text")?;

        self.tokenizer = Some(tokenizer);
        self.text_model = Some(text_model);
        self.text_model_kind = model;
        Ok(())
    }

    pub fn is_loaded(&self) -> bool {
        self.text_model.is_some() && self.vision_model.is_some() && self.tokenizer.is_some()
    }

    pub fn encode_text(&mut self, text: &str) -> Result<Vec<f32>, String> {
        if !self.is_loaded() {
            return Err("AI models not loaded".to_string());
        }

        let tokenizer = self.tokenizer.as_ref().unwrap();
        let encoding = tokenizer
            .encode(text, true)
            .map_err(|e| format!("Tokenization error: {}", e))?;

        let input_ids = encoding.get_ids();
        let attention_mask = encoding.get_attention_mask();

        let input_ids_array = Array::from_shape_vec(
            (1, input_ids.len()),
            input_ids.iter().map(|&x| x as i64).collect(),
        )
        .map_err(|e| e.to_string())?;

        let input_ids_value = Value::from_array(input_ids_array).map_err(|e| e.to_string())?;
        let attention_mask_array = Array::from_shape_vec(
            (1, attention_mask.len()),
            attention_mask.iter().map(|&x| x as i64).collect(),
        )
        .map_err(|e| e.to_string())?;
        let attention_mask_value =
            Value::from_array(attention_mask_array).map_err(|e| e.to_string())?;

        let uses_attention_mask = self
            .text_model
            .as_ref()
            .unwrap()
            .inputs
            .iter()
            .any(|input| input.name == "attention_mask");

        let outputs = if uses_attention_mask {
            self.text_model.as_mut().unwrap().run(inputs![
                "input_ids" => input_ids_value,
                "attention_mask" => attention_mask_value,
            ])
        } else {
            self.text_model.as_mut().unwrap().run(inputs![
                "input_ids" => input_ids_value,
            ])
        }
        .map_err(|e| format!("Inference error: {}", e))?;

        let (embedding, first_token_only) = if let Some(vals) = outputs.get("pooler_output") {
            (vals, false)
        } else if let Some(vals) = outputs.get("text_embeds") {
            (vals, false)
        } else if let Some(vals) = outputs.get("last_hidden_state") {
            (vals, true)
        } else {
            (&outputs[0], true)
        };

        Self::extract_text_embedding(embedding, first_token_only)
    }

    fn extract_text_embedding(
        embedding: &ort::value::DynValue,
        first_token_only: bool,
    ) -> Result<Vec<f32>, String> {
        let (shape, embedding_data) = embedding
            .try_extract_tensor::<f32>()
            .map_err(|e| format!("Failed to extract tensor: {}", e))?;

        if first_token_only && shape.len() >= 3 {
            let hidden_size = shape
                .last()
                .copied()
                .filter(|dim| *dim > 0)
                .ok_or_else(|| format!("Invalid text embedding shape: {}", shape))?
                as usize;
            if embedding_data.len() < hidden_size {
                return Err(format!("Text embedding data is shorter than shape {}", shape));
            }
            return Ok(embedding_data[..hidden_size].to_vec());
        }

        Ok(embedding_data.to_vec())
    }

    pub fn encode_image(&mut self, image_path: &str) -> Result<Vec<f32>, String> {
        if !self.is_loaded() {
            return Err("AI models not loaded".to_string());
        }

        let image_input = self.preprocess_image(image_path)?;
        self.run_vision_model(image_input)
    }

    pub fn encode_image_from_bytes(&mut self, image_bytes: &[u8]) -> Result<Vec<f32>, String> {
        if !self.is_loaded() {
            return Err("AI models not loaded".to_string());
        }

        let img = image::load_from_memory(image_bytes)
            .map_err(|e| format!("Failed to load image from memory: {}", e))?;
        let image_input = self.preprocess_dynamic_image(img)?;

        self.run_vision_model(image_input)
    }

    fn run_vision_model(&mut self, image_input: Array4<f32>) -> Result<Vec<f32>, String> {
        let image_input_value = Value::from_array(image_input).map_err(|e| e.to_string())?;

        let outputs = self
            .vision_model
            .as_mut()
            .unwrap()
            .run(inputs![
                "pixel_values" => image_input_value,
            ])
            .map_err(|e| format!("Inference error: {}", e))?;

        let embedding = if let Some(vals) = outputs.get("pooler_output") {
            vals
        } else if let Some(vals) = outputs.get("image_embeds") {
            vals
        } else {
            &outputs[0]
        };

        let (_, embedding_data) = embedding
            .try_extract_tensor::<f32>()
            .map_err(|e| format!("Failed to extract tensor: {}", e))?;

        Ok(embedding_data.to_vec())
    }

    fn preprocess_image(&self, path: &str) -> Result<Array4<f32>, String> {
        let img = image::open(path).map_err(|e| format!("Failed to open image: {}", e))?;
        self.preprocess_dynamic_image(img)
    }

    fn preprocess_dynamic_image(&self, img: DynamicImage) -> Result<Array4<f32>, String> {
        // resize to 224x224
        let img = img.resize_exact(224, 224, image::imageops::FilterType::Triangle);
        let rgb_img = img.to_rgb8();

        // Normalize
        let mean = [0.48145466, 0.4578275, 0.40821073];
        let std = [0.26862954, 0.26130258, 0.27577711];

        let mut array = Array::zeros((1, 3, 224, 224));

        for (x, y, pixel) in rgb_img.enumerate_pixels() {
            let r = (pixel[0] as f32 / 255.0 - mean[0]) / std[0];
            let g = (pixel[1] as f32 / 255.0 - mean[1]) / std[1];
            let b = (pixel[2] as f32 / 255.0 - mean[2]) / std[2];

            array[[0, 0, y as usize, x as usize]] = r;
            array[[0, 1, y as usize, x as usize]] = g;
            array[[0, 2, y as usize, x as usize]] = b;
        }

        Ok(array)
    }
}

pub struct AiState(pub Mutex<AiEngine>);

async fn get_remote_file_size(client: &reqwest::Client, url: &str) -> Option<u64> {
    let response = client
        .get(url)
        .header(RANGE, "bytes=0-0")
        .timeout(Duration::from_secs(20))
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?;

    if let Some(content_range) = response.headers().get(CONTENT_RANGE) {
        let content_range = content_range.to_str().ok()?;
        if let Some((_, total)) = content_range.rsplit_once('/') {
            if total != "*" {
                return total.parse::<u64>().ok();
            }
        }
    }

    response.content_length()
}

async fn get_release_asset_total_size(
    client: &reqwest::Client,
    files: &[(&str, &str, &str)],
) -> Option<u64> {
    let response = client
        .get(MULTILINGUAL_RELEASE_API_URL)
        .header(USER_AGENT, "Lap")
        .timeout(Duration::from_secs(20))
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?;
    let value = serde_json::from_str::<serde_json::Value>(&response.text().await.ok()?).ok()?;
    let assets = value.get("assets")?.as_array()?;
    let mut total_size = 0u64;

    for (_, filename, _) in files {
        let asset = assets
            .iter()
            .find(|asset| asset.get("name").and_then(|name| name.as_str()) == Some(*filename))?;
        total_size += asset.get("size")?.as_u64()?;
    }

    Some(total_size)
}

async fn get_download_total_size(client: &reqwest::Client, files: &[(&str, &str, &str)]) -> u64 {
    if let Some(total_size) = get_release_asset_total_size(client, files).await {
        return total_size;
    }

    let mut total_size = 0u64;
    for (url, _, _) in files.iter() {
        match get_remote_file_size(client, url).await {
            Some(file_size) => total_size += file_size,
            None => return 0,
        }
    }
    total_size
}

fn is_current_multilingual_download(download_id: u64) -> bool {
    MULTILINGUAL_MODEL_DOWNLOAD_ID.load(Ordering::SeqCst) == download_id
}

fn ensure_current_multilingual_download(download_id: u64, temp_dir: &Path) -> Result<(), String> {
    if is_current_multilingual_download(download_id) {
        return Ok(());
    }

    let _ = std::fs::remove_dir_all(temp_dir);
    Err("Download canceled".to_string())
}

async fn clean_multilingual_download_temp_dirs(model_dir: &Path) {
    let Some(parent) = model_dir.parent() else {
        return;
    };
    let Some(model_name) = model_dir.file_name().and_then(|name| name.to_str()) else {
        return;
    };
    let temp_prefix = format!("{}.download", model_name);
    let Ok(mut entries) = tokio::fs::read_dir(parent).await else {
        return;
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let should_remove = entry
            .file_name()
            .to_str()
            .map(|name| name == temp_prefix || name.starts_with(&format!("{}.", temp_prefix)))
            .unwrap_or(false);
        if should_remove {
            let _ = tokio::fs::remove_dir_all(entry.path()).await;
        }
    }
}

pub async fn download_multilingual_text_model(app: AppHandle) -> Result<(), String> {
    let download_id = MULTILINGUAL_MODEL_DOWNLOAD_ID.fetch_add(1, Ordering::SeqCst) + 1;
    let model_dir = AiEngine::multilingual_model_dir(&app)?;
    clean_multilingual_download_temp_dirs(&model_dir).await;
    let temp_dir = model_dir.with_extension(format!("download.{}", download_id));
    match tokio::fs::remove_dir_all(&temp_dir).await {
        Ok(_) => {}
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => return Err(format!("Failed to clean temporary download files: {}", e)),
    }
    tokio::fs::create_dir_all(&temp_dir)
        .await
        .map_err(|e| e.to_string())?;

    let files = [
        (
            MULTILINGUAL_TEXT_MODEL_URL,
            t_common::AI_TEXT_MODEL,
            "text_model",
        ),
        (
            MULTILINGUAL_TOKENIZER_URL,
            t_common::AI_TOKENIZER,
            "tokenizer",
        ),
    ];
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(3))
        .build()
        .map_err(|e| format!("Failed to create download client: {}", e))?;
    let total_files = files.len() as f64;
    let mut downloaded_total = 0u64;
    let expected_total = get_download_total_size(&client, &files).await;
    ensure_current_multilingual_download(download_id, &temp_dir)?;

    let _ = app.emit(
        "image_search_model_download_progress",
        serde_json::json!({
            "progress": 0,
            "downloadedBytes": 0,
            "totalBytes": expected_total,
            "downloadId": download_id,
            "file": "start",
        }),
    );

    for (index, (url, filename, label)) in files.iter().enumerate() {
        ensure_current_multilingual_download(download_id, &temp_dir)?;
        let response = client
            .get(*url)
            .send()
            .await
            .map_err(|e| format!("Failed to download {}: {}", filename, e))?
            .error_for_status()
            .map_err(|e| format!("Failed to download {}: {}", filename, e))?;

        let path = temp_dir.join(filename);
        let mut file = tokio::fs::File::create(&path)
            .await
            .map_err(|e| e.to_string())?;
        let content_length = response.content_length().unwrap_or(0);
        let mut downloaded = 0u64;
        let mut response = response;
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| format!("Failed to read {}: {}", filename, e))?
        {
            if chunk.is_empty() {
                continue;
            }
            ensure_current_multilingual_download(download_id, &temp_dir)?;
            file.write_all(&chunk).await.map_err(|e| e.to_string())?;
            downloaded += chunk.len() as u64;
            downloaded_total += chunk.len() as u64;

            let file_progress = if content_length > 0 {
                (downloaded as f64 / content_length as f64).min(1.0)
            } else {
                0.0
            };
            let progress = if expected_total > 0 {
                ((downloaded_total as f64 / expected_total as f64).min(1.0) * 100.0).round()
                    as i64
            } else {
                (((index as f64 + file_progress) / total_files) * 100.0).round() as i64
            };
            let _ = app.emit(
                "image_search_model_download_progress",
                serde_json::json!({
                    "progress": progress,
                    "downloadedBytes": downloaded_total,
                    "totalBytes": expected_total,
                    "downloadId": download_id,
                    "file": label,
                }),
            );
        }
        file.flush().await.map_err(|e| e.to_string())?;
        if downloaded == 0 {
            return Err(format!("Downloaded {} is empty", filename));
        }

        let progress = if expected_total > 0 {
            ((downloaded_total as f64 / expected_total as f64).min(1.0) * 100.0).round() as i64
        } else {
            ((((index + 1) as f64) / total_files) * 100.0).round() as i64
        };
        let _ = app.emit(
            "image_search_model_download_progress",
            serde_json::json!({
                "progress": progress,
                "downloadedBytes": downloaded_total,
                "totalBytes": expected_total,
                "downloadId": download_id,
                "file": label,
            }),
        );
    }

    ensure_current_multilingual_download(download_id, &temp_dir)?;
    let temp_text_model_path = temp_dir.join(t_common::AI_TEXT_MODEL);
    let temp_tokenizer_path = temp_dir.join(t_common::AI_TOKENIZER);
    Tokenizer::from_file(&temp_tokenizer_path).map_err(|e| {
        let _ = std::fs::remove_dir_all(&temp_dir);
        format!("Downloaded tokenizer is invalid: {}", e)
    })?;
    if let Err(e) = AiEngine::load_session(&temp_text_model_path, "text") {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(format!("Downloaded text model is invalid: {}", e));
    }

    tokio::fs::create_dir_all(&model_dir)
        .await
        .map_err(|e| e.to_string())?;
    ensure_current_multilingual_download(download_id, &temp_dir)?;
    for (_, filename, _) in files {
        let dest = model_dir.join(filename);
        let temp = temp_dir.join(filename);
        let _ = tokio::fs::remove_file(&dest).await;
        tokio::fs::rename(temp, dest)
            .await
            .map_err(|e| e.to_string())?;
    }
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;

    let _ = app.emit(
        "image_search_model_download_progress",
        serde_json::json!({
            "progress": 100,
            "downloadedBytes": downloaded_total,
            "totalBytes": expected_total,
            "downloadId": download_id,
            "file": "complete",
        }),
    );

    Ok(())
}

pub async fn cancel_multilingual_text_model_download(app: AppHandle) -> Result<(), String> {
    MULTILINGUAL_MODEL_DOWNLOAD_ID.fetch_add(1, Ordering::SeqCst);
    let model_dir = AiEngine::multilingual_model_dir(&app)?;
    clean_multilingual_download_temp_dirs(&model_dir).await;
    Ok(())
}
