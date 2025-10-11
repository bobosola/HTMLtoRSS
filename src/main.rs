extern crate clipboard;
extern crate escaper;

use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;
use regex::Regex;
use escaper::{encode_minimal};
use chrono::Utc;
use std::fs;
use std::path::Path;
use uuid::Uuid;

fn main() {

    const LINES_TO_CUT: usize = 3;  // cuts first n lines of text
    const BASE_URL: &str = "https://osola.org.uk/blog/";

    // Get the HTML file path & article title from command line arguments
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <html-file-path> <title>", args[0]);
        std::process::exit(1);
    }

    let html_file_path = &args[1];  // the HTML file name
    // Get the current working directory
    let cwd = fs::canonicalize(Path::new(".")).unwrap();
    let full_file_path = cwd.join(html_file_path);

    let title = encode_minimal(&args[2]);  // the feed item title
    let full_url = format!("{}{}", BASE_URL, html_file_path);

    // Read the HTML file
    let html_content = match fs::read_to_string(&full_file_path) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Error: Could not read file '{:?}'", &full_file_path);
            std::process::exit(1);
        }
    };

    // Extract text between <main> and </main> tags and cut the first n lines
    let main_content = extract_main_content(&html_content, LINES_TO_CUT);

    // Convert relative URLs to absolute ones
    let processed_content = convert_relative_urls(&main_content, BASE_URL);

    // Remove extraneous whitespace
    let cleaned_content = remove_extraneous_whitespace(&processed_content);

    let rss_item = generate_rss_item(&title, &full_url, &cleaned_content);

    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    ctx.set_contents(rss_item.to_owned()).unwrap();
    println!("✅ Copied to clipboard!");
}

fn generate_rss_item(
    title: &str,
    url: &str,
    description_text: &str
) -> String {
    format!(
        r#"<item>
    <title>{}</title>
    <link>{}</link>
    <description><![CDATA[
        {}
    ]]></description>
    <pubDate>{}</pubDate>
    <guid>{}</guid>
</item>"#,
        title,
        url,
        description_text,
        Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string(),
        Uuid::new_v4()
    )
}

fn extract_main_content(html: &str, cut_lines: usize) -> String {
    let main_start = "<main>";
    let main_end = "</main>";

    // Find the start and end positions of <main> tag
    let start_pos = html.find(main_start);

    if let Some(start) = start_pos {
        // Find the end of main content
        let end_pos = html.find(main_end).unwrap_or_else(|| html.len());

        // Extract content between main tags (excluding the tags themselves)
        let mut result = String::new();

        // Add everything from after the main tag to before the closing tag
        result.push_str(&html[start + main_start.len()..end_pos]);

        // Split into lines and skip first 3 lines
        let mut lines: Vec<&str> = result.lines().collect();
        if lines.len() > cut_lines {
            lines.drain(0..cut_lines);
            result = lines.join("\n");
        } else {
            result = String::new();
        }

        return result;
    }
    // If no <main> tag found, return empty string
    String::new()
}

fn convert_relative_urls(content: &str, base_url: &str) -> String {
    // Regex to match href attributes and src attributes
    let url_regex = Regex::new(r#"(href|src|srcset)=\"([^\"]*)\""#).unwrap();

    // Replace matches with absolute URLs
    url_regex.replace_all(content, |caps: &regex::Captures| {
        let attribute = caps.get(1).unwrap().as_str();
        let value = caps.get(2).unwrap().as_str();

        // Only convert if it's not already a full URL
        if !value.starts_with("http://") && !value.starts_with("https://") {
            // Convert relative path to absolute URL
            format!("{}=\"{}{}\"", attribute, base_url, value)
        } else {
            // Keep original full URL
            format!("{}=\"{}\"", attribute, value)
        }
    }).to_string()
}

fn remove_extraneous_whitespace(content: &str) -> String {
    // Replace multiple whitespace characters with single spaces
    let whitespace_regex = Regex::new(r"\s+").unwrap();

    // Replace with single space, then trim
    let result = whitespace_regex.replace_all(content, " ").to_string();

    // Trim leading/trailing whitespace
    result.trim().to_string()
}
