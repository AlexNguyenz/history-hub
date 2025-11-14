// ============================================
// RUST TUTORIAL - PHASE 2: PARSE CLAUDE JSONL
// ============================================

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};

// ============================================
// DATA STRUCTURES FOR CLAUDE CODE HISTORY
// ============================================

/// Content item trong message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentItem {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },

    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: serde_json::Value,
    },
}

/// Token usage từ Claude API
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: i32,
    pub output_tokens: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<i32>,
}

/// Message object từ Claude API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageObject {
    pub role: String,
    pub content: Vec<ContentItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TokenUsage>,
}

/// Raw log entry từ JSONL file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawLogEntry {
    #[serde(rename = "type")]
    pub entry_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "parentUuid")]
    pub parent_uuid: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<MessageObject>,
}

/// Parsed Claude message (simplified for JavaScript)
#[napi(object)]
#[derive(Debug, Clone)]
pub struct ClaudeMessage {
    pub message_id: String,
    pub session_id: String,
    pub role: String,              // "user" or "assistant"
    pub content: String,            // Text content (merged from content array)
    pub timestamp: String,

    // Optional fields
    pub parent_id: Option<String>,
    pub model: Option<String>,
    pub stop_reason: Option<String>,

    // Token usage
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
}

/// Session summary
#[napi(object)]
#[derive(Debug, Clone)]
pub struct ClaudeSession {
    pub session_id: String,
    pub file_path: String,
    pub message_count: i32,
    pub user_message_count: i32,
    pub assistant_message_count: i32,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
}

// ============================================
// PHASE 1 FUNCTIONS (từ trước)
// ============================================

#[napi]
pub fn count_lines(file_path: String) -> Result<i32> {
    let file = File::open(&file_path)
        .map_err(|e| Error::from_reason(format!("Cannot open file: {}", e)))?;

    let reader = BufReader::new(file);
    let mut count = 0;

    for line in reader.lines() {
        match line {
            Ok(_) => count += 1,
            Err(e) => {
                eprintln!("Error reading line: {}", e);
            }
        }
    }

    Ok(count)
}

#[napi]
pub fn read_lines(file_path: String) -> Result<Vec<String>> {
    let file = File::open(&file_path)
        .map_err(|e| Error::from_reason(format!("Cannot open file: {}", e)))?;

    let reader = BufReader::new(file);
    let mut lines = Vec::new();

    for line in reader.lines() {
        match line {
            Ok(content) => {
                if !content.trim().is_empty() {
                    lines.push(content);
                }
            }
            Err(e) => {
                eprintln!("Error reading line: {}", e);
            }
        }
    }

    Ok(lines)
}

#[napi]
pub fn read_lines_with_pattern(file_path: String, pattern: String) -> Result<Vec<String>> {
    let file = File::open(&file_path)
        .map_err(|e| Error::from_reason(format!("Cannot open file: {}", e)))?;

    let reader = BufReader::new(file);
    let mut matching_lines = Vec::new();

    for line in reader.lines() {
        if let Ok(content) = line {
            if content.contains(&pattern) {
                matching_lines.push(content);
            }
        }
    }

    Ok(matching_lines)
}

#[napi]
pub fn get_file_info(file_path: String) -> Result<String> {
    let path = PathBuf::from(&file_path);

    if !path.exists() {
        return Err(Error::from_reason("File does not exist".to_string()));
    }

    let metadata = std::fs::metadata(&path)
        .map_err(|e| Error::from_reason(format!("Cannot read metadata: {}", e)))?;

    let file_size = metadata.len();
    let line_count = count_lines(file_path.clone())?;

    let info = format!(
        "File: {}\nSize: {} bytes\nLines: {}",
        path.display(),
        file_size,
        line_count
    );

    Ok(info)
}

// ============================================
// PHASE 2: PARSE CLAUDE MESSAGES
// ============================================

/// Parse một dòng JSONL thành RawLogEntry
fn parse_jsonl_line(line: &str) -> std::result::Result<RawLogEntry, serde_json::Error> {
    serde_json::from_str(line)
}

