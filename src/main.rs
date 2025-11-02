//! HTMLtoRSS - Convert HTML content to RSS feed items
//!
//! A command-line tool for extracting content from HTML files and adding it to RSS feeds.

use clap::Parser;
use regex::Regex;
use scraper::{Html, Selector};
use std::fs;

mod utils;

/// Command line arguments for HTMLtoRSS
#[derive(Parser, Debug)]
#[clap(name = "HTMLtoRSS", version = "0.1.0", author = "bobosola@gmail.com")]
struct Args {
    /// Path to the HTML file or URL to read
    #[clap(long = "html", short = 'f', help = "Relative path to HTML file or URL of a website page")]
    html: String,

    /// Path to the RSS file to update
    #[clap(long = "rss", short = 'r', help = "Relative path to your rss.xml file")]
    rss: String,

    /// Base URL for converting relative URLs to absolute
    #[clap(long = "parent-url", short = 'b', help = "Parent URL to convert relative src etc. values")]
    parent_url: String,

    /// CSS selector to extract HTML content
    #[clap(long = "selector", short = 's', default_value = "main", help = "Optional CSS selector for content")]
    selector: String,

    /// Title for the RSS item (defaults to first <h1> text)
    #[clap(long = "title", short = 't', help = "Optional title else first <h1> text is used")]
    title: Option<String>,

    /// Datetime for the item (defaults to the time the application is run)
    #[clap(long = "date-time", short = 'd', default_value = "now", help = "Optional datetime e.g. '2021-06-02 14:30'")]
    date_time: String,

    /// Number of lines to cut from the beginning of HTML text (defaults to 0)
    #[clap(long = "lines-to-cut", short = 'c', default_value = "0", help = "Optional lines to cut")]
    lines_to_cut: usize,

    /// Dry run mode - only display output to terminal
    #[clap(long = "dry-run")]
    dry_run: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args = Args::parse();

    // Get the content of the HTML file, either from a URL or a local file path
    let html_content = if args.html.starts_with("http://") || args.html.starts_with("https://") {
        // It's a URL so fetch it
        let client = reqwest::blocking::Client::new();
        client.get(&args.html).send()?.text()?
    } else {
        // Read the local file
        fs::read_to_string(&args.html)?
    };

    // Process the file's HTML content to extract the
    // RSS item's <title> and <description> elements
    // (NB: the <description> element holds the HTML page content)
    let (item_title, item_description) = process_html_content(
        &html_content,
        &args.parent_url,
        &args.selector,
        args.title.as_ref(),
        args.lines_to_cut,
    )?;

    // Get the user-supplied date or else use now
    // and convert to RFC 2822 to match RSS spec

    let pub_date = if args.date_time == "now" {
        utils::now_rfc2822()
    }
    else {
        match utils::parse_to_rfc2822(&args.date_time){
            Ok(d_rfc) => d_rfc,
            Err(_) => "INVALID DATE ENTERED".to_string()
        }
    };

        // Generate the new RSS item
    let rss_item = generate_rss_item(
        &item_title,
        &item_description,
        &args.parent_url,
        &args.html,
        &pub_date
    )?;

    // If in dry run mode, print item to terminal and exit
    if args.dry_run {
        println!("=== DRY RUN MODE ===");
        println!("Title: {}", item_title);
        println!("Base URL: {}", args.parent_url);
        println!("Selector used: {}", args.selector);
        if args.lines_to_cut > 0 {
            println!("Lines to cut: {}", args.lines_to_cut);
        }
        if let Some(t) = args.title {
            println!("Title override: {}", t);
        }
        println!("RSS Item:");
        println!("{}", rss_item);
        return Ok(());
    }

    // Insert the new item at the end of the </channel> element in the rss.xml file
    let place_before = "</channel>";
    match utils::insert_before_text(&args.rss, &place_before, &rss_item) {
        Ok(_) => {
            println!("RSS item successfully added to {}", args.rss)
        },
        Err(e) => println!("Error writing to rss.xml file: {}", e)
    };
    Ok(())
}

