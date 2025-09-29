//! Centralized Smart-Split Logic - ENG-42
//!
//! Unified parsing utilities for handling quoted strings, escape sequences,
//! and delimiter splitting across all meteor parsers and validators.
//!
//! This module consolidates the previously scattered smart-split implementations
//! from validators.rs, token_stream.rs, and meteor_stream.rs into a single,
//! comprehensive solution with extensive regression testing.

/// Configuration for smart-split behavior
#[derive(Debug, Clone)]
pub struct SplitConfig {
    /// Character to split on
    pub delimiter: char,
    /// Whether to handle escape sequences (backslash escaping)
    pub handle_escapes: bool,
    /// Whether to only handle escapes inside quotes (more permissive)
    pub escapes_only_in_quotes: bool,
    /// Whether to preserve delimiter characters in the output
    pub preserve_delimiters: bool,
    /// Whether to trim whitespace from results
    pub trim_results: bool,
}

impl SplitConfig {
    /// Standard configuration for semicolon-separated tokens (validators.rs style)
    pub fn semicolon_tokens() -> Self {
        Self {
            delimiter: ';',
            handle_escapes: false,
            escapes_only_in_quotes: false,
            preserve_delimiters: false,
            trim_results: false, // Preserve original behavior - no trimming
        }
    }

    /// Standard configuration for general parsing (token_stream.rs style)
    pub fn general_parsing(delimiter: char) -> Self {
        Self {
            delimiter,
            handle_escapes: true,
            escapes_only_in_quotes: false,
            preserve_delimiters: false,
            trim_results: true,
        }
    }

    /// Standard configuration for meteor streams (meteor_stream.rs style)
    pub fn meteor_streams(delimiter: char) -> Self {
        Self {
            delimiter,
            handle_escapes: true,
            escapes_only_in_quotes: true,
            preserve_delimiters: false,
            trim_results: true,
        }
    }

    /// Configuration for meteor delimiter parsing (:;:)
    pub fn meteor_delimiter() -> Self {
        Self {
            delimiter: ':', // Will be handled specially for multi-char delimiter
            handle_escapes: true,
            escapes_only_in_quotes: true,
            preserve_delimiters: false,
            trim_results: true,
        }
    }
}

/// Smart split with comprehensive quote and escape handling
///
/// Returns a vector of owned strings for compatibility with existing APIs.
/// For zero-copy operations, use `smart_split_borrowed()`.
pub fn smart_split(input: &str, config: SplitConfig) -> Vec<String> {
    let borrowed = smart_split_borrowed(input, config);
    borrowed.into_iter().map(|s| s.to_string()).collect()
}

/// Smart split returning borrowed string slices for better performance
///
/// Returns `None` if quote parsing fails (unclosed quotes).
pub fn smart_split_borrowed(input: &str, config: SplitConfig) -> Vec<&str> {
    let mut result = Vec::new();
    let mut current_start_byte = 0;
    let mut in_quotes = false;
    let mut escape_next = false;

    let mut byte_pos = 0;
    for ch in input.chars() {
        if escape_next {
            escape_next = false;
            byte_pos += ch.len_utf8();
            continue;
        }

        match ch {
            '\\' if config.handle_escapes => {
                if !config.escapes_only_in_quotes || in_quotes {
                    escape_next = true;
                }
            }
            '"' => {
                in_quotes = !in_quotes;
            }
            c if c == config.delimiter && !in_quotes => {
                // Found delimiter outside quotes
                let segment = &input[current_start_byte..byte_pos];
                if config.trim_results {
                    let trimmed = segment.trim();
                    if !trimmed.is_empty() {
                        result.push(trimmed);
                    }
                } else if !segment.is_empty() {
                    result.push(segment);
                }
                current_start_byte = byte_pos + ch.len_utf8();
                byte_pos += ch.len_utf8();
                continue;
            }
            _ => {}
        }
        byte_pos += ch.len_utf8();
    }

    // Handle final segment
    if current_start_byte < input.len() {
        let segment = &input[current_start_byte..];
        if config.trim_results {
            let trimmed = segment.trim();
            if !trimmed.is_empty() {
                result.push(trimmed);
            }
        } else if !segment.is_empty() {
            result.push(segment);
        }
    }

    result
}

/// Split by multi-character delimiter (e.g., ":;:" for meteor streams)
///
/// Handles quotes and escaping while looking for the complete delimiter sequence.
pub fn smart_split_multi_char(input: &str, delimiter: &str, config: SplitConfig) -> Vec<String> {
    if delimiter.len() == 1 {
        return smart_split(input, config);
    }

    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut escape_next = false;
    let delimiter_chars: Vec<char> = delimiter.chars().collect();
    let mut delimiter_match = 0;

    for ch in input.chars() {
        if escape_next {
            current.push(ch);
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if config.handle_escapes => {
                if !config.escapes_only_in_quotes || in_quotes {
                    escape_next = true;
                }
                current.push(ch);
            }
            '"' => {
                current.push(ch);
                in_quotes = !in_quotes;
            }
            c if !in_quotes && c == delimiter_chars[delimiter_match] => {
                delimiter_match += 1;
                if delimiter_match == delimiter_chars.len() {
                    // Found complete delimiter
                    if !current.trim().is_empty() {
                        // Remove partial delimiter from current
                        for _ in 0..(delimiter_chars.len() - 1) {
                            current.pop();
                        }
                        let segment = if config.trim_results {
                            current.trim().to_string()
                        } else {
                            current.clone()
                        };
                        if !segment.is_empty() {
                            result.push(segment);
                        }
                    }
                    current.clear();
                    delimiter_match = 0;
                } else {
                    current.push(c);
                }
            }
            _ => {
                // Reset delimiter match if we didn't complete it
                if delimiter_match > 0 && !in_quotes {
                    delimiter_match = 0;
                }
                current.push(ch);
            }
        }
    }

    if !current.trim().is_empty() {
        let segment = if config.trim_results {
            current.trim().to_string()
        } else {
            current
        };
        result.push(segment);
    }

    result
}

