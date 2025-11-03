use chrono::{DateTime, ParseError, FixedOffset, Utc, TimeZone};
use url::Url;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

/// Escape XML special characters
pub fn escape_xml(text: &str) -> String {
    text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&apos;")
}

pub fn now_rfc2822() -> String {
    Utc::now().to_rfc2822()
}

/// Parse an arbitrary date string and return it in RFC-2822 format
/// which is accepted by RSS readers.
/// Accepted inputs are anything that `chrono` can understand
/// If the string has no time-zone information, UTC is assumed.
pub fn parse_to_rfc2822(input: &str) -> Result<String, ParseError> {
    // First try to parse as a DateTime (with time-zone)
    if let Ok(dt) = DateTime::parse_from_rfc2822(input) {
        return Ok(dt.to_rfc2822());
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(input) {
        return Ok(dt.to_rfc2822());
    }
    if let Ok(dt) = input.parse::<DateTime<FixedOffset>>() {
        return Ok(dt.to_rfc2822());
    }

    // No time-zone info → assume UTC
    let naive = chrono::NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S")
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M"))
        .or_else(|_| chrono::NaiveDate::parse_from_str(input, "%Y-%m-%d").map(|d| d.and_hms_opt(0, 0, 0).unwrap()))?;

    let utc = DateTime::<FixedOffset>::from_naive_utc_and_offset(naive, FixedOffset::east_opt(0).unwrap());
    Ok(utc.to_rfc2822())
}

/// Intelligently joins a base URL and a path fragment
pub fn merge_url_and_fragment(base_url: &str, fragment: &str) -> Result<String, url::ParseError> {

    // If fragment is already a full URL, return it as-is
    if fragment.starts_with("http")  {
        return Ok(fragment.to_string());
    }

    // If fragment is empty, return base URL
    if fragment.is_empty() {
        return Ok(base_url.to_string());
    }

    // Add backslash if not present
    let url = match base_url.ends_with("/") {
        true => base_url,
        false => &format!("{}/", base_url)
    };

    // Parse the URL
    let base = Url::parse(url)?;

    // Use join method which handles all the URL merging logic
    let joined = base.join(fragment)?;

    Ok(joined.to_string())
}

// Removes the final file path or directory from a URL
pub fn remove_last_segment_from_url(url: &str) -> Result<String, url::ParseError> {
    let parsed_url = Url::parse(url)?;

    // Get current path
    let current_path = parsed_url.path();

    if current_path.is_empty() || current_path == "/" {
        return Ok(parsed_url.to_string());
    }

    // Remove the last segment from path
    let new_path = if current_path.ends_with('/') {
        // Path ends with slash, remove last segment
        let path_without_trailing = &current_path[..current_path.len() - 1];
        if let Some(last_slash) = path_without_trailing.rfind('/') {
            &path_without_trailing[..last_slash + 1]
        } else {
            "/"
        }
    } else {
        // Path doesn't end with slash, remove last segment
        if let Some(last_slash) = current_path.rfind('/') {
            &current_path[..last_slash + 1]
        } else {
            "/"
        }
    };

    let mut new_url = parsed_url.clone();
    new_url.set_path(new_path);

    Ok(new_url.to_string())
}

/// Inserts text before a given text string in a given file path
pub fn insert_before_text(file_path: &str, target_text: &str, insert_text: &str) -> std::io::Result<()> {
    // Read the entire file content
    let mut file = File::open(file_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    // Find the first occurrence of target text
    if let Some(pos) = content.find(target_text) {
        // Create new content with insert_text before target_text
        let mut new_content = String::new();
        new_content.push_str(&content[..pos]);
        new_content.push_str(insert_text);
        new_content.push_str(target_text);
        new_content.push_str(&content[pos + target_text.len()..]);

        // Write back to the file
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(file_path)?;
        file.write_all(new_content.as_bytes())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /******************** Date formatting  **********************/

    #[test]
    fn check_various_date_formats() {
        let cases = vec![
            ("2022-06-02 14:30", "Thu, 2 Jun 2022 14:30:00 +0000"),
            ("Fri, 02 Jun 2023 14:30:00 +0000", "Fri, 2 Jun 2023 14:30:00 +0000"),
            ("2024-06-02T14:30:00Z", "Sun, 2 Jun 2024 14:30:00 +0000"),
            ("2024-06-16T14:30:00Z", "Sun, 16 Jun 2024 14:30:00 +0000"),
        ];

        for (inp, exp) in cases {
            assert_eq!(parse_to_rfc2822(inp).unwrap(), exp);
        }
    }


    /******************** URL merging **********************/

    #[test]
    fn merge_with_relative_file_path() {
        // Should add the path fragment to the URL
        let merged = merge_url_and_fragment("http://www.xxx.com/blog/", "path/to/image.png").unwrap();
        assert_eq!(merged, "http://www.xxx.com/blog/path/to/image.png")
    }

    #[test]
    fn merge_with_fragment_as_full_url() {
        // Should return just the fragment
        let merged = merge_url_and_fragment("http://www.xxx.com/", "http://www.zzz.com/file.htm").unwrap();
        assert_eq!(merged, "http://www.zzz.com/file.htm")
    }

    #[test]
    fn merge_with_empty_fragment() {
        // Should return just the URL
        let merged = merge_url_and_fragment("http://www.xxx.com/", "").unwrap();
        assert_eq!(merged, "http://www.xxx.com/")
    }

    #[test]
    fn merge_with_root_relative_path() {
        // Should add the just the root domain to the root-relative path
        let merged = merge_url_and_fragment("http://www.xxx.com/blog/temp/", "/path/to/file.htm").unwrap();
        assert_eq!(merged, "http://www.xxx.com/path/to/file.htm")
    }

    #[test]
    fn merge_with_relative_file_path_url_missing_backslash() {
        // Should add the path fragment to the URL
        let merged = merge_url_and_fragment("http://www.xxx.com/blog", "path/to/image.png").unwrap();
        assert_eq!(merged, "http://www.xxx.com/blog/path/to/image.png")
    }

    #[test]
    fn merge_with_parent_dir() {
        let merged = merge_url_and_fragment("http://www.xxx.com/parent/blog/", "../path/to/file.htm").unwrap();
        assert_eq!(merged, "http://www.xxx.com/parent/path/to/file.htm")
    }

    #[test]
    fn merge_with_grandparent_dir() {
        let merged = merge_url_and_fragment("http://www.xxx.com/grandparent/blog/temp/", "../../path/to/file.htm").unwrap();
        assert_eq!(merged, "http://www.xxx.com/grandparent/path/to/file.htm")
    }

    /******************** URL path removal **********************/

    #[test]
    fn strip_last_dir_from_url() {
        // Remove the last directory from the path
        let stripped = remove_last_segment_from_url("http://www.xxx.com/blog/temp/").unwrap();
        assert_eq!(stripped, "http://www.xxx.com/blog/")
    }

    #[test]
    fn strip_file_from_url() {
        // Remove the file from the path
        let stripped = remove_last_segment_from_url("http://www.xxx.com/blog/file.htm").unwrap();
        assert_eq!(stripped, "http://www.xxx.com/blog/")
    }

    #[test]
    fn strip_nothing_from_url() {
        // No path or file, so just return the URL
        let stripped = remove_last_segment_from_url("http://www.xxx.com/").unwrap();
        assert_eq!(stripped, "http://www.xxx.com/")
    }
}
