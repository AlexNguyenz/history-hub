// ============================================
// CLAUDE CODE HISTORY PARSER - ENHANCED VERSION
// Based on claude-code-history-viewer research
// ============================================

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};

// ============================================
// ENHANCED DATA STRUCTURES
// ============================================

/// Content item variants - supports all Claude content types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentItem {
    #[serde(rename = "text")]
    Text {
        text: String
    },

    #[serde(rename = "thinking")]
    Thinking {
        thinking: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },

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
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },

    #[serde(rename = "image")]
    Image {
        source: ImageSource,
    },
}

/// Image source data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub source_type: String,  // "base64"
    pub media_type: String,   // "image/png", "image/jpeg", etc.
    pub data: String,         // base64 encoded data
}

/// Token usage with cache support
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

/// Message object - supports both string and array content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageObject {
    pub role: String,

    // Content can be string (user) or array (assistant)
    #[serde(deserialize_with = "deserialize_content")]
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

/// Custom deserializer for content field (handles both string and array)
fn deserialize_content<'de, D>(deserializer: D) -> std::result::Result<Vec<ContentItem>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    let value: serde_json::Value = serde::Deserialize::deserialize(deserializer)?;

    match value {
        // String content (user messages)
        serde_json::Value::String(s) => Ok(vec![ContentItem::Text { text: s }]),

        // Array content (assistant messages)
        serde_json::Value::Array(arr) => {
            let items: std::result::Result<Vec<ContentItem>, _> = arr
                .into_iter()
                .map(|v| serde_json::from_value(v).map_err(Error::custom))
                .collect();
            items
        }

        _ => Err(Error::custom("Content must be string or array")),
    }
}

/// Raw log entry from JSONL
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

    // Additional fields for completeness
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "leafUuid")]
    pub leaf_uuid: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "isSidechain")]
    pub is_sidechain: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "userType")]
    pub user_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// Enhanced Claude message with full content support
#[napi(object)]
#[derive(Debug, Clone)]
pub struct ClaudeMessage {
    pub message_id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,  // Main text content (merged)
    pub timestamp: String,

    // Content details (serialized as JSON)
    pub raw_content: String,  // Full content array as JSON
    pub has_thinking: bool,
    pub has_tool_use: bool,
    pub has_images: bool,

    // Optional fields
    pub parent_id: Option<String>,
    pub model: Option<String>,
    pub stop_reason: Option<String>,

    // Token usage
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub cache_creation_tokens: Option<i32>,
    pub cache_read_tokens: Option<i32>,

    // Additional metadata
    pub is_sidechain: Option<bool>,
    pub user_type: Option<String>,
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

    // Enhanced stats
    pub total_input_tokens: Option<i32>,
    pub total_output_tokens: Option<i32>,
    pub has_thinking: bool,
    pub has_tool_use: bool,

    // Project info
    pub cwd: Option<String>,
}

// ============================================
// PARSING FUNCTIONS
// ============================================

/// Parse a JSONL line with better error handling
fn parse_jsonl_line(line: &str) -> std::result::Result<RawLogEntry, serde_json::Error> {
    serde_json::from_str(line)
}

/// Extract all text content from content array
fn extract_text_content(content_items: &[ContentItem]) -> String {
    content_items
        .iter()
        .filter_map(|item| match item {
            ContentItem::Text { text } => Some(text.clone()),
            ContentItem::Thinking { thinking, .. } => Some(format!("[Thinking]\n{}", thinking)),
            _ => None,
        })
        .collect::<Vec<String>>()
        .join("\n\n")
}

/// Check if content has thinking
fn has_thinking(content_items: &[ContentItem]) -> bool {
    content_items.iter().any(|item| matches!(item, ContentItem::Thinking { .. }))
}

/// Check if content has tool use
fn has_tool_use(content_items: &[ContentItem]) -> bool {
    content_items.iter().any(|item| matches!(item, ContentItem::ToolUse { .. }))
}

/// Check if content has images
fn has_images(content_items: &[ContentItem]) -> bool {
    content_items.iter().any(|item| matches!(item, ContentItem::Image { .. }))
}

/// Convert RawLogEntry to ClaudeMessage with full content support
fn entry_to_message(entry: RawLogEntry) -> Option<ClaudeMessage> {
    // Only process user and assistant messages
    if entry.entry_type != "user" && entry.entry_type != "assistant" {
        return None;
    }

    let message = entry.message?;

    // Extract text content
    let content = extract_text_content(&message.content);

    // Serialize full content as JSON for frontend
    let raw_content = serde_json::to_string(&message.content).unwrap_or_default();

    // Detect content features
    let has_thinking_flag = has_thinking(&message.content);
    let has_tool_use_flag = has_tool_use(&message.content);
    let has_images_flag = has_images(&message.content);

    // Get token usage
    let (input_tokens, output_tokens, cache_creation, cache_read) = if let Some(usage) = message.usage {
        (
            Some(usage.input_tokens),
            Some(usage.output_tokens),
            usage.cache_creation_input_tokens,
            usage.cache_read_input_tokens,
        )
    } else {
        (None, None, None, None)
    };

    Some(ClaudeMessage {
        message_id: entry.uuid.unwrap_or_else(|| "unknown".to_string()),
        session_id: entry.session_id.unwrap_or_else(|| "unknown".to_string()),
        role: message.role,
        content,
        timestamp: entry.timestamp.unwrap_or_else(|| "unknown".to_string()),
        raw_content,
        has_thinking: has_thinking_flag,
        has_tool_use: has_tool_use_flag,
        has_images: has_images_flag,
        parent_id: entry.parent_uuid,
        model: message.model,
        stop_reason: message.stop_reason,
        input_tokens,
        output_tokens,
        cache_creation_tokens: cache_creation,
        cache_read_tokens: cache_read,
        is_sidechain: entry.is_sidechain,
        user_type: entry.user_type,
    })
}

