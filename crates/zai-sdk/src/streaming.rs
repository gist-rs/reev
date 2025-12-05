//! Streaming response implementation for ZAI SDK

use crate::error::{ZaiError, ZaiResult};
use futures::Stream;
use serde::Deserialize;
use std::pin::Pin;

/// A streaming response from the ZAI API
pub type StreamingResponse = Pin<Box<dyn Stream<Item = ZaiResult<String>> + Send>>;

/// Represents a chunk in a streaming response
#[derive(Debug, Deserialize)]
pub struct StreamingChunk {
    /// The choices in the chunk
    pub choices: Vec<StreamingChoice>,
    /// Usage information if available
    pub usage: Option<StreamingUsage>,
}

/// A choice in a streaming chunk
#[derive(Debug, Deserialize)]
pub struct StreamingChoice {
    /// The index of the choice
    pub index: u32,
    /// The delta of the choice
    pub delta: StreamingDelta,
    /// The finish reason if completed
    pub finish_reason: Option<String>,
}

/// The delta content of a streaming choice
#[derive(Debug, Clone, Deserialize)]
pub struct StreamingDelta {
    /// The text content
    pub content: Option<String>,
    /// Tool calls if present
    pub tool_calls: Option<Vec<StreamingToolCall>>,
    /// Reasoning content for coding model
    pub reasoning_content: Option<String>,
}

/// A tool call in a streaming delta
#[derive(Debug, Clone, Deserialize)]
pub struct StreamingToolCall {
    /// The index of the tool call
    pub index: u32,
    /// The ID of the tool call
    pub id: Option<String>,
    /// The type of tool call
    #[serde(rename = "type")]
    pub tool_type: Option<String>,
    /// The function being called
    pub function: Option<StreamingFunction>,
}

/// A function in a tool call
#[derive(Debug, Clone, Deserialize)]
pub struct StreamingFunction {
    /// The name of the function
    pub name: Option<String>,
    /// The arguments of the function
    pub arguments: Option<String>,
}

/// Usage information in a streaming chunk
#[derive(Debug, Deserialize)]
pub struct StreamingUsage {
    /// The number of prompt tokens used
    pub prompt_tokens: u32,
    /// The number of completion tokens used so far
    pub completion_tokens: u32,
    /// The total number of tokens used so far
    pub total_tokens: u32,
}

/// Builder for creating streaming responses
pub struct StreamingResponseBuilder {
    chunks: Vec<StreamingChunk>,
}

impl StreamingResponseBuilder {
    /// Create a new streaming response builder
    pub fn new() -> Self {
        Self { chunks: Vec::new() }
    }

    /// Add a chunk to the response
    pub fn add_chunk(mut self, chunk: StreamingChunk) -> Self {
        self.chunks.push(chunk);
        self
    }

    /// Add multiple chunks to the response
    pub fn add_chunks<I>(mut self, chunks: I) -> Self
    where
        I: IntoIterator<Item = StreamingChunk>,
    {
        self.chunks.extend(chunks);
        self
    }

    /// Build the streaming response
    pub fn build(self) -> StreamingResponse {
        Box::pin(futures::stream::iter(self.chunks.into_iter().map(
            |chunk| {
                let content = chunk
                    .choices
                    .iter()
                    .find_map(|choice| choice.delta.content.as_ref())
                    .or_else(|| {
                        chunk
                            .choices
                            .iter()
                            .find_map(|choice| choice.delta.reasoning_content.as_ref())
                    })
                    .cloned()
                    .unwrap_or_default();

                Ok(content)
            },
        )))
    }
}

impl Default for StreamingResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a Server-Sent Events (SSE) line into a streaming chunk
pub fn parse_sse_line(line: &str) -> ZaiResult<Option<StreamingChunk>> {
    if line.trim().is_empty() {
        return Ok(None);
    }

    if let Some(data) = line.strip_prefix("data: ") {
        // Skip "data: "

        if data == "[DONE]" {
            return Ok(None);
        }

        let chunk: StreamingChunk = serde_json::from_str(data)
            .map_err(|e| ZaiError::invalid_response(format!("Failed to parse SSE chunk: {e}")))?;

        Ok(Some(chunk))
    } else {
        // Ignore non-data lines (like "event: message")
        Ok(None)
    }
}

/// Create a streaming response from SSE text
pub fn create_sse_stream(sse_text: &str) -> StreamingResponse {
    let lines = sse_text.lines();

    let chunks: Vec<_> = lines
        .filter_map(|line| parse_sse_line(line).ok())
        .flatten()
        .collect();

    StreamingResponseBuilder::new().add_chunks(chunks).build()
}

/// Create a streaming response from a single text chunk
pub fn create_text_stream(text: &str) -> StreamingResponse {
    let chunk = StreamingChunk {
        choices: vec![StreamingChoice {
            index: 0,
            delta: StreamingDelta {
                content: Some(text.to_string()),
                tool_calls: None,
                reasoning_content: None,
            },
            finish_reason: None,
        }],
        usage: None,
    };

    StreamingResponseBuilder::new().add_chunk(chunk).build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sse_line_valid() {
        let line = "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello\"}}]}";
        let chunk = parse_sse_line(line).unwrap().unwrap();
        assert_eq!(chunk.choices[0].delta.content, Some("Hello".to_string()));
    }

    #[test]
    fn test_parse_sse_line_empty() {
        let line = "";
        assert!(parse_sse_line(line).unwrap().is_none());
    }

    #[test]
    fn test_parse_sse_line_done() {
        let line = "data: [DONE]";
        assert!(parse_sse_line(line).unwrap().is_none());
    }

    #[test]
    fn test_create_text_stream() {
        let stream = create_text_stream("Hello, world!");
        // In a real test, we would collect the stream and verify the content
        // For now, we just verify it doesn't panic
        drop(stream);
    }
}
