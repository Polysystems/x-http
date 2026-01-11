use colored::Colorize;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};
use x_http::error::Result;
use x_http::Response;

pub fn display_response(response: &Response) -> Result<()> {
    println!("{}", "━".repeat(80).bright_blue());
    println!("{} {}", "Status:".bold(), format_status(response.status()));
    println!("{} {:?}", "Duration:".bold(), response.duration());

    println!("\n{}", "Headers:".bold().cyan());
    for (key, value) in response.headers() {
        println!(
            "  {}: {}",
            key.as_str().green(),
            value.to_str().unwrap_or("<binary>")
        );
    }

    if let Ok(text) = response.text() {
        println!("\n{}", "Body:".bold().cyan());

        if let Some(content_type) = response.header("content-type") {
            if content_type.contains("application/json") {
                display_json(&text)?;
            } else {
                println!("{}", text);
            }
        } else {
            println!("{}", text);
        }
    } else {
        println!("\n{}", "Body: <binary data>".dimmed());
    }

    println!("{}", "━".repeat(80).bright_blue());

    Ok(())
}

fn format_status(status: u16) -> String {
    let status_str = status.to_string();
    if (200..300).contains(&status) {
        status_str.green().to_string()
    } else if (300..400).contains(&status) {
        status_str.yellow().to_string()
    } else if status >= 400 {
        status_str.red().to_string()
    } else {
        status_str.white().to_string()
    }
}

fn display_json(json_text: &str) -> Result<()> {
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_text) {
        let formatted = serde_json::to_string_pretty(&parsed)?;

        if let Some(highlighted) = highlight_json(&formatted) {
            println!("{}", highlighted);
        } else {
            println!("{}", formatted);
        }
    } else {
        println!("{}", json_text);
    }

    Ok(())
}

fn highlight_json(json_text: &str) -> Option<String> {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = ps.find_syntax_by_extension("json")?;
    let theme = &ts.themes["base16-ocean.dark"];

    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut output = String::new();

    for line in LinesWithEndings::from(json_text) {
        let ranges = highlighter.highlight_line(line, &ps).ok()?;
        let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
        output.push_str(&escaped);
    }

    Some(output)
}