// ============================================
// EXPORTED FUNCTIONS
// ============================================

/// Parse Claude Code session file and return all messages
#[napi]
pub fn parse_claude_session(file_path: String) -> Result<Vec<ClaudeMessage>> {
    let file = File::open(&file_path)
        .map_err(|e| Error::from_reason(format!("Cannot open file: {}", e)))?;

    let reader = BufReader::new(file);
    let mut messages = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| {
            Error::from_reason(format!("Error reading line {}: {}", line_num + 1, e))
        })?;

        if line.trim().is_empty() {
            continue;
        }

        // Parse JSONL line with graceful error handling
        match parse_jsonl_line(&line) {
            Ok(entry) => {
                if let Some(msg) = entry_to_message(entry) {
                    messages.push(msg);
                }
            }
            Err(e) => {
                // Log error but continue parsing
                eprintln!("⚠️  Parse error at line {}: {}", line_num + 1, e);
                eprintln!("   Line content: {}", &line[..line.len().min(100)]);
            }
        }
    }

    Ok(messages)
}

/// Get session summary with enhanced statistics
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
    let mut total_input_tokens = 0;
    let mut total_output_tokens = 0;
    let mut has_thinking_flag = false;
    let mut has_tool_use_flag = false;
    let mut cwd: Option<String> = None;

    for line in reader.lines() {
        if let Ok(line) = line {
            if line.trim().is_empty() {
                continue;
            }

            if let Ok(entry) = parse_jsonl_line(&line) {
                // Update session ID
                if let Some(sid) = &entry.session_id {
                    session_id = sid.clone();
                }

                // Capture cwd if available (only need to do this once)
                if cwd.is_none() {
                    if let Some(ref cwd_value) = entry.cwd {
                        cwd = Some(cwd_value.clone());
                    }
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

                        // Track token usage
                        if let Some(message) = &entry.message {
                            if let Some(usage) = &message.usage {
                                total_input_tokens += usage.input_tokens;
                                total_output_tokens += usage.output_tokens;
                            }

                            // Check for thinking and tool use
                            if has_thinking(&message.content) {
                                has_thinking_flag = true;
                            }
                            if has_tool_use(&message.content) {
                                has_tool_use_flag = true;
                            }
                        }
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
        total_input_tokens: if total_input_tokens > 0 { Some(total_input_tokens) } else { None },
        total_output_tokens: if total_output_tokens > 0 { Some(total_output_tokens) } else { None },
        has_thinking: has_thinking_flag,
        has_tool_use: has_tool_use_flag,
        cwd,
    })
}

// ============================================
// LEGACY FUNCTIONS (kept for compatibility)
// ============================================

#[napi]
pub fn count_lines(file_path: String) -> Result<i32> {
    let file = File::open(&file_path)
        .map_err(|e| Error::from_reason(format!("Cannot open file: {}", e)))?;

    let reader = BufReader::new(file);
    let count = reader.lines().count() as i32;

    Ok(count)
}

#[napi]
pub fn read_lines(file_path: String) -> Result<Vec<String>> {
    let file = File::open(&file_path)
        .map_err(|e| Error::from_reason(format!("Cannot open file: {}", e)))?;

    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| !line.trim().is_empty())
        .collect();

    Ok(lines)
}

#[napi]
pub fn read_lines_with_pattern(file_path: String, pattern: String) -> Result<Vec<String>> {
    let file = File::open(&file_path)
        .map_err(|e| Error::from_reason(format!("Cannot open file: {}", e)))?;

    let reader = BufReader::new(file);
    let matching_lines: Vec<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| line.contains(&pattern))
        .collect();

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
    let line_count = count_lines(file_path)?;

    let info = format!(
        "File: {}\nSize: {} bytes\nLines: {}",
        path.display(),
        file_size,
        line_count
    );

    Ok(info)
}

// ============================================
// TESTS
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_user_message() {
        let json = r#"{
            "type":"user",
            "uuid":"123",
            "sessionId":"abc",
            "timestamp":"2024-01-01T10:00:00Z",
            "message":{"role":"user","content":"Hello Claude"}
        }"#;

        let entry = parse_jsonl_line(json).unwrap();
        assert_eq!(entry.entry_type, "user");
        assert_eq!(entry.message.as_ref().unwrap().content.len(), 1);
    }

    #[test]
    fn test_parse_assistant_message_with_thinking() {
        let json = r#"{
            "type":"assistant",
            "uuid":"456",
            "sessionId":"abc",
            "timestamp":"2024-01-01T10:00:01Z",
            "message":{
                "role":"assistant",
                "content":[
                    {"type":"thinking","thinking":"Let me think..."},
                    {"type":"text","text":"Here's my response"}
                ]
            }
        }"#;

        let entry = parse_jsonl_line(json).unwrap();
        let msg = entry_to_message(entry).unwrap();
        assert!(msg.has_thinking);
        assert!(msg.content.contains("Let me think..."));
    }
}
