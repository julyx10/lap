use crate::t_image;
use exif::{In, Tag, Value as ExifValue};
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::io::Cursor;
use std::path::Path;
use std::time::Duration;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CivitaiSetting {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CivitaiModelResource {
    pub model_name: String,
    pub version_name: String,
    pub model_type: String,
    pub base_model: Option<String>,
    pub model_id: Option<i64>,
    pub model_version_id: Option<i64>,
    pub strength: Option<f64>,
    pub availability: String,
    pub matched_path: Option<String>,
    pub match_basis: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CivitaiImageReport {
    pub source_url: String,
    pub image_id: Option<i64>,
    pub image_url: Option<String>,
    pub prompt: Option<String>,
    pub negative_prompt: Option<String>,
    pub settings: Vec<CivitaiSetting>,
    pub resources: Vec<CivitaiModelResource>,
    pub warnings: Vec<String>,
}

pub async fn analyze_civitai_image_url(
    url: &str,
    model_roots: &[String],
) -> Result<CivitaiImageReport, String> {
    let source_url = url.trim();
    if source_url.is_empty() {
        return Err("Enter a Civitai image URL.".to_string());
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    if let Some(image_id) = extract_image_id_from_civitai_url(source_url) {
        let host = civitai_host(source_url);
        let page_url = format!("https://{}/images/{}", host, image_id);
        let page_result = fetch_text(&client, &page_url).await;

        if let Ok(html) = page_result {
            if let Some(page_json) = extract_next_data_json(&html) {
                let report =
                    build_report_from_page_json(source_url, Some(image_id), &page_json, model_roots)?;
                if report.prompt.is_some() || !report.resources.is_empty() {
                    return Ok(report);
                }
            }
        }

        let api_url = format!("https://{}/api/v1/images?imageId={}", host, image_id);
        let api_json = fetch_json(&client, &api_url).await?;
        let mut report =
            build_report_from_page_json(source_url, Some(image_id), &api_json, model_roots)?;
        if report.prompt.is_none() {
            report.warnings.push(
                "Civitai did not expose prompt metadata for this image through the public page/API."
                    .to_string(),
            );
        }
        return Ok(report);
    }

    let response = client
        .get(source_url)
        .header(USER_AGENT, user_agent())
        .header(ACCEPT, "image/*,text/html;q=0.8,*/*;q=0.5")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch URL: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!(
            "Server returned {} {}",
            status.as_u16(),
            status.canonical_reason().unwrap_or("")
        ));
    }

    let final_url = response.url().as_str().to_string();
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    if content_type.contains("text/html") {
        let html = String::from_utf8_lossy(&bytes);
        if let Some(page_json) = extract_next_data_json(&html) {
            return build_report_from_page_json(source_url, None, &page_json, model_roots);
        }
    }

    let metadata_texts = extract_generation_texts_from_image_bytes(&bytes);
    let mut report = metadata_texts
        .iter()
        .find_map(|text| {
            let report = build_report_from_generation_text(source_url, text, model_roots);
            if report.prompt.is_some() || !report.settings.is_empty() {
                Some(report)
            } else {
                None
            }
        })
        .unwrap_or_else(|| CivitaiImageReport {
            source_url: source_url.to_string(),
            image_id: None,
            image_url: Some(final_url.clone()),
            prompt: None,
            negative_prompt: None,
            settings: Vec::new(),
            resources: Vec::new(),
            warnings: vec![
                "No embedded generation metadata was found in the downloaded image.".to_string(),
            ],
        });

    report.image_url = Some(final_url);
    if report.resources.is_empty() {
        report.warnings.push(
            "Direct image URLs may not include Civitai model resource records; paste the /images/{id} page URL when available."
                .to_string(),
        );
    }
    Ok(report)
}

pub fn extract_image_id_from_civitai_url(url: &str) -> Option<i64> {
    if let Some(index) = url.find("/images/") {
        let rest = &url[index + "/images/".len()..];
        let digits: String = rest.chars().take_while(|ch| ch.is_ascii_digit()).collect();
        if !digits.is_empty() {
            return digits.parse().ok();
        }
    }

    for marker in ["imageId=", "imageid="] {
        if let Some(index) = url.find(marker) {
            let rest = &url[index + marker.len()..];
            let digits: String = rest.chars().take_while(|ch| ch.is_ascii_digit()).collect();
            if !digits.is_empty() {
                return digits.parse().ok();
            }
        }
    }

    None
}

pub fn build_report_from_page_json(
    source_url: &str,
    image_id: Option<i64>,
    page_json: &Value,
    model_roots: &[String],
) -> Result<CivitaiImageReport, String> {
    let mut warnings = Vec::new();
    let data = find_generation_data(page_json, image_id).unwrap_or(page_json);
    let meta = data.get("meta").unwrap_or(&Value::Null);
    let mut prompt = None;
    let mut negative_prompt = None;
    let mut settings = Vec::new();
    let mut resources = resources_from_value(data.get("resources").unwrap_or(&Value::Null));
    let image_url = data
        .get("url")
        .and_then(Value::as_str)
        .map(|value| value.to_string());

    if let Some(meta_obj) = meta.as_object() {
        prompt = string_field(meta, &["prompt", "Prompt"]);
        negative_prompt = string_field(meta, &["negativePrompt", "negative_prompt", "Negative prompt"]);
        settings = settings_from_meta(meta);

        if resources.is_empty() {
            resources = resources_from_meta(meta);
        }

        if prompt.is_none() {
            warnings.push("The Civitai record did not include a prompt field.".to_string());
        }

        if meta_obj.is_empty() {
            warnings.push("The Civitai metadata object was empty.".to_string());
        }
    } else if let Some(meta_text) = meta.as_str() {
        let report = build_report_from_generation_text(source_url, meta_text, model_roots);
        prompt = report.prompt;
        negative_prompt = report.negative_prompt;
        settings = report.settings;
        resources = report.resources;
        warnings.extend(report.warnings);
    } else {
        warnings.push("No structured generation metadata was found.".to_string());
    }

    if resources.is_empty() {
        resources = resources_from_model_version_ids(data.get("modelVersionIds").unwrap_or(&Value::Null));
    }

    warnings.extend(apply_model_availability(&mut resources, model_roots));

    Ok(CivitaiImageReport {
        source_url: source_url.to_string(),
        image_id,
        image_url,
        prompt,
        negative_prompt,
        settings,
        resources,
        warnings,
    })
}

pub fn build_report_from_generation_text(
    source_url: &str,
    generation_text: &str,
    model_roots: &[String],
) -> CivitaiImageReport {
    if let Ok(json) = serde_json::from_str::<Value>(generation_text) {
        if let Ok(report) = build_report_from_page_json(source_url, None, &json, model_roots) {
            if report.prompt.is_some() || !report.settings.is_empty() {
                return report;
            }
        }
    }

    let (prompt, negative_prompt, settings) = parse_a1111_generation_text(generation_text);
    let mut resources = resources_from_generation_text(&prompt, &settings);
    let mut warnings = apply_model_availability(&mut resources, model_roots);

    if prompt.trim().is_empty() && settings.is_empty() {
        warnings.push("The embedded metadata did not match a known prompt format.".to_string());
    }

    CivitaiImageReport {
        source_url: source_url.to_string(),
        image_id: None,
        image_url: None,
        prompt: non_empty(prompt),
        negative_prompt,
        settings,
        resources,
        warnings,
    }
}

fn user_agent() -> &'static str {
    "Lap Civitai metadata importer/0.1"
}

