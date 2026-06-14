use reqwest::header::{ACCEPT, CONTENT_TYPE, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

const DEFAULT_REVERSE_PROMPT_ENDPOINT: &str = "https://api.openai.com/v1/responses";
const DEFAULT_REVERSE_PROMPT_MODEL: &str = "gpt-5.5";
const DEFAULT_REVERSE_PROMPT_INSTRUCTION: &str = "Describe this image as one reusable image-generation prompt. Include visible subject, environment, composition, lighting, color palette, style, mood, and camera or medium. Return one prompt only.";
const MAX_IMAGE_DATA_URL_LEN: usize = 20_971_520;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReversePromptRequest {
    api_key: String,
    image_data_url: String,
    model: Option<String>,
    endpoint: Option<String>,
    detail: Option<String>,
    instruction: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReversePromptResult {
    prompt: String,
}

#[tauri::command]
pub async fn reverse_prompt_image(
    request: ReversePromptRequest,
) -> Result<ReversePromptResult, String> {
    let api_key = request.api_key.trim();
    if api_key.is_empty() {
        return Err("API key is required.".to_string());
    }

    let image_data_url = request.image_data_url.trim();
    validate_image_data_url(image_data_url)?;

    let endpoint = request
        .endpoint
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_REVERSE_PROMPT_ENDPOINT);
    let model = request
        .model
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_REVERSE_PROMPT_MODEL);
    let instruction = request
        .instruction
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_REVERSE_PROMPT_INSTRUCTION);
    let detail = normalize_detail(request.detail.as_deref())?;

    let body = serde_json::json!({
        "model": model,
        "input": [
            {
                "role": "user",
                "content": [
                    { "type": "input_text", "text": instruction },
                    {
                        "type": "input_image",
                        "image_url": image_data_url,
                        "detail": detail,
                    }
                ]
            }
        ]
    });

    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(90))
        .build()
        .map_err(|e| format!("Failed to create vision client: {}", e))?;

    let response = client
        .post(endpoint)
        .bearer_auth(api_key)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .header(USER_AGENT, "Lap Reverse Prompter")
        .body(body.to_string())
        .send()
        .await
        .map_err(|e| format!("Vision request failed: {}", e))?;

    let status = response.status();
    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read vision response: {}", e))?;
    let value: Value = serde_json::from_str(&response_text).map_err(|e| {
        format!(
            "Vision API returned non-JSON response ({}): {}",
            status.as_u16(),
            truncate_response(&format!("{}: {}", e, response_text))
        )
    })?;

    if !status.is_success() {
        let message = extract_api_error_message(&value)
            .unwrap_or_else(|| format!("Vision API returned HTTP {}", status.as_u16()));
        return Err(message);
    }

    Ok(ReversePromptResult {
        prompt: extract_openai_output_text(&value)?,
    })
}

fn validate_image_data_url(image_data_url: &str) -> Result<(), String> {
    if image_data_url.len() > MAX_IMAGE_DATA_URL_LEN {
        return Err("Image is too large for the vision API request.".to_string());
    }

    if !image_data_url.starts_with("data:image/") || !image_data_url.contains(";base64,") {
        return Err("Image must be sent as a base64 image data URL.".to_string());
    }

    Ok(())
}

fn normalize_detail(detail: Option<&str>) -> Result<&'static str, String> {
    match detail.unwrap_or("high").trim().to_ascii_lowercase().as_str() {
        "" | "high" => Ok("high"),
        "low" => Ok("low"),
        "auto" => Ok("auto"),
        "original" => Ok("original"),
        other => Err(format!("Unsupported image detail level: {}", other)),
    }
}

pub(crate) fn extract_openai_output_text(value: &Value) -> Result<String, String> {
    if let Some(message) = extract_api_error_message(value) {
        return Err(message);
    }

    if let Some(text) = value.get("output_text").and_then(Value::as_str) {
        if let Some(cleaned) = clean_output_text(text) {
            return Ok(cleaned);
        }
    }

    if let Some(output) = value.get("output").and_then(Value::as_array) {
        for item in output {
            if let Some(content) = item.get("content").and_then(Value::as_array) {
                for block in content {
                    if let Some(text) = block.get("text").and_then(Value::as_str) {
                        if let Some(cleaned) = clean_output_text(text) {
                            return Ok(cleaned);
                        }
                    }
                }
            }
        }
    }

    if let Some(choices) = value.get("choices").and_then(Value::as_array) {
        for choice in choices {
            if let Some(text) = choice
                .get("message")
                .and_then(|message| message.get("content"))
                .and_then(Value::as_str)
            {
                if let Some(cleaned) = clean_output_text(text) {
                    return Ok(cleaned);
                }
            }
        }
    }

    Err("Vision response did not include prompt text.".to_string())
}

fn extract_api_error_message(value: &Value) -> Option<String> {
    value
        .get("error")
        .and_then(|error| error.get("message"))
        .and_then(Value::as_str)
        .and_then(clean_output_text)
}

fn clean_output_text(text: &str) -> Option<String> {
    let cleaned = text.trim().trim_matches('"').trim();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.to_string())
    }
}

fn truncate_response(text: &str) -> String {
    const MAX_LEN: usize = 240;
    if text.len() <= MAX_LEN {
        return text.to_string();
    }
    format!("{}...", &text[..MAX_LEN])
}

#[cfg(test)]
mod tests {
    use super::extract_openai_output_text;

    #[test]
    fn extracts_output_text_from_responses_api() {
        let value = serde_json::json!({
            "output": [
                {
                    "type": "message",
                    "content": [
                        { "type": "output_text", "text": "cinematic portrait prompt" }
                    ]
                }
            ]
        });

        assert_eq!(
            extract_openai_output_text(&value).unwrap(),
            "cinematic portrait prompt"
        );
    }

    #[test]
    fn returns_api_error_message() {
        let value = serde_json::json!({
            "error": { "message": "invalid api key" }
        });

        let error = extract_openai_output_text(&value).unwrap_err();
        assert!(error.contains("invalid api key"));
    }
}
