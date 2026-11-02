use std::time::Duration;

use super::ai_settings::{is_valid_model_id, AiSettings};
use super::models::{AppError, ServiceInfo};

const API_BASE: &str = "https://api.x.ai/v1";

fn http_client(timeout_secs: u64) -> Result<reqwest::Client, AppError> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| AppError::AiRequest { message: format!("Could not create HTTP client: {}", e) })
}

/// Turn a non-success HTTP status into a user-facing error. The response body
/// is only included (truncated) for non-auth errors, so a key echoed back by
/// the API never reaches the UI or logs.
pub(crate) fn map_http_error(status: u16, body: &str, model: &str) -> AppError {
    match status {
        401 | 403 => AppError::AiAuth {
            message: "xAI rejected the API key, check it in Settings".into(),
        },
        404 => AppError::AiRequest {
            message: format!("Model \"{}\" was not found or is not available for this key", model),
        },
        429 => AppError::AiRequest {
            message: "xAI rate limit or credit limit reached, try again later".into(),
        },
        _ => {
            let snippet: String = body.chars().take(300).collect();
            AppError::AiRequest { message: format!("xAI API error {}: {}", status, snippet.trim()) }
        }
    }
}

fn map_send_error(e: reqwest::Error, timeout_secs: u64) -> AppError {
    if e.is_timeout() {
        AppError::AiRequest { message: format!("xAI request timed out after {} seconds", timeout_secs) }
    } else if e.is_connect() {
        AppError::AiRequest { message: "Could not connect to api.x.ai, check your internet connection".into() }
    } else {
        // reqwest errors don't contain request headers, so the key can't leak here
        AppError::AiRequest { message: format!("xAI request failed: {}", e) }
    }
}

fn build_prompt(service_name: &str) -> String {
    format!("What is the Windows service \"{}\"?\n\nPlease provide a JSON response with exactly these three keys:\n- \"description\": A brief description of what this service does\n- \"explained\": A concise explanation in 2-3 lines of its purpose and functionality\n- \"recommendation\": A bullet-point list covering whether to disable it, what would be affected, and safe disabling scenarios\n\nExample format:\n{{\n  \"description\": \"Brief description here\",\n  \"explained\": \"Concise explanation here\",\n  \"recommendation\": \"• Point 1\\n• Point 2\\n• Point 3\"\n}}\n\nReturn only valid JSON, no additional text.", service_name)
}

/// Chat Completions request body. Reasoning tokens count against
/// `max_tokens`, so for grok-4.3 (configurable reasoning) reasoning is turned
/// off: this is a short factual lookup.
pub(crate) fn build_request_body(model: &str, prompt: &str, max_tokens: u32) -> serde_json::Value {
    let mut body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": max_tokens,
        "temperature": 0.7,
        "stream": false
    });
    if model.starts_with("grok-4.3") {
        body["reasoning_effort"] = serde_json::json!("none");
    }
    body
}

pub async fn fetch_service_info_from_ai(
    service_name: &str,
    api_key: &str,
    settings: &AiSettings,
) -> Result<ServiceInfo, AppError> {
    let client = http_client(settings.timeout_secs)?;
    let response = client
        .post(format!("{}/chat/completions", API_BASE))
        .bearer_auth(api_key)
        .json(&build_request_body(&settings.model, &build_prompt(service_name), settings.max_tokens))
        .send()
        .await
        .map_err(|e| map_send_error(e, settings.timeout_secs))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(map_http_error(status.as_u16(), &body, &settings.model));
    }

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::AiRequest { message: format!("Failed to parse xAI response: {}", e) })?;

    let content = data["choices"][0]["message"]["content"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or(AppError::AiRequest {
            message: "xAI returned an empty answer, try a higher max tokens value".to_string(),
        })?;

    Ok(parse_service_info(content))
}

