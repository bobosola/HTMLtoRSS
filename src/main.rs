//! HTMLtoRSS - Convert HTML content to RSS feed items
//!
//! A command-line tool for extracting content from HTML files and adding it to RSS feeds.

use clap::Parser;
use regex::Regex;
use scraper::{Html, Selector};
use std::fs;
use url::Url;
use uuid::Uuid;
use chrono::{Utc, Datelike, Timelike};

/// Command line arguments for HTMLtoRSS
#[derive(Parser, Debug)]
#[clap(name = "HTMLtoRSS", version = "0.1.0", author = "bobosola@gmail.com")]
struct Args {
    /// Path to the HTML file or URL to read
    #[clap(long = "html", short = 'l')]
    html: String,

    /// Path to the RSS file to update
    #[clap(long = "rss", short = 'r')]
    rss: String,

    /// Base URL for converting relative URLs to absolute
    #[clap(long = "base-url", short = 'b')]
    base_url: String,

    /// CSS selector to extract HTML content
    #[clap(long = "selector", short = 's', default_value = "main")]
    selector: String,

    /// Title for the RSS item (defaults to first <h1> text)
    #[clap(long = "title", short = 't')]
    title: Option<String>,

    /// Number of lines to cut from the beginning of HTML text (defaults to 0)
    #[clap(long = "lines-to-cut", short = 'c')]
    lines_to_cut: Option<usize>,

    /// Dry run mode - only display output to terminal
    #[clap(long = "dry-run")]
    dry_run: bool,
}

