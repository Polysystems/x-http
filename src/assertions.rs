use serde_json::Value;

pub fn json_values_match(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::String(a), Value::String(e)) => a == e,
        (Value::Number(a), Value::Number(e)) => a == e,
        (Value::Bool(a), Value::Bool(e)) => a == e,
        (Value::Null, Value::Null) => true,
        (Value::Array(a), Value::Array(e)) => {
            a.len() == e.len() && a.iter().zip(e.iter()).all(|(a, e)| json_values_match(a, e))
        }
        (Value::Object(a), Value::Object(e)) => {
            a.len() == e.len()
                && a.iter()
                    .all(|(k, v)| e.get(k).is_some_and(|ev| json_values_match(v, ev)))
        }
        _ => false,
    }
}

pub fn matches_pattern(value: &str, pattern: &str) -> bool {
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.is_empty() {
            return true;
        }

        let mut pos = 0;
        for (i, part) in parts.iter().enumerate() {
            if i == 0 && !part.is_empty() {
                if !value.starts_with(part) {
                    return false;
                }
                pos += part.len();
            } else if i == parts.len() - 1 && !part.is_empty() {
                if !value.ends_with(part) {
                    return false;
                }
            } else if !part.is_empty() {
                if let Some(found_pos) = value[pos..].find(part) {
                    pos += found_pos + part.len();
                } else {
                    return false;
                }
            }
        }
        true
    } else {
        value == pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_values_match() {
        assert!(json_values_match(&json!("test"), &json!("test")));
        assert!(json_values_match(&json!(42), &json!(42)));
        assert!(json_values_match(&json!(true), &json!(true)));
        assert!(json_values_match(&json!(null), &json!(null)));
        assert!(json_values_match(
            &json!({"a": 1, "b": 2}),
            &json!({"a": 1, "b": 2})
        ));
        assert!(json_values_match(&json!([1, 2, 3]), &json!([1, 2, 3])));

        assert!(!json_values_match(&json!("test"), &json!("other")));
        assert!(!json_values_match(&json!(42), &json!(43)));
        assert!(!json_values_match(&json!(true), &json!(false)));
    }

    #[test]
    fn test_matches_pattern() {
        assert!(matches_pattern("hello world", "hello world"));
        assert!(matches_pattern("hello world", "hello*"));
        assert!(matches_pattern("hello world", "*world"));
        assert!(matches_pattern("hello world", "*lo wo*"));
        assert!(matches_pattern("hello world", "hello*world"));

        assert!(!matches_pattern("hello world", "goodbye*"));
        assert!(!matches_pattern("hello world", "*universe"));
    }
}
