//! OpenAI-compatible provider implementation.

use crate::domain::error::{AiError, ProviderExecutionFailureKind, ResponseParseFailureKind};
use crate::domain::provider::AiProvider;
use crate::domain::types::{AnalyzeDiffRequest, AnalyzeDiffResponse, PromptPayload, RiskLevel};
use serde_json::{json, Value};
use std::time::Duration;

const PROVIDER_NAME: &str = "openai-compatible";

/// Provider for OpenAI-compatible chat/completions backends.
#[derive(Debug, Default)]
pub struct OpenAiCompatibleProvider;

impl OpenAiCompatibleProvider {
    /// Creates a new provider instance.
    pub fn new() -> Self {
        Self
    }
}

impl AiProvider for OpenAiCompatibleProvider {
    fn analyze_diff(
        &self,
        req: AnalyzeDiffRequest,
        prompt: PromptPayload,
    ) -> Result<AnalyzeDiffResponse, AiError> {
        let endpoint = required_config_value(
            req.config.openai_endpoint.as_deref(),
            ProviderExecutionFailureKind::MissingEndpoint,
            "openai endpoint is required",
        )?;
        if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
            return Err(provider_error(
                ProviderExecutionFailureKind::InvalidEndpoint,
                format!("endpoint must start with http:// or https://: {endpoint}"),
            ));
        }
        let api_key = required_config_value(
            req.config.openai_api_key.as_deref(),
            ProviderExecutionFailureKind::MissingApiKey,
            "openai api key is required",
        )?;
        let model = required_config_value(
            req.config.openai_model.as_deref(),
            ProviderExecutionFailureKind::MissingModel,
            "openai model is required",
        )?;

        let timeout_secs = req.config.request_timeout_secs.max(1);
        let url = format!("{}/chat/completions", endpoint.trim_end_matches('/'));
        let request_body = json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": prompt.system_instruction
                },
                {
                    "role": "user",
                    "content": prompt.user_prompt
                }
            ],
            "temperature": req.config.temperature,
            "max_tokens": req.config.max_output_tokens
        });

        let response_body = send_chat_completions(
            &url,
            &api_key,
            request_body,
            Duration::from_secs(timeout_secs),
        )?;
        parse_chat_completions_response(&response_body)
    }

    fn name(&self) -> &'static str {
        PROVIDER_NAME
    }
}

fn required_config_value(
    raw: Option<&str>,
    kind: ProviderExecutionFailureKind,
    message: &str,
) -> Result<String, AiError> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .ok_or_else(|| provider_error(kind, message.to_string()))
}

fn send_chat_completions(
    url: &str,
    api_key: &str,
    body: Value,
    timeout: Duration,
) -> Result<String, AiError> {
    let agent = ureq::AgentBuilder::new().timeout(timeout).build();
    let request = agent
        .post(url)
        .set("Authorization", &format!("Bearer {api_key}"))
        .set("Content-Type", "application/json");

    let response = request.send_json(body).map_err(|err| match err {
        ureq::Error::Status(status, _response) => provider_error(
            ProviderExecutionFailureKind::HttpStatusNonSuccess,
            format!("http status {status}"),
        ),
        ureq::Error::Transport(transport) => {
            let message = transport.to_string();
            let lower = message.to_ascii_lowercase();
            let kind = if lower.contains("timeout") || lower.contains("timed out") {
                ProviderExecutionFailureKind::Timeout
            } else {
                ProviderExecutionFailureKind::NetworkFailure
            };
            provider_error(kind, message)
        }
    })?;

    response.into_string().map_err(|err| {
        provider_error(
            ProviderExecutionFailureKind::NetworkFailure,
            format!("failed to read response body: {err}"),
        )
    })
}

fn parse_chat_completions_response(raw: &str) -> Result<AnalyzeDiffResponse, AiError> {
    let envelope: Value = serde_json::from_str(raw).map_err(|err| {
        response_parse_error(
            ResponseParseFailureKind::InvalidJson,
            format!("response envelope is not valid json: {err}"),
        )
    })?;

    let content = envelope
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            response_parse_error(
                ResponseParseFailureKind::MissingContent,
                "missing choices[0].message.content".to_string(),
            )
        })?;

    let normalized = strip_markdown_json_fence(content);
    let payload: Value = serde_json::from_str(&normalized).map_err(|err| {
        response_parse_error(
            ResponseParseFailureKind::InvalidJson,
            format!("content is not valid json object: {err}"),
        )
    })?;
    parse_response_contract(payload)
}