/// Extract text content từ content array
fn extract_text_content(content_items: &[ContentItem]) -> String {
    content_items
        .iter()
        .filter_map(|item| match item {
            ContentItem::Text { text } => Some(text.clone()),
            _ => None,
        })
        .collect::<Vec<String>>()
        .join("\n")
}

/// Convert RawLogEntry thành ClaudeMessage
fn entry_to_message(entry: RawLogEntry) -> Option<ClaudeMessage> {
    // Chỉ process user và assistant messages
    if entry.entry_type != "user" && entry.entry_type != "assistant" {
        return None;
    }

    // Cần có message object
    let message = entry.message?;

    // Extract text content
    let content = extract_text_content(&message.content);

    // Get token usage
    let (input_tokens, output_tokens) = if let Some(usage) = message.usage {
        (Some(usage.input_tokens), Some(usage.output_tokens))
    } else {
        (None, None)
    };

    Some(ClaudeMessage {
        message_id: entry.uuid.unwrap_or_else(|| "unknown".to_string()),
        session_id: entry.session_id.unwrap_or_else(|| "unknown".to_string()),
        role: message.role,
        content,
        timestamp: entry.timestamp.unwrap_or_else(|| "unknown".to_string()),
        parent_id: entry.parent_uuid,
        model: message.model,
        stop_reason: message.stop_reason,
        input_tokens,
        output_tokens,
    })
}

/// Parse Claude Code session file và return messages
#[napi]
pub fn parse_claude_session(file_path: String) -> Result<Vec<ClaudeMessage>> {
    let file = File::open(&file_path)
        .map_err(|e| Error::from_reason(format!("Cannot open file: {}", e)))?;

    let reader = BufReader::new(file);
    let mut messages = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| {
            Error::from_reason(format!("Error reading line {}: {}", line_num, e))
        })?;

        if line.trim().is_empty() {
            continue;
        }

        // Parse JSONL line
        match parse_jsonl_line(&line) {
            Ok(entry) => {
                // Convert to ClaudeMessage
                if let Some(msg) = entry_to_message(entry) {
                    messages.push(msg);
                }
            }
            Err(e) => {
                // Log error nhưng continue parsing
                eprintln!("Parse error at line {}: {}", line_num, e);
            }
        }
    }

    Ok(messages)
}

/// Get session summary (metadata)
#[napi]
pub fn get_session_summary(file_path: String) -> Result<ClaudeSession> {
    let file = File::open(&file_path)
        .map_err(|e| Error::from_reason(format!("Cannot open file: {}", e)))?;

    let reader = BufReader::new(file);

    let mut session_id = String::from("unknown");
    let mut message_count = 0;
    let mut user_count = 0;
    let mut assistant_count = 0;
    let mut first_timestamp: Option<String> = None;
    let mut last_timestamp: Option<String> = None;

    for line in reader.lines() {
        if let Ok(line) = line {
            if line.trim().is_empty() {
                continue;
            }

            if let Ok(entry) = parse_jsonl_line(&line) {
                // Update session ID
                if let Some(sid) = entry.session_id {
                    session_id = sid;
                }

                // Count messages
                match entry.entry_type.as_str() {
                    "user" => {
                        user_count += 1;
                        message_count += 1;
                    }
                    "assistant" => {
                        assistant_count += 1;
                        message_count += 1;
                    }
                    _ => {}
                }

                // Track timestamps
                if let Some(ts) = entry.timestamp {
                    if first_timestamp.is_none() {
                        first_timestamp = Some(ts.clone());
                    }
                    last_timestamp = Some(ts);
                }
            }
        }
    }

    Ok(ClaudeSession {
        session_id,
        file_path: file_path.clone(),
        message_count,
        user_message_count: user_count,
        assistant_message_count: assistant_count,
        first_timestamp,
        last_timestamp,
    })
}

// ============================================
// TEST MODULE
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_jsonl_line() {
        let json = r#"{"type":"user","uuid":"123","sessionId":"abc","message":{"role":"user","content":[{"type":"text","text":"Hello"}]}}"#;
        let entry = parse_jsonl_line(json).unwrap();
        assert_eq!(entry.entry_type, "user");
        assert_eq!(entry.uuid, Some("123".to_string()));
    }
}