fn civitai_host(url: &str) -> &'static str {
    if url.contains("civitai.red") || url.contains("civitaired.com") {
        "civitai.red"
    } else {
        "civitai.com"
    }
}

async fn fetch_text(client: &reqwest::Client, url: &str) -> Result<String, String> {
    let response = client
        .get(url)
        .header(USER_AGENT, user_agent())
        .header(ACCEPT, "text/html,application/json;q=0.8,*/*;q=0.5")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch Civitai page: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!(
            "Civitai returned {} {}",
            status.as_u16(),
            status.canonical_reason().unwrap_or("")
        ));
    }

    response
        .text()
        .await
        .map_err(|e| format!("Failed to read Civitai page: {}", e))
}

async fn fetch_json(client: &reqwest::Client, url: &str) -> Result<Value, String> {
    let response = client
        .get(url)
        .header(USER_AGENT, user_agent())
        .header(ACCEPT, "application/json")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch Civitai API: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!(
            "Civitai API returned {} {}",
            status.as_u16(),
            status.canonical_reason().unwrap_or("")
        ));
    }

    response
        .json::<Value>()
        .await
        .map_err(|e| format!("Failed to parse Civitai API JSON: {}", e))
}

fn extract_next_data_json(html: &str) -> Option<Value> {
    let marker = r#"<script id="__NEXT_DATA__" type="application/json">"#;
    let start = html.find(marker)? + marker.len();
    let rest = &html[start..];
    let end = rest.find("</script>")?;
    serde_json::from_str(&rest[..end]).ok()
}