fn parse_response_contract(payload: Value) -> Result<AnalyzeDiffResponse, AiError> {
    let title = payload
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .ok_or_else(|| {
            response_parse_error(
                ResponseParseFailureKind::InvalidContract,
                "title is required".to_string(),
            )
        })?;

    let rationale = payload
        .get("rationale")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .ok_or_else(|| {
            response_parse_error(
                ResponseParseFailureKind::InvalidContract,
                "rationale is required".to_string(),
            )
        })?;

    let risk_level_text = payload
        .get("risk_level")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            response_parse_error(
                ResponseParseFailureKind::InvalidContract,
                "risk_level is required".to_string(),
            )
        })?;
    let risk_level = parse_risk_level(risk_level_text).ok_or_else(|| {
        response_parse_error(
            ResponseParseFailureKind::InvalidContract,
            format!("unsupported risk_level value: {risk_level_text}"),
        )
    })?;

    let key_points = parse_string_list(payload.get("key_points"));
    let review_suggestions = parse_string_list(payload.get("review_suggestions"));

    Ok(AnalyzeDiffResponse {
        risk_level,
        title,
        rationale,
        key_points,
        review_suggestions,
    })
}

fn parse_risk_level(raw: &str) -> Option<RiskLevel> {
    match raw.to_ascii_lowercase().as_str() {
        "low" => Some(RiskLevel::Low),
        "medium" => Some(RiskLevel::Medium),
        "high" => Some(RiskLevel::High),
        _ => None,
    }
}

fn parse_string_list(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(|item| item.to_string())
            .collect(),
        Some(Value::String(item)) => {
            let item = item.trim();
            if item.is_empty() {
                Vec::new()
            } else {
                vec![item.to_string()]
            }
        }
        _ => Vec::new(),
    }
}

fn strip_markdown_json_fence(content: &str) -> String {
    let trimmed = content.trim();
    if !trimmed.starts_with("```") {
        return trimmed.to_string();
    }

    let lines = trimmed.lines().collect::<Vec<_>>();
    if lines.len() < 3 {
        return trimmed.to_string();
    }
    if !lines[0].starts_with("```") || !lines[lines.len() - 1].starts_with("```") {
        return trimmed.to_string();
    }

    lines[1..lines.len() - 1].join("\n").trim().to_string()
}

fn provider_error(kind: ProviderExecutionFailureKind, message: String) -> AiError {
    AiError::ProviderExecutionFailed {
        provider: PROVIDER_NAME.to_string(),
        kind,
        message,
    }
}

