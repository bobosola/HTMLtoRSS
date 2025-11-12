use chrono::{DateTime, ParseError, FixedOffset, Utc};
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

/// Joins a base URL and a path fragment
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

/// Merges a base URL and a path fragment and removes any overlaps by
/// handling the case where a base URL and relative path might contain
/// common segments, e.g. avoiding:
// https://site/blog  + blog/page.html -> https://site/blog/blog/page.html
pub fn merge_remove_overlap(base_url: &str, relative_path: &str) -> Result<String, url::ParseError> {
    // If relative_path is already a full URL, return it as-is
    if relative_path.starts_with("http") {
        return Ok(relative_path.to_string());
    }

    // If relative_path is empty, return base URL
    if relative_path.is_empty() {
        return Ok(base_url.to_string());
    }

    // Handle special URL patterns that shouldn't trigger overlap removal
    if relative_path.starts_with('/') || relative_path.starts_with("..") {
        // Root-relative or parent directory paths - use normal join
        let base = Url::parse(base_url)?;
        return Ok(base.join(relative_path)?.to_string());
    }

    // Normalize the base URL to ensure it ends with a slash for proper directory handling
    let normalized_base = if !base_url.ends_with('/') {
        format!("{}/", base_url)
    } else {
        base_url.to_string()
    };

    // Parse the normalized base URL
    let base = Url::parse(&normalized_base)?;
    
    // Get the path segments from the base URL (filter out empty segments)
    let base_segments: Vec<&str> = base.path_segments()
        .map(|segments| segments.filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();
    
    // Get the first segment of the relative path (if it has path segments)
    let relative_first_segment = relative_path.split('/').next().unwrap_or("");
    
    // Check if the relative path starts with a segment that's the same as the last segment of base URL path
    // This indicates potential overlap that we want to avoid
    let should_strip_base_path = !base_segments.is_empty() && 
                                 !relative_first_segment.is_empty() &&
                                 base_segments.last() == Some(&relative_first_segment);
    
    let result = if should_strip_base_path {
        // Remove the last segment from the base URL path to avoid duplication
        let mut new_base = base.clone();
        let new_path = if base_segments.len() > 1 {
            format!("/{}/", base_segments[..base_segments.len() - 1].join("/"))
        } else {
            "/".to_string()
        };
        new_base.set_path(&new_path);
        new_base.join(relative_path)?
    } else {
        // Normal merge
        base.join(relative_path)?
    };
    
    Ok(result.to_string())
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

    /******************** URL merge with overlap removal **********************/

    #[test]
    fn merge_remove_overlap_basic_case() {
        // The main case: avoid duplication when relative path starts with same segment as base path ends
        let merged = merge_remove_overlap("https://site/blog", "blog/page.html").unwrap();
        assert_eq!(merged, "https://site/blog/page.html")
    }

    #[test]
    fn merge_remove_overlap_with_trailing_slash() {
        // Should work the same with trailing slash on base URL
        let merged = merge_remove_overlap("https://site/blog/", "blog/page.html").unwrap();
        assert_eq!(merged, "https://site/blog/page.html")
    }

    #[test]
    fn merge_remove_overlap_no_overlap() {
        // Should work normally when there's no overlap
        // Base URL without trailing slash should still treat "blog" as a directory when appending
        let merged = merge_remove_overlap("https://site/blog", "posts/page.html").unwrap();
        assert_eq!(merged, "https://site/blog/posts/page.html")
    }

    #[test]
    fn merge_remove_overlap_root_relative() {
        // Should not remove overlap for root-relative paths (starting with /)
        let merged = merge_remove_overlap("https://site/blog", "/other/page.html").unwrap();
        assert_eq!(merged, "https://site/other/page.html")
    }

    #[test]
    fn merge_remove_overlap_parent_dir() {
        // Should not remove overlap for parent directory paths
        let merged = merge_remove_overlap("https://site/blog/temp/", "../page.html").unwrap();
        assert_eq!(merged, "https://site/blog/page.html")
    }

    #[test]
    fn merge_remove_overlap_full_url() {
        // Should return full URL as-is
        let merged = merge_remove_overlap("https://site/blog", "https://other.com/page.html").unwrap();
        assert_eq!(merged, "https://other.com/page.html")
    }

    #[test]
    fn merge_remove_overlap_empty_path() {
        // Should return base URL when relative path is empty
        let merged = merge_remove_overlap("https://site/blog", "").unwrap();
        assert_eq!(merged, "https://site/blog")
    }

    #[test]
    fn merge_remove_overlap_multiple_segments() {
        // Should only remove the last segment if it matches
        let merged = merge_remove_overlap("https://site/a/b/c", "c/d/e.html").unwrap();
        assert_eq!(merged, "https://site/a/b/c/d/e.html")
    }

    #[test]
    fn merge_remove_overlap_no_base_path() {
        // Should work when base URL has no path
        let merged = merge_remove_overlap("https://site", "blog/page.html").unwrap();
        assert_eq!(merged, "https://site/blog/page.html")
    }

    #[test]
    fn merge_remove_overlap_substring_match() {
        // Should only match complete path segments, not substrings
        // "blogger" is not the same as "blog", so no overlap removal
        let merged = merge_remove_overlap("https://site/blogger/", "blog/page.html").unwrap();
        assert_eq!(merged, "https://site/blogger/blog/page.html")
    }
}
