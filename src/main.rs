use quick_xml::events::{BytesEnd, BytesStart, BytesText, BytesCData, Event};
use quick_xml::writer::Writer;
use quick_xml::reader::Reader;
use scraper::{Html, Selector};
use std::io::Cursor;
use std::path::Path;
use std::fs;
use uuid::Uuid;
use chrono::Utc;
use regex::Regex;
use url::Url;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    /*************************  ALTER THESE TO SUIT  *************************/
    // The base URL for converting any relative urls to absolute
    const BASE_URL: &str = "https://osola.org.uk/blog";

    // The element which contains the content to be exported as an RSS item
    // e.g. the <main> element but without the '<' and '>'
    const CONTENT_ELEMENT: &str = "main";

    // The number of lines to cut from the start of the content
    // (e.g. unwanted headings etc.)
    const LINES_TO_CUT: usize = 3;
    /************************************************************************/

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        println!("\nUsage: {} <html file> '<item title>' <rss file>", args[0]);
        println!("NB: file paths are relative to current directory\n");
        std::process::exit(1);
    }
    let html_file = &args[1];
    let title = &args[2];
    let rss_file = &args[3];

    let html_file_path = Path::new(html_file);
    if !html_file_path.exists(){
         return Err("The HTML file does not exist".into());
    }

    // Ensure the base url has a trailing slash
    // and create the full url for the html page
    let base_url = match BASE_URL.ends_with("/") {
        true => Url::parse(BASE_URL)?,
        false => Url::parse(&format!("{}/", BASE_URL))?
    };
    let full_url = base_url.join(html_file)?;

    // Get the contents of the HTML file
    let html_content = fs::read_to_string(html_file_path)
        .map_err(|_| format!("Could not read file '{}'", html_file))?;

    // Extract the content from the chosen element
    let main_content = extract_main_content(&html_content, LINES_TO_CUT, CONTENT_ELEMENT)
        .map_err(|e| format!("Error extracting content: {}", e))?;

    // Fix up any relative urls in the extracted content
    // and remove all unecessary whitespace
    let corrected_content = convert_relative_urls(&main_content, base_url)?;
    let compressed_content = remove_whitespace(&corrected_content);

    // Generate the new RSS item from the compressed content
    let rss_item = generate_rss_item(title, &full_url.as_str(), &compressed_content)?;

    // Insert the new item into the RSS file
    append_item_to_rss_file(&rss_file, &rss_item)?;
    Ok(())
}

/// Obtains the inner HTML text from a given element in an HTML file
/// optionally removing the first n lines
fn extract_main_content( html: &str, cut_lines: usize, elem: &str) -> Result<String, Box<dyn std::error::Error>> {

    // Parse HTML using the scraper crate
    let document = Html::parse_document(html);

    // Create a selector for the target element
    let selector = Selector::parse(elem)
        .map_err(|e| format!("Invalid CSS selector '{}': {}", elem, e))?;

    // Find the first matching element
    let element = document.select(&selector).next()
        .ok_or(format!("Could not find <{}> element in HTML", elem))?;

    // Extract the inner HTML from the element
    let content = element.inner_html();

    // Cut lines from the beginning
    let lines: Vec<&str> = content.lines().collect();
    let result = if (lines.len() > cut_lines) && (cut_lines > 0){
        lines[cut_lines..].join("\n")
    } else {
        content
    };
    Ok(result)
}