fn find_generation_data(value: &Value, image_id: Option<i64>) -> Option<&Value> {
    let mut matches = Vec::new();
    collect_generation_data(value, &mut matches);

    if let Some(target_id) = image_id {
        if let Some(found) = matches
            .iter()
            .copied()
            .find(|candidate| value_has_image_id(candidate, target_id))
        {
            return Some(found);
        }
    }

    matches.into_iter().next()
}

fn collect_generation_data<'a>(value: &'a Value, matches: &mut Vec<&'a Value>) {
    match value {
        Value::Object(map) => {
            let has_generation_meta = map
                .get("meta")
                .map(|meta| {
                    meta.is_object()
                        || meta.as_str().map(|text| !text.trim().is_empty()).unwrap_or(false)
                })
                .unwrap_or(false);
            let has_resource_data = map
                .get("resources")
                .and_then(Value::as_array)
                .map(|items| !items.is_empty())
                .unwrap_or(false);
            let has_model_versions = map
                .get("modelVersionIds")
                .and_then(Value::as_array)
                .map(|items| !items.is_empty())
                .unwrap_or(false);

            if has_generation_meta || has_resource_data || has_model_versions {
                matches.push(value);
            }

            for child in map.values() {
                collect_generation_data(child, matches);
            }
        }
        Value::Array(items) => {
            for child in items {
                collect_generation_data(child, matches);
            }
        }
        _ => {}
    }
}

fn value_has_image_id(value: &Value, image_id: i64) -> bool {
    match value {
        Value::Object(map) => {
            for key in ["id", "imageId"] {
                if map.get(key).and_then(Value::as_i64) == Some(image_id) {
                    return true;
                }
            }
            map.values().any(|child| value_has_image_id(child, image_id))
        }
        Value::Array(items) => items.iter().any(|child| value_has_image_id(child, image_id)),
        _ => false,
    }
}

fn string_field(value: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(text) = value.get(*key).and_then(Value::as_str) {
            if let Some(text) = non_empty(text.to_string()) {
                return Some(text);
            }
        }
    }
    None
}

fn settings_from_meta(meta: &Value) -> Vec<CivitaiSetting> {
    let mut settings = Vec::new();
    let Some(map) = meta.as_object() else {
        return settings;
    };

    for (key, value) in map {
        if matches!(
            key.as_str(),
            "prompt" | "Prompt" | "negativePrompt" | "negative_prompt" | "Negative prompt"
        ) {
            continue;
        }

        if key == "hashes" {
            if let Some(hashes) = value.as_object() {
                for (hash_key, hash_value) in hashes {
                    if let Some(hash) = value_to_report_string(hash_value) {
                        settings.push(CivitaiSetting {
                            key: format!("Hash: {}", hash_key),
                            value: hash,
                        });
                    }
                }
            }
            continue;
        }

        if let Some(value) = value_to_report_string(value) {
            settings.push(CivitaiSetting {
                key: display_setting_key(key),
                value,
            });
        }
    }

    settings
}

