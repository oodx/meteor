//! Validation utilities for Meteor token formats
//!
//! Provides helper functions to validate token, meteor, and meteor shower string formats
//! using format validators from utils module.

use crate::utils::validators;

/// Check if a string is a valid token format
///
/// Returns true if the string has valid token format.
/// Format: "key=value" or with bracket notation "list[0]=item"
///
/// # Examples
/// ```
/// use meteor::validation::is_valid_token;
///
/// assert!(is_valid_token("button=click"));
/// assert!(is_valid_token("list[0]=item"));
/// assert!(!is_valid_token("invalid"));
/// assert!(!is_valid_token(""));
/// ```
pub fn is_valid_token(s: &str) -> bool {
    validators::is_valid_token_format(s)
}

/// Check if a string is a valid meteor format
///
/// Returns true if the string has valid meteor format.
/// Format: "context:namespace:key=value; key2=value2" or "key=value" (defaults to "app" context)
///
/// # Examples
/// ```
/// use meteor::validation::is_valid_meteor;
///
/// assert!(is_valid_meteor("app:ui:button=click"));
/// assert!(is_valid_meteor("button=click")); // defaults to "app" context
/// assert!(is_valid_meteor("app:ui:button=click; theme=dark"));
/// assert!(!is_valid_meteor("app:ui:button")); // missing value
/// assert!(!is_valid_meteor(""));
/// ```
pub fn is_valid_meteor(s: &str) -> bool {
    validators::is_valid_meteor_format(s)
}

/// Check if a string is a valid meteor shower format
///
/// Returns true if the string has valid meteor shower format.
/// Format: Multiple meteors separated by `:;:` delimiter
/// Also validates no consecutive semicolons within meteors.
///
/// # Examples
/// ```
/// use meteor::validation::is_valid_meteor_shower;
///
/// assert!(is_valid_meteor_shower("app:ui:button=click"));
/// assert!(is_valid_meteor_shower("app:ui:button=click :;: user:settings:theme=dark"));
/// assert!(!is_valid_meteor_shower("app:ui:button=click;; theme=dark")); // consecutive semicolons
/// assert!(!is_valid_meteor_shower(""));
/// ```
pub fn is_valid_meteor_shower(s: &str) -> bool {
    validators::is_valid_meteor_shower_format(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_tokens() {
        assert!(is_valid_token("button=click"));
        assert!(is_valid_token("list[0]=item"));
        assert!(is_valid_token("grid[x,y]=value"));
        assert!(is_valid_token("key=value with spaces"));
    }

    #[test]
    fn test_invalid_tokens() {
        assert!(!is_valid_token(""));
        assert!(!is_valid_token("just_key"));
        assert!(!is_valid_token("=value"));
        // Note: key= with empty value is actually valid according to validators
    }

    #[test]
    fn test_valid_meteors() {
        assert!(is_valid_meteor("app:ui:button=click"));
        assert!(is_valid_meteor("button=click")); // defaults to app context
        assert!(is_valid_meteor("app:ui:button=click; theme=dark"));
        assert!(is_valid_meteor("user:settings:profile=admin; role=moderator"));
    }

    #[test]
    fn test_invalid_meteors() {
        assert!(!is_valid_meteor(""));
        assert!(!is_valid_meteor("app:ui:button")); // missing value
        assert!(!is_valid_meteor("app:ui:")); // missing key and value
    }

    #[test]
    fn test_valid_meteor_showers() {
        assert!(is_valid_meteor_shower("app:ui:button=click"));
        assert!(is_valid_meteor_shower("app:ui:button=click :;: user:settings:theme=dark"));
        assert!(is_valid_meteor_shower("button=click :;: theme=dark :;: size=large"));
    }

    #[test]
    fn test_invalid_meteor_showers() {
        assert!(!is_valid_meteor_shower(""));
        assert!(!is_valid_meteor_shower("app:ui:button=click;; theme=dark")); // consecutive semicolons
        assert!(!is_valid_meteor_shower("app:ui:button=click;;; theme=dark")); // multiple consecutive semicolons
    }

    #[test]
    fn test_semicolon_validation() {
        // Single semicolons are OK (token separators within meteor)
        assert!(is_valid_meteor_shower("app:ui:button=click; theme=dark"));

        // Consecutive semicolons are syntax errors
        assert!(!is_valid_meteor_shower("app:ui:button=click;; theme=dark"));
        assert!(!is_valid_meteor_shower("app:ui:button=click;;; theme=dark"));

        // Properly separated meteors with single semicolons are OK
        assert!(is_valid_meteor_shower("app:ui:button=click; theme=dark :;: user:settings:profile=admin; role=moderator"));
    }
}