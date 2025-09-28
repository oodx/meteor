//! String format validators for Meteor token formats
//!
//! This module provides low-level string format validation without parsing.
//! These validators check syntax and format correctness only.

use crate::types::METEOR_DELIMITER;

/// Validate token string format: "key=value"
///
/// Checks for:
/// - Exactly one '=' separator
/// - Non-empty key (before =)
/// - Value can be empty
/// - Key cannot be just whitespace
///
/// # Examples
/// ```
/// use meteor::utils::validators::is_valid_token_format;
///
/// assert!(is_valid_token_format("button=click"));
/// assert!(is_valid_token_format("key="));           // empty value OK
/// assert!(is_valid_token_format("list[0]=item"));   // bracket notation OK
/// assert!(!is_valid_token_format("just_key"));      // missing =
/// assert!(!is_valid_token_format("=value"));        // empty key
/// assert!(!is_valid_token_format(" =value"));       // whitespace-only key
/// ```
pub fn is_valid_token_format(s: &str) -> bool {
    // Must contain exactly one '='
    let eq_count = s.matches('=').count();
    if eq_count != 1 {
        return false;
    }

    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return false;
    }

    let key = parts[0].trim();

    // Key cannot be empty or whitespace-only
    if key.is_empty() {
        return false;
    }

    // Key cannot contain spaces (unless it's a special case we allow)
    // This prevents ambiguous parsing like "spaces in key=value"
    if key.contains(' ') {
        return false;
    }

    // Value can be anything (including empty)
    true
}

/// Validate meteor string format: "context:namespace:key=value; key2=value2"
///
/// Checks for:
/// - Contains at least one valid token
/// - No consecutive semicolons (;;, ;;;, etc.) outside quoted values
/// - Valid colon structure for context:namespace addressing
/// - All tokens within meteor are valid
/// - Supports quoted values: key="value; with semicolons"
///
/// # Examples
/// ```
/// use meteor::utils::validators::is_valid_meteor_format;
///
/// assert!(is_valid_meteor_format("app:ui:button=click"));
/// assert!(is_valid_meteor_format("button=click"));              // minimal format OK
/// assert!(is_valid_meteor_format("app:ui:button=click; theme=dark"));
/// assert!(is_valid_meteor_format("key=\"value;;; with semicolons\"")); // quoted values OK
/// assert!(!is_valid_meteor_format("app:ui:button=click;; theme=dark")); // consecutive semicolons
/// assert!(!is_valid_meteor_format(""));                         // empty
/// ```
pub fn is_valid_meteor_format(s: &str) -> bool {
    let s = s.trim();

    if s.is_empty() {
        return false;
    }

    // Smart split by semicolon, respecting quotes
    let token_parts = match crate::parser::split::smart_split_semicolons(s) {
        Some(parts) => parts,
        None => return false, // Unclosed quotes
    };

    // Check for consecutive semicolons in the original string
    // But only outside of quoted values
    if has_consecutive_semicolons_outside_quotes(s) {
        return false;
    }

    let mut valid_tokens = 0;

    for token_part in token_parts {
        let token_part = token_part.trim();
        if token_part.is_empty() {
            continue;
        }

        // For meteor format, we need to handle context:namespace:key=value
        // But also allow simple key=value (defaults to app context)

        if is_valid_token_in_meteor_context(token_part) {
            valid_tokens += 1;
        } else {
            return false;
        }
    }

    // Must have at least one valid token
    valid_tokens > 0
}

/// Validate meteor shower string format: multiple meteors separated by `:;:`
///
/// Checks for:
/// - Contains at least one valid meteor
/// - Proper meteor delimiter usage
/// - Each meteor is individually valid
/// - No syntax errors
///
/// # Examples
/// ```
/// use meteor::utils::validators::is_valid_meteor_shower_format;
///
/// assert!(is_valid_meteor_shower_format("app:ui:button=click"));
/// assert!(is_valid_meteor_shower_format("app:ui:button=click :;: user:settings:theme=dark"));
/// assert!(!is_valid_meteor_shower_format(""));
/// assert!(!is_valid_meteor_shower_format("app:ui:button=click;; theme=dark")); // consecutive semicolons
/// ```
pub fn is_valid_meteor_shower_format(s: &str) -> bool {
    let s = s.trim();

    if s.is_empty() {
        return false;
    }

    // Split by meteor delimiter and validate each meteor
    let meteor_parts: Vec<&str> = s.split(METEOR_DELIMITER).collect();
    let mut valid_meteors = 0;

    for meteor_part in meteor_parts {
        let meteor_part = meteor_part.trim();
        if meteor_part.is_empty() {
            continue;
        }

        if is_valid_meteor_format(meteor_part) {
            valid_meteors += 1;
        } else {
            return false;
        }
    }

    // Must have at least one valid meteor
    valid_meteors > 0
}