fn response_parse_error(kind: ResponseParseFailureKind, message: String) -> AiError {
    AiError::ResponseParseFailed {
        provider: PROVIDER_NAME.to_string(),
        kind,
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::{AiConfig, AiProviderKind, AnalysisTask};
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    fn remote_request() -> AnalyzeDiffRequest {
        let mut config = AiConfig::default();
        config.provider_kind = AiProviderKind::OpenAiCompatible;
        config.openai_endpoint = Some("http://127.0.0.1:1".to_string());
        config.openai_api_key = Some("test-key".to_string());
        config.openai_model = Some("test-model".to_string());
        config.request_timeout_secs = 1;

        AnalyzeDiffRequest {
            task: AnalysisTask::Summary,
            relative_path: Some("src/lib.rs".to_string()),
            language_hint: Some("rust".to_string()),
            diff_excerpt: "-a\n+b".to_string(),
            summary: None,
            truncation_note: None,
            config,
        }
    }

    fn spawn_one_shot_server(
        status_code: u16,
        body: String,
        response_delay: Duration,
    ) -> (String, Arc<Mutex<Option<String>>>, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener bind should succeed");
        let endpoint = format!(
            "http://{}",
            listener.local_addr().expect("addr should exist")
        );
        let captured = Arc::new(Mutex::new(None::<String>));
        let captured_ref = Arc::clone(&captured);
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept should succeed");
            let stream_for_read = stream
                .try_clone()
                .expect("stream clone for read should succeed");
            let mut reader = BufReader::new(stream_for_read);

            let mut raw_headers = String::new();
            loop {
                let mut line = String::new();
                let bytes = reader.read_line(&mut line).unwrap_or(0);
                if bytes == 0 {
                    break;
                }
                raw_headers.push_str(&line);
                if line == "\r\n" {
                    break;
                }
            }

            let content_length = raw_headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    if name.eq_ignore_ascii_case("content-length") {
                        value.trim().parse::<usize>().ok()
                    } else {
                        None
                    }
                })
                .unwrap_or(0);
            let mut body_buf = vec![0u8; content_length];
            if content_length > 0 {
                let _ = reader.read_exact(&mut body_buf);
            }
            let request_text = if body_buf.is_empty() {
                raw_headers
            } else {
                format!("{}{}", raw_headers, String::from_utf8_lossy(&body_buf))
            };
            *captured_ref.lock().expect("capture lock should succeed") = Some(request_text);
            if !response_delay.is_zero() {
                thread::sleep(response_delay);
            }
            let status_text = match status_code {
                200 => "OK",
                401 => "Unauthorized",
                500 => "Internal Server Error",
                _ => "Unknown",
            };
            let response = format!(
                "HTTP/1.1 {status_code} {status_text}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        });

        (endpoint, captured, handle)
    }

    #[test]
    fn provider_rejects_missing_endpoint() {
        let provider = OpenAiCompatibleProvider::new();
        let mut req = remote_request();
        req.config.openai_endpoint = None;

        let err = provider
            .analyze_diff(
                req,
                PromptPayload {
                    system_instruction: "sys".to_string(),
                    user_prompt: "user".to_string(),
                },
            )
            .expect_err("provider should reject missing endpoint");
        assert!(matches!(
            err,
            AiError::ProviderExecutionFailed {
                kind: ProviderExecutionFailureKind::MissingEndpoint,
                ..
            }
        ));
    }

    #[test]
    fn provider_rejects_missing_api_key_or_model() {
        let provider = OpenAiCompatibleProvider::new();
        let mut req = remote_request();
        req.config.openai_api_key = None;
        let err = provider
            .analyze_diff(
                req,
                PromptPayload {
                    system_instruction: "sys".to_string(),
                    user_prompt: "user".to_string(),
                },
            )
            .expect_err("provider should reject missing api key");
        assert!(matches!(
            err,
            AiError::ProviderExecutionFailed {
                kind: ProviderExecutionFailureKind::MissingApiKey,
                ..
            }
        ));

        let mut req = remote_request();
        req.config.openai_model = Some("".to_string());
        let err = provider
            .analyze_diff(
                req,
                PromptPayload {
                    system_instruction: "sys".to_string(),
                    user_prompt: "user".to_string(),
                },
            )
            .expect_err("provider should reject missing model");
        assert!(matches!(
            err,
            AiError::ProviderExecutionFailed {
                kind: ProviderExecutionFailureKind::MissingModel,
                ..
            }
        ));
    }

    #[test]
    fn provider_rejects_invalid_endpoint() {
        let provider = OpenAiCompatibleProvider::new();
        let mut req = remote_request();
        req.config.openai_endpoint = Some("not-a-url".to_string());

        let err = provider
            .analyze_diff(
                req,
                PromptPayload {
                    system_instruction: "sys".to_string(),
                    user_prompt: "user".to_string(),
                },
            )
            .expect_err("provider should reject malformed endpoint");
        assert!(matches!(
            err,
            AiError::ProviderExecutionFailed {
                kind: ProviderExecutionFailureKind::InvalidEndpoint,
                ..
            }
        ));
    }

    #[test]
    fn provider_maps_http_non_success_status() {
        let provider = OpenAiCompatibleProvider::new();
        let (endpoint, _captured, handle) = spawn_one_shot_server(
            401,
            "{\"error\":{\"message\":\"bad key\"}}".to_string(),
            Duration::from_millis(0),
        );
        let mut req = remote_request();
        req.config.openai_endpoint = Some(endpoint);

        let err = provider
            .analyze_diff(
                req,
                PromptPayload {
                    system_instruction: "sys".to_string(),
                    user_prompt: "user".to_string(),
                },
            )
            .expect_err("provider should return http status failure");
        handle.join().expect("server thread should join");
        assert!(matches!(
            err,
            AiError::ProviderExecutionFailed {
                kind: ProviderExecutionFailureKind::HttpStatusNonSuccess,
                ..
            }
        ));
    }

    #[test]
    fn provider_parses_successful_response_and_builds_request() {
        let provider = OpenAiCompatibleProvider::new();
        let assistant_content = serde_json::json!({
            "risk_level": "medium",
            "title": "Risk review for src/lib.rs",
            "rationale": "Potential branch behavior changed.",
            "key_points": ["added branch", "changed fallback"],
            "review_suggestions": ["add regression tests"]
        })
        .to_string();
        let envelope = serde_json::json!({
            "choices": [
                {
                    "message": {
                        "content": assistant_content
                    }
                }
            ]
        })
        .to_string();
        let (endpoint, captured, handle) =
            spawn_one_shot_server(200, envelope, Duration::from_millis(0));
        let mut req = remote_request();
        req.config.openai_endpoint = Some(endpoint);

        let response = provider
            .analyze_diff(
                req,
                PromptPayload {
                    system_instruction: "sys".to_string(),
                    user_prompt: "user".to_string(),
                },
            )
            .expect("provider should parse successful response");
        handle.join().expect("server thread should join");

        assert_eq!(response.risk_level, RiskLevel::Medium);
        assert!(response.title.contains("src/lib.rs"));
        assert_eq!(response.key_points.len(), 2);
        assert_eq!(response.review_suggestions.len(), 1);

        let raw_request = captured
            .lock()
            .expect("capture lock should succeed")
            .clone()
            .unwrap_or_default();
        assert!(raw_request.contains("POST /chat/completions HTTP/1.1"));
        assert!(raw_request.contains("Authorization: Bearer test-key"));
        assert!(raw_request.contains("\"model\":\"test-model\""));
        assert!(raw_request.contains("\"max_tokens\":512"));
    }

    #[test]
    fn provider_reports_parse_failure_for_invalid_contract() {
        let provider = OpenAiCompatibleProvider::new();
        let envelope = serde_json::json!({
            "choices": [
                {
                    "message": {
                        "content": "{\"risk_level\":\"bad\",\"title\":\"x\",\"rationale\":\"y\"}"
                    }
                }
            ]
        })
        .to_string();
        let (endpoint, _captured, handle) =
            spawn_one_shot_server(200, envelope, Duration::from_millis(0));
        let mut req = remote_request();
        req.config.openai_endpoint = Some(endpoint);

        let err = provider
            .analyze_diff(
                req,
                PromptPayload {
                    system_instruction: "sys".to_string(),
                    user_prompt: "user".to_string(),
                },
            )
            .expect_err("provider should reject invalid contract");
        handle.join().expect("server thread should join");
        assert!(matches!(
            err,
            AiError::ResponseParseFailed {
                kind: ResponseParseFailureKind::InvalidContract,
                ..
            }
        ));
    }

    #[test]
    fn provider_maps_timeout_failure() {
        let provider = OpenAiCompatibleProvider::new();
        let envelope = serde_json::json!({
            "choices": [
                {
                    "message": {
                        "content": "{\"risk_level\":\"low\",\"title\":\"t\",\"rationale\":\"r\"}"
                    }
                }
            ]
        })
        .to_string();
        let (endpoint, _captured, handle) =
            spawn_one_shot_server(200, envelope, Duration::from_secs(2));
        let mut req = remote_request();
        req.config.openai_endpoint = Some(endpoint);
        req.config.request_timeout_secs = 1;

        let err = provider
            .analyze_diff(
                req,
                PromptPayload {
                    system_instruction: "sys".to_string(),
                    user_prompt: "user".to_string(),
                },
            )
            .expect_err("provider should report timeout");
        handle.join().expect("server thread should join");
        assert!(matches!(
            err,
            AiError::ProviderExecutionFailed {
                kind: ProviderExecutionFailureKind::Timeout,
                ..
            }
        ));
    }
}
