use std::time::Duration;

use tokio::time::timeout;

use crate::models::ServiceInfo;

pub async fn fetch_service_info_from_ai(service_name: &str) -> Result<ServiceInfo, String> {
    let api_key = std::env::var("GROK_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return Err("Grok API key not configured. Please set GROK_API_KEY in your .env file.".to_string());
    }

    let timeout_secs: u64 = std::env::var("GROK_API_TIMEOUT")
        .unwrap_or_else(|_| "15".to_string())
        .parse()
        .unwrap_or(15);

    let max_tokens: u32 = std::env::var("GROK_MAX_TOKENS")
        .unwrap_or_else(|_| "1000".to_string())
        .parse()
        .unwrap_or(1000);

    let prompt = format!("What is the Windows service \"{}\"?\n\nPlease provide a JSON response with exactly these three keys:\n- \"description\": A brief description of what this service does\n- \"explained\": A concise explanation in 2-3 lines of its purpose and functionality\n- \"recommendation\": A bullet-point list covering whether to disable it, what would be affected, and safe disabling scenarios\n\nExample format:\n{{\n  \"description\": \"Brief description here\",\n  \"explained\": \"Concise explanation here\",\n  \"recommendation\": \"• Point 1\\n• Point 2\\n• Point 3\"\n}}\n\nReturn only valid JSON, no additional text.", service_name);

    let client = reqwest::Client::new();
    let response = timeout(
        Duration::from_secs(timeout_secs),
        client
            .post("https://api.x.ai/v1/chat/completions")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&serde_json::json!({
                "model": "grok-3-mini",
                "messages": [{"role": "user", "content": prompt}],
                "max_tokens": max_tokens,
                "temperature": 0.7,
                "stream": false
            }))
            .send(),
    )
    .await
    .map_err(|_| format!("AI API request timed out after {} seconds", timeout_secs))?
    .map_err(|e| format!("AI API request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("AI API error {}: {}", status, error_text));
    }

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse AI response: {}", e))?;

    let ai_response = data["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("Invalid AI response format: missing content".to_string())?;

    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(ai_response) {
        Ok(ServiceInfo {
            description: parsed["description"].as_str().map(|s| s.to_string()),
            explained: parsed["explained"].as_str().map(|s| s.to_string()),
            recommendation: parsed["recommendation"].as_str().map(|s| s.to_string()),
        })
    } else {
        // Fallback: extract information from text response
        let description = extract_field_from_text(ai_response, "description");
        let explained = extract_field_from_text(ai_response, "explained");
        let recommendation = extract_field_from_text(ai_response, "recommendation");

        Ok(ServiceInfo {
            description: description.or_else(|| Some("AI-generated description".to_string())),
            explained: explained.or_else(|| Some("AI-generated explanation".to_string())),
            recommendation: recommendation.or_else(|| Some("AI-generated recommendation".to_string())),
        })
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
