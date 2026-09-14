use crate::models::{AppSettings, ChatMessage, ConnectionTest};
use futures_util::StreamExt;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub async fn chat_stream<F>(
    settings: &AppSettings,
    messages: Vec<ChatMessage>,
    cancel: &AtomicBool,
    mut on_chunk: F,
) -> Result<String, String>
where
    F: FnMut(&str) -> Result<(), String>,
{
    let url = endpoint(&settings.base_url, "chat/completions");
    let body = json!({
        "model": settings.model,
        "temperature": settings.temperature,
        "max_tokens": settings.max_tokens,
        "stream": true,
        "messages": messages,
    });

    let client = client(120)?;
    let response = client
        .post(&url)
        .headers(headers(settings))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("无法连接模型服务：{e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("模型接口返回 {status}：{}", clip_err(&text)));
    }

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut full = String::new();

    while let Some(part) = stream.next().await {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        let part = part.map_err(|e| format!("读取流失败：{e}"))?;
        buffer.push_str(&String::from_utf8_lossy(&part));

        while let Some(idx) = buffer.find('\n') {
            let line = buffer[..idx].trim().to_string();
            buffer = buffer[idx + 1..].to_string();
            if let Some(token) = parse_sse_line(&line) {
                full.push_str(&token);
                on_chunk(&token)?;
            }
        }
    }

    if !buffer.trim().is_empty() {
        if let Some(token) = parse_sse_line(buffer.trim()) {
            full.push_str(&token);
            on_chunk(&token)?;
        }
    }

    if full.trim().is_empty() && !cancel.load(Ordering::Relaxed) {
        return Err("模型没有返回内容，请检查地址、模型名和密钥。".into());
    }
    Ok(full)
}

pub async fn chat_once(settings: &AppSettings, messages: Vec<ChatMessage>) -> Result<String, String> {
    let url = endpoint(&settings.base_url, "chat/completions");
    let body = json!({
        "model": settings.model,
        "temperature": settings.temperature.min(0.4),
        "max_tokens": settings.max_tokens.min(1200),
        "stream": false,
        "messages": messages,
    });
    let client = client(90)?;
    let response = client
        .post(&url)
        .headers(headers(settings))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("无法连接模型服务：{e}"))?;
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("模型接口返回 {status}：{}", clip_err(&text)));
    }
    let value: Value = response.json().await.map_err(|e| e.to_string())?;
    value["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "模型没有返回内容。".into())
}

pub async fn test_connection(settings: &AppSettings) -> Result<ConnectionTest, String> {
    if settings.base_url.trim().is_empty() || settings.model.trim().is_empty() {
        return Ok(ConnectionTest {
            ok: false,
            message: "请先填写接口地址和模型名。".into(),
            models: vec![],
        });
    }

    let client = client(20)?;
    let models_url = endpoint(&settings.base_url, "models");
    let models_resp = client.get(&models_url).headers(headers(settings)).send().await;

    let mut models = Vec::new();
    if let Ok(resp) = models_resp {
        if resp.status().is_success() {
            if let Ok(value) = resp.json::<Value>().await {
                if let Some(arr) = value.get("data").and_then(|v| v.as_array()) {
                    for item in arr {
                        if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                            models.push(id.to_string());
                        }
                    }
                }
            }
        }
    }

    let probe = chat_once(
        settings,
        vec![
            ChatMessage {
                role: "system".into(),
                content: "只回复一个字：好".into(),
            },
            ChatMessage {
                role: "user".into(),
                content: "测".into(),
            },
        ],
    )
    .await;

    match probe {
        Ok(_) => Ok(ConnectionTest {
            ok: true,
            message: if models.is_empty() {
                format!("已连通 {} / {}", settings.mode_label(), settings.model)
            } else {
                format!(
                    "已连通 {}，当前模型 {}，服务列出 {} 个模型。",
                    settings.mode_label(),
                    settings.model,
                    models.len()
                )
            },
            models,
        }),
        Err(err) => Ok(ConnectionTest {
            ok: false,
            message: err,
            models,
        }),
    }
}

trait ModeLabel {
    fn mode_label(&self) -> &str;
}

impl ModeLabel for AppSettings {
    fn mode_label(&self) -> &str {
        if self.mode == "local" {
            "本地模型"
        } else {
            "在线模型"
        }
    }
}

fn client(timeout_secs: u64) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| e.to_string())
}

fn headers(settings: &AppSettings) -> reqwest::header::HeaderMap {
    let mut map = reqwest::header::HeaderMap::new();
    map.insert(
        reqwest::header::CONTENT_TYPE,
        "application/json".parse().unwrap(),
    );
    let key = if settings.api_key.trim().is_empty() {
        "local"
    } else {
        settings.api_key.trim()
    };
    if let Ok(value) = format!("Bearer {key}").parse() {
        map.insert(reqwest::header::AUTHORIZATION, value);
    }
    map
}

fn endpoint(base: &str, path: &str) -> String {
    let trimmed = base.trim().trim_end_matches('/');
    format!("{trimmed}/{path}")
}

fn parse_sse_line(line: &str) -> Option<String> {
    let data = line.strip_prefix("data:")?.trim();
    if data.is_empty() || data == "[DONE]" {
        return None;
    }
    let value: Value = serde_json::from_str(data).ok()?;
    let delta = &value["choices"][0]["delta"];
    delta
        .get("content")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn clip_err(text: &str) -> String {
    let t = text.trim();
    if t.chars().count() > 280 {
        format!("{}…", t.chars().take(280).collect::<String>())
    } else {
        t.to_string()
    }
}
