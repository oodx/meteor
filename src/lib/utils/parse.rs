//! Token stream parsing utilities
//!
//! This module provides the main entry point for parsing token streams
//! into structured TokenBucket data.

use crate::types::{Context, Namespace, Key, TokenBucket, MeteorError};
use std::str::FromStr;

/// Parse a token stream into a TokenBucket
///
/// # Token Stream Format
///
/// Token streams consist of key=value pairs separated by semicolons:
/// - Basic: `key=value`
/// - Multiple: `key1=value1; key2=value2`
/// - Namespaced: `namespace:key=value`
/// - Context switch: `ctx=user; key=value`
/// - Bracket notation: `list[0]=item` (will be transformed to `list__i_0`)
///
/// # Context Switching
///
/// The special key `ctx` switches the current context:
/// ```ignore
/// ctx=app; app:data=value1; ctx=user; user:data=value2
/// ```
///
/// # Examples
///
/// ```ignore
/// use meteor::parse_token_stream;
///
/// let tokens = parse_token_stream("key=value; ui:button=click").unwrap();
/// assert_eq!(tokens.get("", "key"), Some("value"));
/// assert_eq!(tokens.get("ui", "button"), Some("click"));
/// ```
pub fn parse_token_stream(input: &str) -> Result<TokenBucket, MeteorError> {
    if input.trim().is_empty() {
        return Err(MeteorError::empty("input"));
    }

    let mut bucket = TokenBucket::new();
    let mut position = 0;

    // Split by semicolons for individual tokens
    for token_str in input.split(';') {
        let token_str = token_str.trim();
        if token_str.is_empty() {
            continue;
        }

        // Parse the individual token
        parse_token(token_str, &mut bucket, position)?;
        position += token_str.len() + 1; // +1 for semicolon
    }

    Ok(bucket)
}

/// Parse a single token into the bucket
fn parse_token(
    token: &str,
    bucket: &mut TokenBucket,
    position: usize,
) -> Result<(), MeteorError> {
    // Find the equals sign
    let equals_pos = token.find('=')
        .ok_or_else(|| MeteorError::invalid_token(token, "missing '=' separator"))?;

    let left = token[..equals_pos].trim();
    let right = token[equals_pos + 1..].trim();

    if left.is_empty() {
        return Err(MeteorError::invalid_token(token, "empty key"));
    }

    // Check for context switch
    if left == "ctx" {
        let context = Context::from_str(right)
            .map_err(|e| MeteorError::parse(position, e))?;
        bucket.switch_context(context);
        return Ok(());
    }

    // Parse namespace and key
    let (namespace, key_str) = parse_namespace_key(left)?;

    // Apply bracket transformation to key
    let key = Key::new(key_str);

    // Parse the value with quote/escape handling
    let parsed_value = parse_value(right)?;
    bucket.set(&namespace, key.transformed(), parsed_value);

    Ok(())
}

/// Parse a namespace:key combination
fn parse_namespace_key(input: &str) -> Result<(String, String), MeteorError> {
    if let Some(colon_pos) = input.find(':') {
        let namespace = input[..colon_pos].trim();
        let key = input[colon_pos + 1..].trim();

        if namespace.is_empty() {
            return Err(MeteorError::invalid_token(input, "empty namespace"));
        }
        if key.is_empty() {
            return Err(MeteorError::invalid_token(input, "empty key after namespace"));
        }

        // Check namespace depth
        let ns = Namespace::from_string(namespace);
        if ns.is_too_deep() {
            return Err(MeteorError::namespace_too_deep(namespace, ns.depth()));
        }

        Ok((namespace.to_string(), key.to_string()))
    } else {
        // No namespace, just key
        Ok((String::new(), input.to_string()))
    }
}

/// Parse a value with quote and escape handling (RSB string-biased approach)
///
/// Handles common value formats:
/// - Unquoted: `simple_value`
/// - Double quoted: `"quoted value with spaces"`
/// - Single quoted: `'another quoted value'`
/// - Escaped characters: `"value with \"escaped\" quotes"`
fn parse_value(input: &str) -> Result<String, MeteorError> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Ok(String::new());
    }

    // Check for quoted values
    if let Some(quote_char) = get_quote_char(trimmed) {
        parse_quoted_value(trimmed, quote_char)
    } else {
        // Unquoted value - return as-is (RSB string-biased approach)
        Ok(trimmed.to_string())
    }
}

/// Get the quote character if the value is quoted
fn get_quote_char(value: &str) -> Option<char> {
    if value.len() < 2 {
        return None;
    }

    let first = value.chars().next().unwrap();
    let last = value.chars().last().unwrap();

    if (first == '"' && last == '"') || (first == '\'' && last == '\'') {
        Some(first)
    } else {
        None
    }
}