fn value_to_report_string(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => non_empty(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn display_setting_key(key: &str) -> String {
    match key {
        "cfgScale" => "CFG scale".to_string(),
        "steps" => "Steps".to_string(),
        "sampler" => "Sampler".to_string(),
        "seed" => "Seed".to_string(),
        other => {
            let mut chars = other.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        }
    }
}

fn resources_from_value(value: &Value) -> Vec<CivitaiModelResource> {
    let Some(items) = value.as_array() else {
        return Vec::new();
    };

    items
        .iter()
        .filter_map(|item| {
            let model_name = item
                .get("modelName")
                .or_else(|| item.get("name"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim()
                .to_string();
            let version_name = item
                .get("versionName")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim()
                .to_string();
            let model_type = item
                .get("modelType")
                .or_else(|| item.get("type"))
                .and_then(Value::as_str)
                .unwrap_or("Model")
                .trim()
                .to_string();

            if model_name.is_empty() && version_name.is_empty() {
                return None;
            }

            Some(CivitaiModelResource {
                model_name,
                version_name,
                model_type,
                base_model: item
                    .get("baseModel")
                    .and_then(Value::as_str)
                    .map(|value| value.to_string()),
                model_id: item.get("modelId").and_then(Value::as_i64),
                model_version_id: item
                    .get("modelVersionId")
                    .or_else(|| item.get("versionId"))
                    .and_then(Value::as_i64),
                strength: item.get("strength").and_then(Value::as_f64),
                availability: "unchecked".to_string(),
                matched_path: None,
                match_basis: None,
            })
        })
        .collect()
}

fn resources_from_model_version_ids(value: &Value) -> Vec<CivitaiModelResource> {
    let Some(ids) = value.as_array() else {
        return Vec::new();
    };

    ids.iter()
        .filter_map(Value::as_i64)
        .map(|id| CivitaiModelResource {
            model_name: format!("Model version {}", id),
            version_name: String::new(),
            model_type: "Model".to_string(),
            base_model: None,
            model_id: None,
            model_version_id: Some(id),
            strength: None,
            availability: "unchecked".to_string(),
            matched_path: None,
            match_basis: None,
        })
        .collect()
}

fn resources_from_meta(meta: &Value) -> Vec<CivitaiModelResource> {
    let mut resources = Vec::new();
    if let Some(model) = string_field(meta, &["Model", "model"]) {
        resources.push(CivitaiModelResource {
            model_name: model,
            version_name: String::new(),
            model_type: "Checkpoint".to_string(),
            base_model: None,
            model_id: None,
            model_version_id: None,
            strength: None,
            availability: "unchecked".to_string(),
            matched_path: None,
            match_basis: None,
        });
    }

    if let Some(prompt) = string_field(meta, &["prompt", "Prompt"]) {
        resources.extend(lora_resources_from_prompt(&prompt));
    }

    resources
}

fn parse_a1111_generation_text(text: &str) -> (String, Option<String>, Vec<CivitaiSetting>) {
    let mut prompt_lines = Vec::new();
    let mut negative_lines = Vec::new();
    let mut setting_lines = Vec::new();
    let mut phase = "prompt";

    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Negative prompt:") {
            phase = "negative";
            if !rest.trim().is_empty() {
                negative_lines.push(rest.trim().to_string());
            }
            continue;
        }

        if looks_like_settings_line(trimmed) {
            phase = "settings";
        }

        match phase {
            "prompt" => prompt_lines.push(line.trim_end().to_string()),
            "negative" => negative_lines.push(line.trim_end().to_string()),
            _ => setting_lines.push(trimmed.to_string()),
        }
    }

    let prompt = prompt_lines.join("\n").trim().to_string();
    let negative_prompt = non_empty(negative_lines.join("\n").trim().to_string());
    let settings = parse_settings_lines(&setting_lines.join(", "));

    (prompt, negative_prompt, settings)
}

fn looks_like_settings_line(line: &str) -> bool {
    line.starts_with("Steps:")
        || (line.contains("Sampler:") && line.contains("Seed:"))
        || (line.contains("CFG scale:") && line.contains("Size:"))
}

fn parse_settings_lines(line: &str) -> Vec<CivitaiSetting> {
    line.split(", ")
        .filter_map(|part| {
            let (key, value) = part.split_once(':')?;
            let key = key.trim();
            let value = value.trim();
            if key.is_empty() || value.is_empty() {
                return None;
            }
            Some(CivitaiSetting {
                key: key.to_string(),
                value: value.to_string(),
            })
        })
        .collect()
}

fn resources_from_generation_text(
    prompt: &str,
    settings: &[CivitaiSetting],
) -> Vec<CivitaiModelResource> {
    let mut resources = Vec::new();
    if let Some(model) = settings
        .iter()
        .find(|item| item.key.eq_ignore_ascii_case("Model"))
        .map(|item| item.value.clone())
    {
        resources.push(CivitaiModelResource {
            model_name: model,
            version_name: String::new(),
            model_type: "Checkpoint".to_string(),
            base_model: None,
            model_id: None,
            model_version_id: None,
            strength: None,
            availability: "unchecked".to_string(),
            matched_path: None,
            match_basis: None,
        });
    }

    resources.extend(lora_resources_from_prompt(prompt));
    dedupe_resources(resources)
}

fn lora_resources_from_prompt(prompt: &str) -> Vec<CivitaiModelResource> {
    let mut resources = Vec::new();
    let mut rest = prompt;

    while let Some(start) = rest.find("<lora:") {
        let after = &rest[start + "<lora:".len()..];
        let Some(end) = after.find('>') else {
            break;
        };
        let tag = &after[..end];
        let mut parts = tag.rsplitn(2, ':');
        let maybe_strength = parts.next().unwrap_or("").trim();
        let maybe_name = parts.next().unwrap_or(tag).trim();
        let (name, strength) = if let Ok(strength) = maybe_strength.parse::<f64>() {
            (maybe_name, Some(strength))
        } else {
            (tag.trim(), None)
        };

        if !name.is_empty() {
            resources.push(CivitaiModelResource {
                model_name: name.to_string(),
                version_name: String::new(),
                model_type: "LORA".to_string(),
                base_model: None,
                model_id: None,
                model_version_id: None,
                strength,
                availability: "unchecked".to_string(),
                matched_path: None,
                match_basis: None,
            });
        }

        rest = &after[end + 1..];
    }

    resources
}

fn dedupe_resources(resources: Vec<CivitaiModelResource>) -> Vec<CivitaiModelResource> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();
    for resource in resources {
        let key = format!(
            "{}:{}:{}",
            normalize_match_text(&resource.model_type),
            normalize_match_text(&resource.model_name),
            normalize_match_text(&resource.version_name)
        );
        if seen.insert(key) {
            deduped.push(resource);
        }
    }
    deduped
}

fn apply_model_availability(
    resources: &mut [CivitaiModelResource],
    model_roots: &[String],
) -> Vec<String> {
    let roots: Vec<String> = model_roots
        .iter()
        .map(|root| root.trim())
        .filter(|root| !root.is_empty())
        .map(|root| root.to_string())
        .collect();

    if roots.is_empty() {
        for resource in resources {
            resource.availability = "unchecked".to_string();
        }
        return Vec::new();
    }

    let (inventory, warnings) = scan_model_inventory(&roots);
    for resource in resources {
        if let Some(path) = find_matching_model_path(resource, &inventory) {
            resource.availability = "found".to_string();
            resource.matched_path = Some(path);
            resource.match_basis = Some("file name".to_string());
        } else {
            resource.availability = "missing".to_string();
        }
    }

    warnings
}

fn scan_model_inventory(model_roots: &[String]) -> (Vec<(String, String)>, Vec<String>) {
    let mut inventory = Vec::new();
    let mut warnings = Vec::new();

    for root in model_roots {
        let path = Path::new(root);
        if !path.exists() {
            warnings.push(format!("Model folder not found: {}", root));
            continue;
        }

        if !path.is_dir() {
            warnings.push(format!("Model path is not a folder: {}", root));
            continue;
        }

        for entry in WalkDir::new(path).follow_links(true).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            let Some(ext) = path.extension().and_then(|value| value.to_str()) else {
                continue;
            };

            if !is_model_extension(ext) {
                continue;
            }

            let file_name = path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("")
                .to_string();
            inventory.push((
                normalize_match_text(&file_name),
                path.to_string_lossy().to_string(),
            ));
        }
    }

    (inventory, warnings)
}

