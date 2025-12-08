/**
 * AI Engine module
 * Handles ONNX Runtime sessions and model inference.
 */
use ndarray::{Array, Array4};
use ort::{
    inputs,
    session::{Session, builder::GraphOptimizationLevel},
    value::Value,
};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use tokenizers::Tokenizer;

pub struct AiEngine {
    text_model: Option<Session>,
    vision_model: Option<Session>,
    tokenizer: Option<Tokenizer>,
}

impl AiEngine {
    pub fn new() -> Self {
        Self {
            text_model: None,
            vision_model: None,
            tokenizer: None,
        }
    }

    pub fn load_models(&mut self, app: &AppHandle) -> Result<(), String> {
        if self.text_model.is_some() {
            return Ok(());
        }

        println!("Loading AI Models...");

        // Resolve paths
        let resource_dir = app
            .path()
            .resolve("resources/models", tauri::path::BaseDirectory::Resource)
            .map_err(|e| format!("Failed to resolve resource path: {}", e))?;

        let text_model_path = resource_dir.join("text_model.onnx");
        let vision_model_path = resource_dir.join("vision_model.onnx");
        let tokenizer_path = resource_dir.join("tokenizer.json");

        // Load Tokenizer
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| format!("Failed to load tokenizer from {:?}: {}", tokenizer_path, e))?;
        self.tokenizer = Some(tokenizer);

        // Load Text Model
        let text_model = Session::builder()
            .map_err(|e| e.to_string())?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| e.to_string())?
            .with_intra_threads(4)
            .map_err(|e| e.to_string())?
            .commit_from_file(&text_model_path)
            .map_err(|e| {
                format!(
                    "Failed to load text model from {:?}: {}",
                    text_model_path, e
                )
            })?;

        self.text_model = Some(text_model);

        // Load Vision Model (Lazy load or load here for now)
        // For MVP locally, load both
        let vision_model = Session::builder()
            .map_err(|e| e.to_string())?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| e.to_string())?
            .with_intra_threads(4)
            .map_err(|e| e.to_string())?
            .commit_from_file(&vision_model_path)
            .map_err(|e| {
                format!(
                    "Failed to load vision model from {:?}: {}",
                    vision_model_path, e
                )
            })?;

        self.vision_model = Some(vision_model);

        println!("AI Models Loaded Successfully!");
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
        // let attention_mask = encoding.get_attention_mask(); // Not used by this specific ONNX model

        let input_ids_array = Array::from_shape_vec(
            (1, input_ids.len()),
            input_ids.iter().map(|&x| x as i64).collect(),
        )
        .map_err(|e| e.to_string())?;

        // let attention_mask_array = Array::from_shape_vec(
        //     (1, attention_mask.len()),
        //     attention_mask.iter().map(|&x| x as i64).collect(),
        // )
        // .map_err(|e| e.to_string())?;

        let input_ids_value = Value::from_array(input_ids_array).map_err(|e| e.to_string())?;
        // let attention_mask_value =
        //     Value::from_array(attention_mask_array).map_err(|e| e.to_string())?;

        let outputs = self
            .text_model
            .as_mut()
            .unwrap()
            .run(inputs![
                "input_ids" => input_ids_value,
                // "attention_mask" => attention_mask_value
            ])
            .map_err(|e| format!("Inference error: {}", e))?;

        // Extract pooler_output (usually the last output or named "pooler_output")
        // For standard CLIP export, it's often the second output, or we look by name.
        // Assuming typical index 1 is pooler_output (index 0 is last_hidden_state)
        // Let's try getting by name or index. Standard HF uses "pooler_output" or "text_embeds"

        let embedding = if let Some(vals) = outputs.get("pooler_output") {
            vals
        } else if let Some(vals) = outputs.get("text_embeds") {
            vals
        } else {
            // Fallback to second output if available, else first
            &outputs[0]
        };

        let (_, embedding_data) = embedding
            .try_extract_tensor::<f32>()
            .map_err(|e| format!("Failed to extract tensor: {}", e))?;

        // Return as Vec
        Ok(embedding_data.to_vec())
    }

    pub fn encode_image(&mut self, image_path: &str) -> Result<Vec<f32>, String> {
        if !self.is_loaded() {
            return Err("AI models not loaded".to_string());
        }

        let image_input = self.preprocess_image(image_path)?;
        let image_input_value = Value::from_array(image_input).map_err(|e| e.to_string())?;

        let outputs = self
            .vision_model
            .as_mut()
            .unwrap()
            .run(inputs![
                "pixel_values" => image_input_value,
            ])
            .map_err(|e| format!("Inference error: {}", e))?;

        // Vision model usually outputs "pooler_output" or "image_embeds"
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
        // resize to 224x224
        let img = img.resize_exact(224, 224, image::imageops::FilterType::Triangle);
        let rgb_img = img.to_rgb8();

        // Normalize
        // Mean: [0.48145466, 0.4578275, 0.40821073]
        // Std: [0.26862954, 0.26130258, 0.27577711]
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
