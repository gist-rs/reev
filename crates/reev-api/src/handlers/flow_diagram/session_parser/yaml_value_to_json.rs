//! YAML to JSON Value Conversion Module
//!
//! This module provides utility functions for converting YAML values to JSON values.

use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;

/// Convert YAML Value to JSON Value
pub fn yaml_value_to_json(yaml_value: &YamlValue) -> JsonValue {
    match yaml_value {
        YamlValue::String(s) => JsonValue::String(s.clone()),
        YamlValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                JsonValue::Number(i.into())
            } else if let Some(f) = n.as_f64() {
                JsonValue::Number(serde_json::Number::from_f64(f).unwrap_or(0.into()))
            } else {
                JsonValue::Null
            }
        }
        YamlValue::Bool(b) => JsonValue::Bool(*b),
        YamlValue::Null => JsonValue::Null,
        YamlValue::Sequence(seq) => JsonValue::Array(seq.iter().map(yaml_value_to_json).collect()),
        YamlValue::Mapping(map) => {
            let mut json_map = serde_json::Map::new();
            for (k, v) in map {
                if let Some(key_str) = k.as_str() {
                    json_map.insert(key_str.to_string(), yaml_value_to_json(v));
                }
            }
            JsonValue::Object(json_map)
        }
        // Handle YAML tags and other complex types as strings
        _ => JsonValue::String(format!("{yaml_value:?}")),
    }
}
