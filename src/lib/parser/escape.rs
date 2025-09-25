//! Escape Sequence Parser - JSON-compatible escape handling
//!
//! Handles:
//! - \" → literal quote
//! - \\ → literal backslash
//! - \n → literal newline
//! - \t → literal tab
//! - \uXXXX → unicode character
//! - Security: fails on invalid escape sequences

/// Parse a string with escape sequences
///
/// Converts JSON-style escape sequences to their literal equivalents.
/// Fails on invalid escape sequences for security.
///
/// # Examples
/// ```ignore
/// let result = parse_escaped_value(r#"Hello \"world\""#)?;
/// assert_eq!(result, "Hello \"world\"");
/// ```
pub fn parse_escaped_value(input: &str) -> Result<String, String> {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('u') => {
                    // Unicode escape: \uXXXX
                    let mut hex = String::new();
                    for _ in 0..4 {
                        match chars.next() {
                            Some(c) if c.is_ascii_hexdigit() => hex.push(c),
                            _ => {
                                return Err(format!(
                                    "Invalid unicode escape sequence at \\u{}",
                                    hex
                                ))
                            }
                        }
                    }
                    let code = u32::from_str_radix(&hex, 16)
                        .map_err(|_| format!("Invalid unicode value: \\u{}", hex))?;
                    let unicode_char = char::from_u32(code)
                        .ok_or_else(|| format!("Invalid unicode code point: \\u{}", hex))?;
                    result.push(unicode_char);
                }
                Some(c) => {
                    return Err(format!("Invalid escape sequence: \\{}", c));
                }
                None => {
                    return Err("Unexpected end of input after backslash".to_string());
                }
            }
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}

/// Validate escape sequences without parsing
///
/// Checks that all escape sequences are valid without converting them.
///
/// # Examples
/// ```ignore
/// assert!(validate_escapes(r#"Hello \"world\""#).is_ok());
/// assert!(validate_escapes(r#"Bad \x escape"#).is_err());
/// ```
pub fn validate_escapes(input: &str) -> Result<(), String> {
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('"') | Some('\\') | Some('n') | Some('t') | Some('r') => {
                    // Valid single-char escapes
                }
                Some('u') => {
                    // Validate unicode escape
                    for i in 0..4 {
                        match chars.next() {
                            Some(c) if c.is_ascii_hexdigit() => {}
                            _ => return Err(format!("Invalid unicode escape at position {}", i)),
                        }
                    }
                }
                Some(c) => {
                    return Err(format!("Invalid escape sequence: \\{}", c));
                }
                None => {
                    return Err("Unexpected end of input after backslash".to_string());
                }
            }
        }
    }

    Ok(())
}

/// Strip quotes from a value if present
///
/// Removes surrounding quotes and processes escape sequences.
///
/// # Examples
/// ```ignore
/// let result = strip_quotes(r#""quoted value""#)?;
/// assert_eq!(result, "quoted value");
/// ```
pub fn strip_quotes(input: &str) -> Result<String, String> {
    let trimmed = input.trim();

    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        let without_quotes = &trimmed[1..trimmed.len() - 1];
        parse_escaped_value(without_quotes)
    } else {
        Ok(input.to_string())
    }
}

/// Check if a string contains unescaped quotes
pub fn has_unescaped_quotes(input: &str) -> bool {
    let mut chars = input.chars().peekable();
    let mut escape_next = false;

    while let Some(ch) = chars.next() {
        if escape_next {
            escape_next = false;
            continue;
        }

        if ch == '\\' {
            escape_next = true;
        } else if ch == '"' {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_escapes() {
        assert_eq!(
            parse_escaped_value(r#"Hello \"world\""#).unwrap(),
            "Hello \"world\""
        );
        assert_eq!(
            parse_escaped_value(r#"Line 1\nLine 2"#).unwrap(),
            "Line 1\nLine 2"
        );
        assert_eq!(parse_escaped_value(r#"Tab\there"#).unwrap(), "Tab\there");
        assert_eq!(
            parse_escaped_value(r#"Back\\slash"#).unwrap(),
            "Back\\slash"
        );
    }

    #[test]
    fn test_unicode_escapes() {
        assert_eq!(parse_escaped_value(r#"\u0041"#).unwrap(), "A");
        assert_eq!(
            parse_escaped_value(r#"Hello \u4e16\u754c"#).unwrap(),
            "Hello 世界"
        );
    }

    #[test]
    fn test_invalid_escapes() {
        assert!(parse_escaped_value(r#"\x20"#).is_err());
        assert!(parse_escaped_value(r#"\q"#).is_err());
        assert!(parse_escaped_value(r#"\"#).is_err());
    }

    #[test]
    fn test_validate_escapes() {
        assert!(validate_escapes(r#"Valid \"escape\""#).is_ok());
        assert!(validate_escapes(r#"Invalid \x escape"#).is_err());
        assert!(validate_escapes(r#"Unicode \u0041"#).is_ok());
        assert!(validate_escapes(r#"Bad unicode \uGGGG"#).is_err());
    }

    #[test]
    fn test_strip_quotes() {
        assert_eq!(strip_quotes(r#""quoted""#).unwrap(), "quoted");
        assert_eq!(strip_quotes(r#"unquoted"#).unwrap(), "unquoted");
        assert_eq!(
            strip_quotes(r#""with \"escapes\"""#).unwrap(),
            "with \"escapes\""
        );
    }

    #[test]
    fn test_unescaped_quotes() {
        assert!(has_unescaped_quotes(r#"has " quote"#));
        assert!(!has_unescaped_quotes(r#"escaped \" quote"#));
        assert!(!has_unescaped_quotes("no quotes"));
    }
}