/// Process HTML content and convert it to RSS item format
fn process_html_content(
    html_content: &str,
    base_url: &str,
    selector: &str,
    title: Option<&String>,
    lines_to_cut: Option<usize>,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let document = Html::parse_document(html_content);

    // Find the selector
    let selector_obj = Selector::parse(selector).map_err(|_| "Invalid CSS selector")?;
    let element = document
        .select(&selector_obj)
        .next()
        .ok_or("Selector not found in HTML")?;

    // Get the inner HTML content
    let mut html_content = element.html();

    // Cut lines if specified
    if let Some(lines_to_cut) = lines_to_cut {
        let mut lines: Vec<&str> = html_content.lines().collect();
        if lines_to_cut < lines.len() && lines_to_cut > 0{
            lines.drain(..lines_to_cut);
            html_content = lines.join("\n");
        }
    }

    // Clean up whitespace
    let re_whitespace = Regex::new(r"\s+")?;
    html_content = re_whitespace.replace_all(&html_content, " ").to_string();

    // Extract title from first h1 if not provided
    let item_title = match title {
        Some(t) => t.clone(),
        None => {
            // Find first h1 element
            let h1_selector = Selector::parse("h1").map_err(|_| "Invalid H1 selector")?;
            if let Some(h1_element) = document.select(&h1_selector).next() {
                h1_element.text().collect::<Vec<_>>().join(" ")
            } else {
                "Untitled".to_string()
            }
        }
    };

    // Convert relative URLs to absolute
    let mut processed_html = html_content;

    // Ensure base URL has trailing slash for proper joining
    let normalized_base_url = if base_url.ends_with('/') {
        base_url.to_string()
    } else {
        format!("{}/", base_url)
    };

    let base_url_obj = Url::parse(&normalized_base_url)?;

    // Process src, href and srcset attributes
    let re_src = Regex::new(r#"src\s*=\s*"([^"]*)""#)?;
    let re_href = Regex::new(r#"href\s*=\s*"([^"]*)""#)?;
    let re_srcset = Regex::new(r#"srcset\s*=\s*"([^"]*)""#)?;

    // Process src attributes
    processed_html = re_src.replace_all(&processed_html, |caps: &regex::Captures| {
        let attr_value = &caps[1];
        if !attr_value.starts_with("http") {
            let absolute_url = base_url_obj.join(attr_value).unwrap_or_else(|_| Url::parse(&format!("{}{}", normalized_base_url, attr_value)).unwrap());
            format!("src=\"{}\"", absolute_url)
        } else {
            caps[0].to_string()
        }
    }).to_string();

    // Process href attributes
    processed_html = re_href.replace_all(&processed_html, |caps: &regex::Captures| {
        let attr_value = &caps[1];
        if !attr_value.starts_with("http") {
            let absolute_url = base_url_obj.join(attr_value).unwrap_or_else(|_| Url::parse(&format!("{}{}", normalized_base_url, attr_value)).unwrap());
            format!("href=\"{}\"", absolute_url)
        } else {
            caps[0].to_string()
        }
    }).to_string();

    // Process srcset attributes
    processed_html = re_srcset.replace_all(&processed_html, |caps: &regex::Captures| {
        let attr_value = &caps[1];
        // Handle multiple URLs in srcset
        let urls: Vec<&str> = attr_value.split(',').map(|s| s.trim()).collect();
        let processed_urls: Vec<String> = urls.iter().map(|url| {
            if !url.starts_with("http") {
                let absolute_url = base_url_obj.join(url).unwrap_or_else(|_| Url::parse(&format!("{}{}", normalized_base_url, url)).unwrap());
                absolute_url.to_string()
            } else {
                url.to_string()
            }
        }).collect();

        format!("srcset=\"{}\"", processed_urls.join(", "))
    }).to_string();

    Ok((item_title, processed_html))
}

/// Escape XML special characters
fn escape_xml(text: &str) -> String {
    text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&apos;")
}

/// Format the date according to RSS specification
fn format_rss_date() -> String {
    let now = Utc::now();
    format!(
        "{}, {:02} {} {}:{:02}:{:02} GMT",
        ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"][now.weekday().num_days_from_monday() as usize],
        now.day(),
        ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"][now.month0() as usize],
        now.year(),
        now.hour(),
        now.minute()
    )
}

/// Generate RSS item XML
fn generate_rss_item(
    title: &str,
    description_html: &str,
    base_url: &str,
    html_path: &str,
) -> String {
    let guid = Uuid::new_v4().to_string();
    let pub_date = format_rss_date();

    // Escape XML characters in title
    let escaped_title = escape_xml(title);

    format!(
        r#"<item>
    <title>{}</title>
    <link>{}/{}</link>
    <description><![CDATA[{}]]></description>
    <pubDate>{}</pubDate>
    <guid>{}</guid>
</item>"#,
        escaped_title,
        base_url.trim_end_matches('/'),
        html_path.trim_start_matches('/'),
        description_html,
        pub_date,
        guid
    )
}

/// Main function to run the application
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Check if HTML is a URL or file
    let html_content = if args.html.starts_with("http://") || args.html.starts_with("https://") {
        // It's a URL, fetch it
        let client = reqwest::blocking::Client::new();
        client.get(&args.html).send()?.text()?
    } else {
        // It's a file path
        fs::read_to_string(&args.html)?
    };

    // Process HTML content to extract title and description
    let (title, description_html) = process_html_content(
        &html_content,
        &args.base_url,
        &args.selector,
        args.title.as_ref(),
        args.lines_to_cut,
    )?;

    // If in dry run mode, print to terminal and exit
    if args.dry_run {
        println!("=== DRY RUN MODE ===");
        println!("Title: {}", title);
        println!("Description (HTML): {}", description_html);
        println!("Base URL: {}", args.base_url);
        println!("Selector used: {}", args.selector);
        if let Some(lines) = args.lines_to_cut {
            println!("Lines to cut: {}", lines);
        }
        if let Some(t) = args.title {
            println!("Title override: {}", t);
        }
        return Ok(());
    }

    // Read the existing RSS file
    let rss_content = fs::read_to_string(&args.rss)?;

    // Find the position of </channel> tag to insert our item
    let channel_end = rss_content.rfind("</channel>")
        .ok_or("Could not find </channel> tag in RSS file")?;

    // Generate the new RSS item
    let rss_item = generate_rss_item(&title, &description_html, &args.base_url, &args.html);

    // Insert the new item before </channel>
    let mut updated_rss = rss_content[..channel_end].to_string();
    updated_rss.push_str("\n    ");
    updated_rss.push_str(&rss_item);
    updated_rss.push_str("\n    </channel>");

    // Write the updated RSS file
    fs::write(&args.rss, updated_rss)?;

    println!("RSS item successfully added to {}", args.rss);
    Ok(())
}
