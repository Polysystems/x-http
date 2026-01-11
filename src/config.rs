use crate::display;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use x_http::error::{Error, Result};
use x_http::{Method, Request};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub variables: HashMap<String, String>,
    #[serde(default)]
    pub requests: Vec<RequestConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestConfig {
    pub name: String,
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    #[serde(default)]
    pub json: bool,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("Failed to read config file: {}", e)))?;

        toml::from_str(&contents)
            .map_err(|e| Error::Config(format!("Failed to parse config file: {}", e)))
    }

    pub fn substitute_variables(&self, text: &str) -> String {
        let mut result = text.to_string();
        for (key, value) in &self.variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    }
}

pub fn run_from_config(config_path: &str, request_name: Option<&str>) -> Result<()> {
    let config = Config::load(config_path)?;

    let requests_to_run: Vec<&RequestConfig> = if let Some(name) = request_name {
        config.requests.iter().filter(|r| r.name == name).collect()
    } else {
        config.requests.iter().collect()
    };

    if requests_to_run.is_empty() {
        return Err(Error::Config(format!(
            "No requests found{}",
            request_name.map_or(String::new(), |n| format!(" with name '{}'", n))
        )));
    }

    for request_config in requests_to_run {
        println!("\n🚀 Running: {}", request_config.name);
        execute_request_config(&config, request_config)?;
    }

    Ok(())
}

fn execute_request_config(config: &Config, request_config: &RequestConfig) -> Result<()> {
    let method = parse_method(&request_config.method)?;
    let url = config.substitute_variables(&request_config.url);

    let mut request = Request::new(method, url);

    for (key, value) in &request_config.headers {
        let substituted_value = config.substitute_variables(value);
        request = request.header(key, substituted_value);
    }

    if let Some(body) = &request_config.body {
        let substituted_body = config.substitute_variables(body);
        if request_config.json {
            let json_value: serde_json::Value = serde_json::from_str(&substituted_body)?;
            request = request.json(&json_value)?;
        } else {
            request = request.text(substituted_body);
        }
    }

    let response = request.send()?;
    display::display_response(&response)?;

    Ok(())
}

fn parse_method(method: &str) -> Result<Method> {
    match method.to_uppercase().as_str() {
        "GET" => Ok(Method::Get),
        "POST" => Ok(Method::Post),
        "PUT" => Ok(Method::Put),
        "DELETE" => Ok(Method::Delete),
        "PATCH" => Ok(Method::Patch),
        "HEAD" => Ok(Method::Head),
        "OPTIONS" => Ok(Method::Options),
        _ => Err(Error::Config(format!("Invalid HTTP method: {}", method))),
    }
}
