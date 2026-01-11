use crate::error::Result;
use crate::response::Response;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl Method {
    fn as_reqwest_method(&self) -> reqwest::Method {
        match self {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Delete => reqwest::Method::DELETE,
            Method::Patch => reqwest::Method::PATCH,
            Method::Head => reqwest::Method::HEAD,
            Method::Options => reqwest::Method::OPTIONS,
        }
    }
}

#[derive(Debug)]
pub struct Request {
    method: Method,
    url: String,
    headers: HeaderMap,
    body: Option<Vec<u8>>,
    query_params: HashMap<String, String>,
    timeout: Option<Duration>,
    follow_redirects: bool,
}

impl Request {
    pub fn new(method: Method, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            headers: HeaderMap::new(),
            body: None,
            query_params: HashMap::new(),
            timeout: Some(Duration::from_secs(30)),
            follow_redirects: true,
        }
    }

    pub fn get(url: impl Into<String>) -> Self {
        Self::new(Method::Get, url)
    }

    pub fn post(url: impl Into<String>) -> Self {
        Self::new(Method::Post, url)
    }

    pub fn put(url: impl Into<String>) -> Self {
        Self::new(Method::Put, url)
    }

    pub fn delete(url: impl Into<String>) -> Self {
        Self::new(Method::Delete, url)
    }

    pub fn patch(url: impl Into<String>) -> Self {
        Self::new(Method::Patch, url)
    }

    pub fn head(url: impl Into<String>) -> Self {
        Self::new(Method::Head, url)
    }

    pub fn options(url: impl Into<String>) -> Self {
        Self::new(Method::Options, url)
    }

    pub fn header(mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        if let (Ok(name), Ok(val)) = (
            HeaderName::try_from(key.as_ref()),
            HeaderValue::try_from(value.as_ref()),
        ) {
            self.headers.insert(name, val);
        }
        self
    }

    pub fn headers(mut self, headers: Vec<(impl AsRef<str>, impl AsRef<str>)>) -> Self {
        for (key, value) in headers {
            self = self.header(key, value);
        }
        self
    }

    pub fn json<T: Serialize>(mut self, body: &T) -> Result<Self> {
        let json_string = serde_json::to_string(body)?;
        self.body = Some(json_string.into_bytes());
        self = self.header("Content-Type", "application/json");
        Ok(self)
    }

    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn text(self, text: impl Into<String>) -> Self {
        self.body(text.into().into_bytes())
            .header("Content-Type", "text/plain")
    }

    pub fn query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_params.insert(key.into(), value.into());
        self
    }

    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    pub fn no_timeout(mut self) -> Self {
        self.timeout = None;
        self
    }

    pub fn follow_redirects(mut self, follow: bool) -> Self {
        self.follow_redirects = follow;
        self
    }

    pub fn send(self) -> Result<Response> {
        let client = Client::builder()
            .redirect(if self.follow_redirects {
                reqwest::redirect::Policy::default()
            } else {
                reqwest::redirect::Policy::none()
            })
            .build()?;

        let mut url = url::Url::parse(&self.url)?;

        for (key, value) in self.query_params {
            url.query_pairs_mut().append_pair(&key, &value);
        }

        let mut request_builder = client
            .request(self.method.as_reqwest_method(), url)
            .headers(self.headers);

        if let Some(timeout) = self.timeout {
            request_builder = request_builder.timeout(timeout);
        }

        if let Some(body) = self.body {
            request_builder = request_builder.body(body);
        }

        let start = std::time::Instant::now();
        let response = request_builder.send()?;
        let duration = start.elapsed();

        Response::from_reqwest(response, duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_builder() {
        let req = Request::get("https://example.com")
            .header("Authorization", "Bearer token")
            .query("page", "1")
            .timeout(Duration::from_secs(10));

        assert_eq!(req.method, Method::Get);
        assert_eq!(req.url, "https://example.com");
        assert_eq!(req.timeout, Some(Duration::from_secs(10)));
    }

    #[test]
    fn test_json_body() {
        use serde_json::json;

        let body = json!({
            "name": "test",
            "value": 42
        });

        let req = Request::post("https://example.com").json(&body).unwrap();

        assert!(req.body.is_some());
        assert!(req.headers.contains_key("content-type"));
    }
}