/// Smart split by semicolons, respecting quoted values
///
/// Splits on semicolons but treats quoted strings as single units.
/// Returns None if quotes are unclosed.
// smart_split_semicolons moved to centralized parser::split module (ENG-42)

/// Check for consecutive semicolons outside quoted values
fn has_consecutive_semicolons_outside_quotes(s: &str) -> bool {
    let mut in_quotes = false;
    let mut prev_was_semicolon = false;

    for ch in s.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
                prev_was_semicolon = false;
            }
            ';' => {
                if !in_quotes && prev_was_semicolon {
                    return true; // Found consecutive semicolons outside quotes
                }
                prev_was_semicolon = !in_quotes;
            }
            _ => {
                prev_was_semicolon = false;
            }
        }
    }

    false
}

/// Helper: Validate a token within meteor context
///
/// Handles both full context:namespace:key=value and simple key=value formats
fn is_valid_token_in_meteor_context(s: &str) -> bool {
    let s = s.trim();

    // Check for context:namespace:key=value format
    if s.contains(':') {
        // Validate structure by parsing path specification

        // Find the last '=' to separate path from value
        if let Some(eq_pos) = s.rfind('=') {
            let path_part = &s[..eq_pos]; // context:namespace:key
            let value_part = &s[eq_pos + 1..]; // value

            // Count colons only in the path specification
            let path_colons = path_part.matches(':').count();

            // Should be 1 or 2 colons in path: namespace:key or context:namespace:key
            if path_colons == 0 || path_colons > 2 {
                return false;
            }

            // Validate the key=value part by reconstructing it
            let key_part = if path_colons == 1 {
                // namespace:key format
                path_part.split(':').last().unwrap_or("")
            } else {
                // context:namespace:key format
                path_part.split(':').last().unwrap_or("")
            };

            let reconstructed_token = format!("{}={}", key_part, value_part);
            return is_valid_token_format(&reconstructed_token);
        } else {
            // No '=' found, invalid
            return false;
        }
    } else {
        // Simple key=value format
        return is_valid_token_format(s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_token_formats() {
        assert!(is_valid_token_format("button=click"));
        assert!(is_valid_token_format("key="));
        assert!(is_valid_token_format("list[0]=item"));
        assert!(is_valid_token_format("key=value with spaces"));
        assert!(is_valid_token_format("complex_key[x,y]=complex value"));
    }

    #[test]
    fn test_invalid_token_formats() {
        assert!(!is_valid_token_format(""));
        assert!(!is_valid_token_format("just_key"));
        assert!(!is_valid_token_format("=value"));
        assert!(!is_valid_token_format(" =value"));
        assert!(!is_valid_token_format("key=value=extra"));
        assert!(!is_valid_token_format("key"));

        // Spaces in keys should be invalid
        assert!(!is_valid_token_format("spaces in key=value"));
        assert!(!is_valid_token_format("key with spaces=value"));
    }

    #[test]
    fn test_valid_meteor_formats() {
        assert!(is_valid_meteor_format("button=click"));
        assert!(is_valid_meteor_format("app:ui:button=click"));
        assert!(is_valid_meteor_format("ui:button=click"));
        assert!(is_valid_meteor_format("app:ui:button=click; theme=dark"));
        assert!(is_valid_meteor_format(
            "button=click; theme=dark; size=large"
        ));
    }

    #[test]
    fn test_invalid_meteor_formats() {
        assert!(!is_valid_meteor_format(""));
        assert!(!is_valid_meteor_format("app:ui:button"));
        assert!(!is_valid_meteor_format("app:ui:button=click;; theme=dark"));
        assert!(!is_valid_meteor_format("app:ui:button=click;;; theme=dark"));
        assert!(!is_valid_meteor_format("invalid"));

        // Unclosed quotes should be invalid
        assert!(!is_valid_meteor_format("key=\"unclosed quote"));
        assert!(!is_valid_meteor_format("key=value; message=\"unclosed"));

        // Spaces in keys should be invalid
        assert!(!is_valid_meteor_format("spaces in key=value"));
    }

    #[test]
    fn test_valid_meteor_shower_formats() {
        assert!(is_valid_meteor_shower_format("button=click"));
        assert!(is_valid_meteor_shower_format("app:ui:button=click"));
        assert!(is_valid_meteor_shower_format(
            "app:ui:button=click :;: user:settings:theme=dark"
        ));
        assert!(is_valid_meteor_shower_format(
            "button=click; theme=dark :;: user:profile=admin"
        ));
    }

    #[test]
    fn test_invalid_meteor_shower_formats() {
        assert!(!is_valid_meteor_shower_format(""));
        assert!(!is_valid_meteor_shower_format(
            "app:ui:button=click;; theme=dark"
        ));
        assert!(!is_valid_meteor_shower_format("invalid"));
        assert!(!is_valid_meteor_shower_format("app:ui:button"));
    }

    #[test]
    fn test_meteor_context_validation() {
        // Full addressing
        assert!(is_valid_token_in_meteor_context("app:ui:button=click"));
        assert!(is_valid_token_in_meteor_context("user:settings:theme=dark"));

        // Namespace only
        assert!(is_valid_token_in_meteor_context("ui:button=click"));

        // Simple format
        assert!(is_valid_token_in_meteor_context("button=click"));

        // Invalid formats
        assert!(!is_valid_token_in_meteor_context("app:ui:button"));
        assert!(!is_valid_token_in_meteor_context(
            "app:ui:button:extra=value"
        ));
        assert!(!is_valid_token_in_meteor_context(""));
    }

    #[test]
    fn test_consecutive_semicolon_detection() {
        assert!(!is_valid_meteor_format("key=value;; key2=value2"));
        assert!(!is_valid_meteor_format("key=value;;; key2=value2"));
        assert!(!is_valid_meteor_shower_format(
            "app:ui:button=click;; theme=dark"
        ));

        // Single semicolons should be OK
        assert!(is_valid_meteor_format("key=value; key2=value2"));
        assert!(is_valid_meteor_shower_format(
            "app:ui:button=click; theme=dark"
        ));
    }

    #[test]
    fn test_quoted_values() {
        // Quoted values with semicolons should be valid
        assert!(is_valid_token_format("key=\"value; with semicolons\""));
        assert!(is_valid_token_format("message=\"Hello;; World;;;\""));

        // Quoted values in meteor context
        assert!(is_valid_meteor_format("key=\"value;;; with semicolons\""));
        assert!(is_valid_meteor_format(
            "app:ui:message=\"Hello; World\"; theme=dark"
        ));
        assert!(is_valid_meteor_format(
            "key1=\"val;ue\"; key2=\"another;;value\""
        ));

        // Quoted values in meteor shower
        assert!(is_valid_meteor_shower_format(
            "app:ui:message=\"Hello;; World\" :;: user:data=\"test;;; value\""
        ));
    }

    #[test]
    fn test_smart_semicolon_splitting() {
        let result =
            crate::parser::split::smart_split_semicolons("key=value; message=\"hello; world\"; theme=dark").unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "key=value");
        assert_eq!(result[1], " message=\"hello; world\"");
        assert_eq!(result[2], " theme=dark");

        let result =
            crate::parser::split::smart_split_semicolons("message=\"value;;; with lots; of semicolons\"").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "message=\"value;;; with lots; of semicolons\"");

        // Test unclosed quotes
        assert!(crate::parser::split::smart_split_semicolons("key=\"unclosed quote").is_none());
        assert!(crate::parser::split::smart_split_semicolons("key=value; message=\"unclosed").is_none());
    }

    #[test]
    fn test_consecutive_semicolons_outside_quotes() {
        // Consecutive semicolons outside quotes should be invalid
        assert!(has_consecutive_semicolons_outside_quotes(
            "key=value;; theme=dark"
        ));
        assert!(has_consecutive_semicolons_outside_quotes(
            "key=value;;; theme=dark"
        ));

        // Consecutive semicolons inside quotes should be OK
        assert!(!has_consecutive_semicolons_outside_quotes(
            "key=\"value;;; inside quotes\""
        ));
        assert!(!has_consecutive_semicolons_outside_quotes(
            "message=\"hello;; world\"; theme=dark"
        ));

        // Mixed case: semicolons inside quotes OK, but consecutive outside should fail
        assert!(has_consecutive_semicolons_outside_quotes(
            "message=\"hello; world\";; theme=dark"
        ));
    }
}