// Creates the new item element with all its child elements as a formatted string
fn generate_rss_item(title: &str, url: &str, description: &str) -> Result<String, Box<dyn std::error::Error>> {

    let mut w = Writer::new(Cursor::new(Vec::new()));

    // Start item element
    w.write_event(Event::Start(BytesStart::new("item")))?;

    // Add title
    w.write_event(Event::Start(BytesStart::new("title")))?;
    w.write_event(Event::Text(BytesText::new(title)))?;
    w.write_event(Event::End(BytesEnd::new("title")))?;

    // Add link
    w.write_event(Event::Start(BytesStart::new("link")))?;
    w.write_event(Event::Text(BytesText::new(url)))?;
    w.write_event(Event::End(BytesEnd::new("link")))?;

    // Add description with CDATA
    w.write_event(Event::Start(BytesStart::new("description")))?;
    w.write_event(Event::CData(BytesCData::new(description)))?;
    w.write_event(Event::End(BytesEnd::new("description")))?;

    // Add pubDate
    w.write_event(Event::Start(BytesStart::new("pubDate")))?;
    let pub_date = Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();
    w.write_event(Event::Text(BytesText::new(&pub_date)))?;
    w.write_event(Event::End(BytesEnd::new("pubDate")))?;

    // Add guid
    w.write_event(Event::Start(BytesStart::new("guid")))?;
    let guid = Uuid::new_v4().to_string();
    w.write_event(Event::Text(BytesText::new(&guid)))?;
    w.write_event(Event::End(BytesEnd::new("guid")))?;

    // End item element
    w.write_event(Event::End(BytesEnd::new("item")))?;

    // Get the result
    let result = w.into_inner().into_inner();
    pretty_xml(&result)
}

/// Indents the xml elements for easier human reading by adding spaces
fn pretty_xml(input: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    let mut reader = Reader::from_reader(input);

    let mut out = Vec::new();
    let mut writer = Writer::new_with_indent(&mut out, b' ', 12); // 12-space indent

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Eof => break,
            ev => writer.write_event(ev)?, // every event is re-emitted indented
        }
        buf.clear();
    }
    Ok(String::from_utf8(out)?)
}

/// Inserts the item into the RSS file before the end of the </channel> element
fn append_item_to_rss_file(rss_file: &str, item_xml: &str) -> Result<(), Box<dyn std::error::Error>> {

    if !rss_file.ends_with(".xml") {
        return Err("The RSS file is not an XML file".into());
    }
    let file_path = Path::new(rss_file);
    if !file_path.exists(){
        return Err("The XML file does not exist".into());
    }

    let rss_content = fs::read_to_string(file_path)?;

    // Find the closing </channel> tag and insert the item before it
    let channel_end = rss_content.find("</channel>");
    if let Some(pos) = channel_end {
        let (before, after) = rss_content.split_at(pos);
        let updated = format!("{}    {}\n    {}", before, item_xml, after);
        fs::write(file_path, updated)?;
    } else {
        return Err("Could not find </channel> tag in RSS file".into());
    }

    println!("✅ RSS item inserted into {}", rss_file);
    Ok(())
}

/// Finds all href, src, and srcset attributes
/// and converts all relative urls to absolute urls
fn convert_relative_urls(content: &str, base_url: Url) -> Result<String, Box<dyn std::error::Error>>  {

    // Regex to match attributes with paths
    let url_regex = Regex::new(r#"(href|src|srcset)=\"([^\"]*)\""#)?;

    // Replace matches with absolute URLs
    let converted = url_regex.replace_all(content, |caps: &regex::Captures | {
        let attribute = caps.get(1).unwrap().as_str();
        let value = caps.get(2).unwrap().as_str();
        // Only convert if it's not already a full URL
        if value.starts_with("http") {
            // Keep original full URL
            format!("{}=\"{}\"", attribute, value)
        } else {
            // Try to join the base url to the relative fragment
            // but just send back the fragment if the join fails for some reason
            let full_url = match base_url.join(value) {
                Ok(url) => url.to_string(),
                Err(_) => value.to_string()
            };
            format!("{}=\"{}\"", attribute, full_url.as_str())
        }
    }).to_string();
    Ok(converted)
}

 /// Replace multiple whitespace characters with single spaces
fn remove_whitespace(content: &str) -> String {
    let whitespace_regex = Regex::new(r"\s+").unwrap();
    let result = whitespace_regex.replace_all(content, " ").to_string();
    result.trim().to_string()
}