/// Check that the key is accepted and the model is available. Uses
/// `GET /v1/models/{model}`, which costs no tokens.
pub async fn test_connection(api_key: &str, model: &str, timeout_secs: u64) -> Result<String, AppError> {
    if !is_valid_model_id(model) {
        return Err(AppError::Validation { message: "Invalid model name".into() });
    }
    let client = http_client(timeout_secs)?;
    let response = client
        .get(format!("{}/models/{}", API_BASE, model))
        .bearer_auth(api_key)
        .send()
        .await
        .map_err(|e| map_send_error(e, timeout_secs))?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(map_http_error(status.as_u16(), &body, model));
    }
    Ok(format!("Connected, model {} is available", model))
}

/// Strip optional ```json fences that models sometimes add.
fn strip_code_fence(s: &str) -> &str {
    let t = s.trim();
    let Some(rest) = t.strip_prefix("```") else { return t };
    let rest = rest.strip_prefix("json").unwrap_or(rest);
    rest.strip_suffix("```").unwrap_or(rest).trim()
}

pub(crate) fn parse_service_info(ai_response: &str) -> ServiceInfo {
    let cleaned = strip_code_fence(ai_response);
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(cleaned) {
        ServiceInfo {
            description: parsed["description"].as_str().map(|s| s.to_string()),
            explained: parsed["explained"].as_str().map(|s| s.to_string()),
            recommendation: parsed["recommendation"].as_str().map(|s| s.to_string()),
        }
    } else {
        // Fallback: extract information from text response
        let description = extract_field_from_text(cleaned, "description");
        let explained = extract_field_from_text(cleaned, "explained");
        let recommendation = extract_field_from_text(cleaned, "recommendation");

        ServiceInfo {
            description: description.or_else(|| Some("AI-generated description".to_string())),
            explained: explained.or_else(|| Some("AI-generated explanation".to_string())),
            recommendation: recommendation.or_else(|| Some("AI-generated recommendation".to_string())),
        }
    }
}

fn extract_field_from_text(text: &str, field: &str) -> Option<String> {
    let patterns = [
        format!("\"{}\": \"", field),
        format!("{}: ", field),
        format!("{}\n", field),
    ];

    for pattern in &patterns {
        if let Some(start) = text.find(pattern) {
            let start_pos = start + pattern.len();
            let remaining = &text[start_pos..];

            let end_pos = remaining
                .find('"')
                .or_else(|| remaining.find('\n'))
                .unwrap_or(remaining.len());

            let value = remaining[..end_pos].trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_and_fenced_json() {
        let json = r#"{"description":"d","explained":"e","recommendation":"r"}"#;
        let a = parse_service_info(json);
        assert_eq!(a.description.as_deref(), Some("d"));
        let fenced = format!("```json\n{}\n```", json);
        let b = parse_service_info(&fenced);
        assert_eq!(b.explained.as_deref(), Some("e"));
        assert_eq!(b.recommendation.as_deref(), Some("r"));
    }

    #[test]
    fn falls_back_to_text_extraction() {
        let info = parse_service_info("description: Print spooler\nexplained: Handles printing\n");
        assert_eq!(info.description.as_deref(), Some("Print spooler"));
        assert_eq!(info.explained.as_deref(), Some("Handles printing"));
        assert!(info.recommendation.is_some());
    }

    #[test]
    fn maps_http_errors_without_leaking_body_on_auth() {
        match map_http_error(401, "Incorrect API key provided: xai-SECRET", "grok-4.3") {
            AppError::AiAuth { message } => assert!(!message.contains("SECRET")),
            other => panic!("unexpected {:?}", other),
        }
        assert!(matches!(map_http_error(403, "", "m"), AppError::AiAuth { .. }));
        match map_http_error(404, "", "grok-x") {
            AppError::AiRequest { message } => assert!(message.contains("grok-x")),
            other => panic!("unexpected {:?}", other),
        }
        let long = "x".repeat(1000);
        match map_http_error(500, &long, "m") {
            AppError::AiRequest { message } => assert!(message.len() < 400),
            other => panic!("unexpected {:?}", other),
        }
    }

    #[test]
    fn request_body_disables_reasoning_only_for_grok_43() {
        let b = build_request_body("grok-4.3", "p", 500);
        assert_eq!(b["reasoning_effort"], "none");
        assert_eq!(b["max_tokens"], 500);
        let c = build_request_body("grok-4.7", "p", 500);
        assert!(c.get("reasoning_effort").is_none());
    }
}
