use clap::{Parser, Subcommand};
use x_http::error::Result;

mod config;
mod display;
mod interactive;

use interactive::InteractiveSession;

#[derive(Parser)]
#[command(name = "x-http")]
#[command(version, about = "Instant HTTP API testing suite", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Interactive,

    Run {
        #[arg(short, long, default_value = "x-http.toml")]
        config: String,

        #[arg(short, long)]
        name: Option<String>,
    },

    Request {
        method: String,

        url: String,

        #[arg(short = 'H', long)]
        header: Vec<String>,

        #[arg(short, long)]
        body: Option<String>,

        #[arg(short, long)]
        json: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Interactive) | None => {
            InteractiveSession::run()?;
        }
        Some(Commands::Run { config, name }) => {
            config::run_from_config(&config, name.as_deref())?;
        }
        Some(Commands::Request {
            method,
            url,
            header,
            body,
            json,
        }) => {
            quick_request(&method, &url, &header, body.as_deref(), json)?;
        }
    }

    Ok(())
}

fn quick_request(
    method: &str,
    url: &str,
    headers: &[String],
    body: Option<&str>,
    is_json: bool,
) -> Result<()> {
    use x_http::{Method, Request};

    let method = match method.to_uppercase().as_str() {
        "GET" => Method::Get,
        "POST" => Method::Post,
        "PUT" => Method::Put,
        "DELETE" => Method::Delete,
        "PATCH" => Method::Patch,
        "HEAD" => Method::Head,
        "OPTIONS" => Method::Options,
        _ => {
            eprintln!("Invalid method: {}", method);
            std::process::exit(1);
        }
    };

    let mut request = Request::new(method, url);

    for header in headers {
        if let Some((key, value)) = header.split_once(':') {
            request = request.header(key.trim(), value.trim());
        }
    }

    if let Some(body_str) = body {
        if is_json {
            let json_value: serde_json::Value = serde_json::from_str(body_str)?;
            request = request.json(&json_value)?;
        } else {
            request = request.text(body_str);
        }
    }

    let response = request.send()?;
    display::display_response(&response)?;

    Ok(())
}