/// Compatibility function for existing validators.rs usage
///
/// Returns `None` if quotes are unclosed (for error handling).
pub fn smart_split_semicolons(s: &str) -> Option<Vec<&str>> {
    let result = smart_split_borrowed(s, SplitConfig::semicolon_tokens());
    // Check for unclosed quotes by counting quote pairs
    let quote_count = s.chars().filter(|&c| c == '"').count();
    if quote_count % 2 != 0 {
        return None; // Unclosed quotes
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_semicolon_split() {
        let config = SplitConfig::semicolon_tokens();
        let result = smart_split("key=value;theme=dark;lang=en", config);
        assert_eq!(result, vec!["key=value", "theme=dark", "lang=en"]);
    }

    #[test]
    fn test_quoted_semicolon_split() {
        let config = SplitConfig::semicolon_tokens();
        let result = smart_split("key=value; message=\"hello; world\"; theme=dark", config);
        // Semicolon config preserves whitespace (validators.rs compatibility)
        assert_eq!(
            result,
            vec!["key=value", " message=\"hello; world\"", " theme=dark"]
        );
    }

    #[test]
    fn test_escaped_quotes_token_style() {
        let config = SplitConfig::general_parsing(';');
        let result = smart_split("key=\"value with \\\"quotes\\\"\"; theme=dark", config);
        assert_eq!(
            result,
            vec!["key=\"value with \\\"quotes\\\"\"", "theme=dark"]
        );
    }

    #[test]
    fn test_escaped_quotes_meteor_style() {
        let config = SplitConfig::meteor_streams(';');
        let result = smart_split("key=\"value with \\\"quotes\\\"\"; theme=dark", config);
        assert_eq!(
            result,
            vec!["key=\"value with \\\"quotes\\\"\"", "theme=dark"]
        );
    }

    #[test]
    fn test_multi_char_delimiter() {
        let config = SplitConfig::meteor_delimiter();
        let result = smart_split_multi_char(
            "app:ui:button=click :;: user:main:profile=admin",
            ":;:",
            config,
        );
        assert_eq!(
            result,
            vec!["app:ui:button=click", "user:main:profile=admin"]
        );
    }

    #[test]
    fn test_multi_char_delimiter_with_quotes() {
        let config = SplitConfig::meteor_delimiter();
        let result = smart_split_multi_char(
            "app:ui:button=click :;: user:main:profile=\"admin :;: test\"",
            ":;:",
            config,
        );
        assert_eq!(
            result,
            vec![
                "app:ui:button=click",
                "user:main:profile=\"admin :;: test\""
            ]
        );
    }

    #[test]
    fn test_control_tokens() {
        let config = SplitConfig::general_parsing(';');
        let result = smart_split("button=click; ns=ui; ctl:delete=app:main:test", config);
        assert_eq!(
            result,
            vec!["button=click", "ns=ui", "ctl:delete=app:main:test"]
        );
    }

    #[test]
    fn test_bracket_notation() {
        let config = SplitConfig::general_parsing(';');
        let result = smart_split("list[0]=first; grid[1,2]=cell; config[debug]=true", config);
        assert_eq!(
            result,
            vec!["list[0]=first", "grid[1,2]=cell", "config[debug]=true"]
        );
    }

    #[test]
    fn test_empty_segments() {
        let config = SplitConfig::general_parsing(';');
        let result = smart_split("key=value;; theme=dark; ", config);
        assert_eq!(result, vec!["key=value", "theme=dark"]);
    }

    #[test]
    fn test_semicolon_compatibility() {
        // Test compatibility with validators.rs behavior (preserves whitespace)
        let result =
            smart_split_semicolons("key=value; message=\"hello; world\"; theme=dark").unwrap();
        assert_eq!(
            result,
            vec!["key=value", " message=\"hello; world\"", " theme=dark"]
        );
    }

    #[test]
    fn test_unclosed_quotes() {
        let result = smart_split_semicolons("key=\"unclosed quote");
        assert!(result.is_none());
    }

    #[test]
    fn test_complex_escaping() {
        let config = SplitConfig::general_parsing(';');
        let result = smart_split(
            "path=\"/usr/bin\"; message=\"Hello\\nWorld\"; escaped=\"quote\\\"here\"",
            config,
        );
        assert_eq!(
            result,
            vec![
                "path=\"/usr/bin\"",
                "message=\"Hello\\nWorld\"",
                "escaped=\"quote\\\"here\""
            ]
        );
    }

    #[test]
    fn test_borrowed_vs_owned() {
        let input = "key=value; theme=dark";
        let config = SplitConfig::semicolon_tokens();

        let borrowed = smart_split_borrowed(input, config.clone());
        let owned = smart_split(input, config);

        assert_eq!(borrowed.len(), owned.len());
        for (b, o) in borrowed.iter().zip(owned.iter()) {
            assert_eq!(*b, o);
        }
    }
}