/// Parse a quoted value with escape handling
fn parse_quoted_value(input: &str, quote_char: char) -> Result<String, MeteorError> {
    // Remove surrounding quotes
    let content = &input[1..input.len()-1];

    let mut result = String::new();
    let mut chars = content.chars().peekable();
    let mut position = 1; // Start at 1 since we stripped the opening quote

    while let Some(ch) = chars.next() {
        position += 1;

        if ch == '\\' {
            // Handle escape sequences
            if let Some(&next_ch) = chars.peek() {
                chars.next(); // consume the escaped character
                position += 1;

                match next_ch {
                    '"' => result.push('"'),
                    '\'' => result.push('\''),
                    '\\' => result.push('\\'),
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    _ => {
                        // Unknown escape sequence - include the backslash and character
                        result.push('\\');
                        result.push(next_ch);
                    }
                }
            } else {
                // Backslash at end of string
                return Err(MeteorError::invalid_char(
                    '\\',
                    position,
                    "incomplete escape sequence at end of quoted value".to_string(),
                ));
            }
        } else if ch == quote_char {
            // Unescaped quote character inside quoted string
            return Err(MeteorError::invalid_char(
                ch,
                position,
                format!("unescaped {} inside quoted value", quote_char),
            ));
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_token() {
        let result = parse_token_stream("key=value");
        assert!(result.is_ok());

        let bucket = result.unwrap();
        assert_eq!(bucket.get("", "key"), Some("value"));
    }

    #[test]
    fn test_parse_multiple_tokens() {
        let result = parse_token_stream("key1=value1; key2=value2");
        assert!(result.is_ok());

        let bucket = result.unwrap();
        assert_eq!(bucket.get("", "key1"), Some("value1"));
        assert_eq!(bucket.get("", "key2"), Some("value2"));
    }

    #[test]
    fn test_parse_namespaced_token() {
        let result = parse_token_stream("ui:button=click");
        assert!(result.is_ok());

        let bucket = result.unwrap();
        assert_eq!(bucket.get("ui", "button"), Some("click"));
    }

    #[test]
    fn test_parse_context_switch() {
        let result = parse_token_stream("ctx=user; key=uservalue; ctx=app; key=appvalue");
        assert!(result.is_ok());

        let mut bucket = result.unwrap();

        // Current context should be app (last switched)
        assert_eq!(bucket.context().name(), "app");
        assert_eq!(bucket.get("", "key"), Some("appvalue"));

        // Switch to user context to see user data
        bucket.switch_context(Context::user());
        assert_eq!(bucket.get("", "key"), Some("uservalue"));
    }

    #[test]
    fn test_parse_empty_input() {
        let result = parse_token_stream("");
        assert!(result.is_err());

        let result = parse_token_stream("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_token() {
        let result = parse_token_stream("invalid");
        assert!(result.is_err());

        let result = parse_token_stream("=value");
        assert!(result.is_err());
    }

    #[test]
    fn test_namespace_depth_check() {
        // This should work (3 levels - at warning threshold)
        let result = parse_token_stream("ui.widgets.buttons:primary=click");
        assert!(result.is_ok());

        // This should fail (4+ levels)
        let result = parse_token_stream("ui.widgets.buttons.primary:icon=arrow");
        assert!(result.is_err());
    }

    #[test]
    fn test_value_parsing_quotes() {
        // Double quoted values
        let result = parse_token_stream("message=\"hello world\"");
        assert!(result.is_ok());
        let bucket = result.unwrap();
        assert_eq!(bucket.get("", "message"), Some("hello world"));

        // Single quoted values
        let result = parse_token_stream("name='John Doe'");
        assert!(result.is_ok());
        let bucket = result.unwrap();
        assert_eq!(bucket.get("", "name"), Some("John Doe"));

        // Unquoted values
        let result = parse_token_stream("simple=value");
        assert!(result.is_ok());
        let bucket = result.unwrap();
        assert_eq!(bucket.get("", "simple"), Some("value"));
    }

    #[test]
    fn test_value_parsing_escapes() {
        // Escaped quotes
        let result = parse_token_stream("text=\"She said \\\"hello\\\"\"");
        assert!(result.is_ok());
        let bucket = result.unwrap();
        assert_eq!(bucket.get("", "text"), Some("She said \"hello\""));

        // Escaped backslashes
        let result = parse_token_stream("path=\"C:\\\\Program Files\\\\\"");
        assert!(result.is_ok());
        let bucket = result.unwrap();
        assert_eq!(bucket.get("", "path"), Some("C:\\Program Files\\"));

        // Newlines and tabs
        let result = parse_token_stream("multiline=\"line1\\nline2\\tindented\"");
        assert!(result.is_ok());
        let bucket = result.unwrap();
        assert_eq!(bucket.get("", "multiline"), Some("line1\nline2\tindented"));
    }

    #[test]
    fn test_value_parsing_errors() {
        // Incomplete escape sequence (properly quoted)
        let result = parse_token_stream("bad=\"incomplete\\\"");
        assert!(result.is_err());

        // Unescaped quote inside string
        let result = parse_token_stream("bad=\"quote\"inside\"");
        assert!(result.is_err());

        // Mismatched quotes
        let result = parse_token_stream("bad=\"mismatched'");
        assert!(result.is_ok()); // This actually parses as unquoted value - that's RSB string-biased behavior
    }
}