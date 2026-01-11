use crate::error::{Error, Result};
use reqwest::blocking::Response as ReqwestResponse;
use reqwest::header::HeaderMap;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::time::Duration;

#[derive(Debug)]
pub struct Response {
    status: StatusCode,
    headers: HeaderMap,
    body: Vec<u8>,
    duration: Duration,
}

impl Response {
    pub(crate) fn from_reqwest(response: ReqwestResponse, duration: Duration) -> Result<Self> {
        let status = response.status();
        let headers = response.headers().clone();
        let body = response.bytes()?.to_vec();

        Ok(Self {
            status,
            headers,
            body,
            duration,
        })
    }

    pub fn status(&self) -> u16 {
        self.status.as_u16()
    }

    pub fn status_code(&self) -> StatusCode {
        self.status
    }

    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }

    pub fn is_error(&self) -> bool {
        self.status.is_client_error() || self.status.is_server_error()
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers.get(key)?.to_str().ok()
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn body_bytes(&self) -> &[u8] {
        &self.body
    }

    pub fn text(&self) -> Result<String> {
        String::from_utf8(self.body.clone())
            .map_err(|e| Error::Assertion(format!("Response body is not valid UTF-8: {}", e)))
    }

    pub fn json<T: DeserializeOwned>(&self) -> Result<T> {
        let text = self.text()?;
        serde_json::from_str(&text).map_err(|e| e.into())
    }

    pub fn json_value(&self) -> Result<Value> {
        self.json()
    }

    // Assertion methods - chainable
    pub fn expect_status(self, expected: u16) -> Result<Self> {
        let actual = self.status();
        if actual != expected {
            return Err(Error::StatusMismatch { expected, actual });
        }
        Ok(self)
    }

    pub fn expect_success(self) -> Result<Self> {
        if !self.is_success() {
            return Err(Error::Assertion(format!(
                "Expected success status, got {}",
                self.status()
            )));
        }
        Ok(self)
    }

    pub fn expect_error(self) -> Result<Self> {
        if !self.is_error() {
            return Err(Error::Assertion(format!(
                "Expected error status, got {}",
                self.status()
            )));
        }
        Ok(self)
    }

    pub fn expect_json(self) -> Result<Self> {
        let content_type = self.header("content-type").unwrap_or("unknown");

        if !content_type.contains("application/json") {
            return Err(Error::NotJson(content_type.to_string()));
        }

        self.json_value()?;
        Ok(self)
    }

    pub fn expect_text(self) -> Result<Self> {
        self.text()?;
        Ok(self)
    }

    pub fn expect_body_contains(self, text: &str) -> Result<Self> {
        let body = self.text()?;
        if !body.contains(text) {
            return Err(Error::Assertion(format!(
                "Expected body to contain '{}', but it didn't",
                text
            )));
        }
        Ok(self)
    }

    pub fn expect_header(self, key: &str, expected: &str) -> Result<Self> {
        let actual = self
            .header(key)
            .ok_or_else(|| Error::Assertion(format!("Header '{}' not found", key)))?;

        if actual != expected {
            return Err(Error::HeaderMismatch {
                key: key.to_string(),
                expected: expected.to_string(),
                actual: actual.to_string(),
            });
        }
        Ok(self)
    }

    pub fn expect_content_type(self, content_type: &str) -> Result<Self> {
        self.expect_header("content-type", content_type)
    }

    pub fn assert_field(self, path: &str, expected: impl Into<Value>) -> Result<Self> {
        let json = self.json_value()?;
        let expected_value = expected.into();

        let actual_value = extract_json_path(&json, path).ok_or_else(|| Error::PathNotFound {
            path: path.to_string(),
        })?;

        if actual_value != &expected_value {
            return Err(Error::FieldMismatch {
                field: path.to_string(),
                expected: expected_value.to_string(),
                actual: actual_value.to_string(),
            });
        }

        Ok(self)
    }

    pub fn assert_field_exists(self, path: &str) -> Result<Self> {
        let json = self.json_value()?;

        extract_json_path(&json, path).ok_or_else(|| Error::PathNotFound {
            path: path.to_string(),
        })?;

        Ok(self)
    }

    pub fn assert_array_length(self, path: &str, expected_length: usize) -> Result<Self> {
        let json = self.json_value()?;

        let array = extract_json_path(&json, path)
            .and_then(|v| v.as_array())
            .ok_or_else(|| Error::Assertion(format!("Path '{}' is not an array", path)))?;

        if array.len() != expected_length {
            return Err(Error::Assertion(format!(
                "Array at '{}' expected length {}, got {}",
                path,
                expected_length,
                array.len()
            )));
        }

        Ok(self)
    }
}

fn extract_json_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for part in parts {
        if let Some(index_start) = part.find('[') {
            let field = &part[..index_start];
            let index_str = &part[index_start + 1..part.len() - 1];
            let index: usize = index_str.parse().ok()?;

            current = current.get(field)?.get(index)?;
        } else {
            current = current.get(part)?;
        }
    }

    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_path_extraction() {
        let json = json!({
            "user": {
                "name": "John",
                "age": 30
            },
            "items": [
                {"id": 1, "name": "First"},
                {"id": 2, "name": "Second"}
            ]
        });

        assert_eq!(extract_json_path(&json, "user.name"), Some(&json!("John")));
        assert_eq!(extract_json_path(&json, "user.age"), Some(&json!(30)));
        assert_eq!(
            extract_json_path(&json, "items[0].name"),
            Some(&json!("First"))
        );
        assert_eq!(extract_json_path(&json, "items[1].id"), Some(&json!(2)));
        assert_eq!(extract_json_path(&json, "nonexistent"), None);
    }
}
