use crate::display;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use x_http::error::Result;
use x_http::{Method, Request};

pub struct InteractiveSession;

impl InteractiveSession {
    pub fn run() -> Result<()> {
        println!("x-http Interactive Mode");
        println!("Press Ctrl+C to exit\n");

        loop {
            match Self::prompt_and_execute() {
                Ok(true) => {}
                Ok(false) => break,
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }

        Ok(())
    }

    fn prompt_and_execute() -> Result<bool> {
        let method = Self::prompt_method()?;
        let url = Self::prompt_url()?;
        let headers = Self::prompt_headers()?;
        let body = if matches!(method, Method::Post | Method::Put | Method::Patch) {
            Self::prompt_body()?
        } else {
            None
        };

        let mut request = Request::new(method, url);

        for (key, value) in headers {
            request = request.header(key, value);
        }

        if let Some((body_str, is_json)) = body {
            if is_json {
                let json_value: serde_json::Value = serde_json::from_str(&body_str)?;
                request = request.json(&json_value)?;
            } else {
                request = request.text(body_str);
            }
        }

        println!("\n⏳ Sending request...\n");

        let response = request.send()?;
        display::display_response(&response)?;

        let continue_prompt: bool = dialoguer::Confirm::new()
            .with_prompt("\nMake another request?")
            .default(true)
            .interact()?;

        Ok(continue_prompt)
    }

    fn prompt_method() -> Result<Method> {
        let methods = vec!["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select HTTP method")
            .items(&methods)
            .default(0)
            .interact()?;

        Ok(match selection {
            0 => Method::Get,
            1 => Method::Post,
            2 => Method::Put,
            3 => Method::Delete,
            4 => Method::Patch,
            5 => Method::Head,
            6 => Method::Options,
            _ => Method::Get,
        })
    }

    fn prompt_url() -> Result<String> {
        let url: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("URL")
            .interact_text()?;
        Ok(url)
    }

    fn prompt_headers() -> Result<Vec<(String, String)>> {
        let mut headers = Vec::new();

        loop {
            let header: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Header (key:value, or press Enter to skip)")
                .allow_empty(true)
                .interact_text()?;

            if header.is_empty() {
                break;
            }

            if let Some((key, value)) = header.split_once(':') {
                headers.push((key.trim().to_string(), value.trim().to_string()));
            } else {
                eprintln!("Invalid header format. Use key:value");
            }
        }

        Ok(headers)
    }

    fn prompt_body() -> Result<Option<(String, bool)>> {
        let has_body: bool = dialoguer::Confirm::new()
            .with_prompt("Include request body?")
            .default(false)
            .interact()?;

        if !has_body {
            return Ok(None);
        }

        let is_json: bool = dialoguer::Confirm::new()
            .with_prompt("Is the body JSON?")
            .default(true)
            .interact()?;

        let body: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(if is_json { "JSON body" } else { "Body" })
            .interact_text()?;

        Ok(Some((body, is_json)))
    }
}