fn is_model_extension(ext: &str) -> bool {
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "safetensors" | "ckpt" | "pt" | "pth" | "bin"
    )
}

fn find_matching_model_path(
    resource: &CivitaiModelResource,
    inventory: &[(String, String)],
) -> Option<String> {
    let candidates = [
        normalize_match_text(&resource.model_name),
        normalize_match_text(&resource.version_name),
    ];

    inventory
        .iter()
        .find(|(file_name, _)| {
            candidates.iter().any(|candidate| {
                candidate.len() >= 4
                    && (file_name.contains(candidate) || candidate.contains(file_name))
            })
        })
        .map(|(_, path)| path.clone())
}

fn normalize_match_text(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn extract_generation_texts_from_image_bytes(bytes: &[u8]) -> Vec<String> {
    let mut texts = Vec::new();

    if let Some(exif) = t_image::read_exif_from_bytes_permissive(bytes) {
        for tag in [Tag::ImageDescription, Tag::UserComment, Tag::Software] {
            if let Some(text) = exif_text_field(&exif, tag) {
                texts.push(text);
            }
        }
    }

    texts.extend(png_text_chunks(bytes));
    dedupe_texts(texts)
}

fn exif_text_field(exif: &exif::Exif, tag: Tag) -> Option<String> {
    let field = exif
        .get_field(tag, In::PRIMARY)
        .or_else(|| exif.fields().find(|field| field.tag == tag))?;

    match &field.value {
        ExifValue::Ascii(values) => {
            let mut bytes = Vec::new();
            for value in values {
                bytes.extend(value.iter().copied().take_while(|byte| *byte != 0));
            }
            non_empty(String::from_utf8_lossy(&bytes).trim().to_string())
        }
        ExifValue::Undefined(bytes, _) => {
            let cleaned = strip_user_comment_prefix(bytes);
            non_empty(String::from_utf8_lossy(cleaned).trim_matches('\0').trim().to_string())
        }
        _ => non_empty(field.display_value().with_unit(exif).to_string()),
    }
}

fn strip_user_comment_prefix(bytes: &[u8]) -> &[u8] {
    for prefix in [b"ASCII\0\0\0".as_slice(), b"UNICODE\0".as_slice(), b"JIS\0\0\0\0\0".as_slice()] {
        if bytes.starts_with(prefix) {
            return &bytes[prefix.len()..];
        }
    }
    bytes
}

fn png_text_chunks(bytes: &[u8]) -> Vec<String> {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if !bytes.starts_with(PNG_SIGNATURE) {
        return Vec::new();
    }

    let mut cursor = Cursor::new(&bytes[8..]);
    let mut texts = Vec::new();

    loop {
        let position = cursor.position() as usize;
        let remaining = &bytes[8 + position..];
        if remaining.len() < 12 {
            break;
        }

        let length = u32::from_be_bytes([
            remaining[0],
            remaining[1],
            remaining[2],
            remaining[3],
        ]) as usize;
        if remaining.len() < 12 + length {
            break;
        }

        let chunk_type = &remaining[4..8];
        let data = &remaining[8..8 + length];
        match chunk_type {
            b"tEXt" => {
                if let Some((_keyword, text)) = split_once_byte(data, 0) {
                    if let Some(text) = non_empty(String::from_utf8_lossy(text).to_string()) {
                        texts.push(text);
                    }
                }
            }
            b"iTXt" => {
                if let Some(text) = parse_itxt_chunk(data) {
                    texts.push(text);
                }
            }
            _ => {}
        }

        cursor.set_position((position + 12 + length) as u64);
    }

    texts
}

fn parse_itxt_chunk(data: &[u8]) -> Option<String> {
    let (_keyword, rest) = split_once_byte(data, 0)?;
    if rest.len() < 2 {
        return None;
    }
    let compression_flag = rest[0];
    if compression_flag != 0 {
        return None;
    }
    let rest = &rest[2..];
    let (_language, rest) = split_once_byte(rest, 0)?;
    let (_translated, text) = split_once_byte(rest, 0)?;
    non_empty(String::from_utf8_lossy(text).to_string())
}

fn split_once_byte(data: &[u8], byte: u8) -> Option<(&[u8], &[u8])> {
    let index = data.iter().position(|item| *item == byte)?;
    Some((&data[..index], &data[index + 1..]))
}

fn dedupe_texts(texts: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();
    for text in texts {
        if text.trim().is_empty() {
            continue;
        }
        if seen.insert(text.clone()) {
            deduped.push(text);
        }
    }
    deduped
}

fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_image_id_from_civitai_page_urls() {
        assert_eq!(
            extract_image_id_from_civitai_url("https://civitai.red/images/12097475?foo=bar"),
            Some(12097475)
        );
        assert_eq!(
            extract_image_id_from_civitai_url("https://civitai.com/posts/2669960?imageId=12097475"),
            Some(12097475)
        );
    }

    #[test]
    fn builds_report_from_embedded_civitai_page_data() {
        let page_json = json!({
            "props": {
                "pageProps": {
                    "trpcState": {
                        "json": {
                            "queries": [
                                {
                                    "state": {
                                        "data": {
                                            "type": "image",
                                            "process": "txt2img",
                                            "meta": {
                                                "prompt": "score_9, forest, <lora:pony/kenva:0.8>",
                                                "negativePrompt": "score_6, blurry",
                                                "cfgScale": 7,
                                                "steps": 25,
                                                "sampler": "Euler a Karras",
                                                "seed": 3269595310u64,
                                                "Size": "896x1152",
                                                "hashes": {
                                                    "model": "67ab2fd8ec",
                                                    "LORA:pony/kenva": "189804f733"
                                                }
                                            },
                                            "resources": [
                                                {
                                                    "imageId": 12097475,
                                                    "modelVersionId": 290640,
                                                    "modelId": 257749,
                                                    "modelName": "Pony Diffusion V6 XL",
                                                    "modelType": "Checkpoint",
                                                    "versionName": "V6 (start with this one)",
                                                    "baseModel": "Pony"
                                                },
                                                {
                                                    "imageId": 12097475,
                                                    "modelVersionId": 330475,
                                                    "modelId": 264290,
                                                    "modelName": "Not Artists Styles for Pony Diffusion V6 XL",
                                                    "modelType": "LORA",
                                                    "versionName": "Concept Art Twilight [M]",
                                                    "baseModel": "Pony",
                                                    "strength": 0.8
                                                }
                                            ]
                                        }
                                    }
                                }
                            ]
                        }
                    }
                }
            }
        });

        let report = build_report_from_page_json(
            "https://civitai.red/images/12097475",
            Some(12097475),
            &page_json,
            &[],
        )
        .expect("page JSON should produce a report");

        assert_eq!(report.prompt.as_deref(), Some("score_9, forest, <lora:pony/kenva:0.8>"));
        assert_eq!(report.negative_prompt.as_deref(), Some("score_6, blurry"));
        assert!(report.settings.iter().any(|item| item.key == "Steps" && item.value == "25"));
        assert!(report.settings.iter().any(|item| item.key == "Sampler" && item.value == "Euler a Karras"));
        assert_eq!(report.resources.len(), 2);
        assert_eq!(report.resources[0].model_name, "Pony Diffusion V6 XL");
        assert_eq!(report.resources[0].availability, "unchecked");
    }

    #[test]
    fn parses_generation_text_into_prompt_negative_prompt_and_settings() {
        let report = build_report_from_generation_text(
            "https://image.civitai.com/example.jpeg",
            "a cat in a window\nNegative prompt: blurry, low quality\nSteps: 30, Sampler: DPM++ 2M Karras, CFG scale: 7, Seed: 1234, Size: 512x768, Model: ponyDiffusionV6XL",
            &[],
        );

        assert_eq!(report.prompt.as_deref(), Some("a cat in a window"));
        assert_eq!(report.negative_prompt.as_deref(), Some("blurry, low quality"));
        assert!(report.settings.iter().any(|item| item.key == "CFG scale" && item.value == "7"));
        assert!(report.settings.iter().any(|item| item.key == "Model" && item.value == "ponyDiffusionV6XL"));
    }

    #[test]
    fn marks_resource_found_when_model_root_contains_matching_file_name() {
        let temp_root = std::env::temp_dir().join(format!(
            "lap-civitai-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&temp_root).unwrap();
        let model_path = temp_root.join("ponyDiffusionV6XL_v6StartWithThisOne.safetensors");
        std::fs::write(&model_path, b"placeholder").unwrap();

        let page_json = json!({
            "state": {
                "data": {
                    "meta": { "prompt": "x" },
                    "resources": [
                        {
                            "modelName": "Pony Diffusion V6 XL",
                            "modelType": "Checkpoint",
                            "versionName": "V6 (start with this one)"
                        }
                    ]
                }
            }
        });

        let report = build_report_from_page_json(
            "https://civitai.red/images/1",
            Some(1),
            &page_json,
            &[temp_root.to_string_lossy().to_string()],
        )
        .unwrap();

        assert_eq!(report.resources[0].availability, "found");
        assert_eq!(
            report.resources[0].matched_path.as_deref(),
            Some(model_path.to_string_lossy().as_ref())
        );

        std::fs::remove_file(&model_path).unwrap();
        std::fs::remove_dir_all(&temp_root).unwrap();
    }
}