/// Process HTML content and convert it to RSS item format
fn process_html_content(
    html_content: &str,
    base_url: &str,
    selector: &str,
    title: Option<&String>,
    lines_to_cut: usize,
) -> Result<(String, String), Box<dyn std::error::Error>> {

    let document = Html::parse_document(html_content);

    // Find the selector
    let selector_obj = Selector::parse(selector).map_err(|_| "Invalid CSS selector")?;
    let element = document
        .select(&selector_obj)
        .next()
        .ok_or("Selector not found in HTML")?;

    // Get the inner HTML content
    let mut html_content = element.inner_html();

    // Cut lines if specified
    if lines_to_cut > 0 {
        let mut lines: Vec<&str> = html_content.lines().collect();
        if lines_to_cut < lines.len() {
            lines.drain(..lines_to_cut);
            html_content = lines.join("\n");
        }
    }

    // Clean up whitespace
    let re_whitespace = Regex::new(r"\s+")?;
    html_content = re_whitespace.replace_all(&html_content, " ").to_string();

    // Extract title from first h1 if not provided as an arg
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

    // Convert any relative URLs to absolute
    let mut processed_html = html_content;

    // Process src, href and srcset attributes
    let re_src = Regex::new(r#"src\s*=\s*"([^"]*)""#)?;
    let re_href = Regex::new(r#"href\s*=\s*"([^"]*)""#)?;
    let re_srcset = Regex::new(r#"srcset\s*=\s*"([^"]*)""#)?;

    // Process src attributes
    processed_html = re_src.replace_all(&processed_html, |caps: &regex::Captures| {
        let attr_value = &caps[1];
        if !attr_value.starts_with("http") {
            //let absolute_url = base_url_obj.join(attr_value).unwrap_or_else(|_| Url::parse(&format!("{}{}", normalized_base_url, attr_value)).unwrap());
            let absolute_url = utils::merge_url_and_fragment(base_url, attr_value).unwrap();
            format!("src=\"{}\"", absolute_url)
        } else {
            caps[0].to_string()
        }
    }).to_string();

    // Process href attributes
    processed_html = re_href.replace_all(&processed_html, |caps: &regex::Captures| {
        let attr_value = &caps[1];
        if !attr_value.starts_with("http") {
            //let absolute_url = base_url_obj.join(attr_value).unwrap_or_else(|_| Url::parse(&format!("{}{}", normalized_base_url, attr_value)).unwrap());
            let absolute_url = utils::merge_url_and_fragment(base_url, attr_value).unwrap();
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
                //let absolute_url = base_url_obj.join(url).unwrap_or_else(|_| Url::parse(&format!("{}{}", normalized_base_url, url)).unwrap());
                let absolute_url = utils::merge_url_and_fragment(base_url, attr_value).unwrap();
                absolute_url.to_string()
            } else {
                url.to_string()
            }
        }).collect();

        format!("srcset=\"{}\"", processed_urls.join(", "))
    }).to_string();

    Ok((item_title, processed_html))
}

/// Generate RSS item XML
fn generate_rss_item(
    title: &str,
    description_html: &str,
    base_url: &str,
    html_path: &str,
    date_time: &str
) -> Result<String, url::ParseError> {

    // Construct the <link> element as a URL to the item's web page
    // (NB: this is also used as the <guid> element as per RSS spec)
    let link = match html_path.starts_with("http") {
        // it's a URL to a remote site page, so no merging required
        true => html_path.to_owned(),
        // Its a local file path, so merge with the base URL
        false => {
            // Avoid getting incorrect concatenation of "https://site/blog" & "blog/page.html"
            // as "https://site/blog/blog/page.html"
            let trimmed_url = utils::remove_last_segment_from_url(base_url)?;
            utils::merge_url_and_fragment(&trimmed_url, html_path)?
        }
    };

    let escaped_title = utils::escape_xml(title);

    Ok(format!(r#"    <item>
           <title>{}</title>
            <link>{}</link>
            <description><![CDATA[{}]]>
            </description>
            <pubDate>{}</pubDate>
            <guid>{}</guid>
        </item>
    "#,
        escaped_title,
        link,
        description_html,
        date_time,
        link
    ))
}
