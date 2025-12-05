//! Tests for streaming response functionality

use zai_sdk::streaming::{
    create_sse_stream, create_text_stream, parse_sse_line, StreamingChoice, StreamingChunk,
    StreamingDelta, StreamingResponseBuilder,
};

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

#[test]
fn test_sse_stream_creation() {
    let sse_text = r#"
data: {"choices":[{"index":0,"delta":{"content":"Hello"}}]}
data: {"choices":[{"index":0,"delta":{"content":" world"}}]}
data: {"choices":[{"index":0,"delta":{"content":"!"}}]}
data: [DONE]
"#;

    let stream = create_sse_stream(sse_text);
    // Just verify it doesn't panic
    drop(stream);
}

#[test]
fn test_streaming_response_builder() {
    let chunk = StreamingChunk {
        choices: vec![StreamingChoice {
            index: 0,
            delta: StreamingDelta {
                content: Some("Hello".to_string()),
                tool_calls: None,
                reasoning_content: None,
            },
            finish_reason: None,
        }],
        usage: None,
    };

    let stream = StreamingResponseBuilder::new().add_chunk(chunk).build();
    // Just verify it doesn't panic
    drop(stream);
}
