//! Meteor Core Sanity Tests
//!
//! RSB-compliant sanity tests for the main meteor module.
//! These tests validate core functionality with no ceremony - following RSB patterns.

extern crate meteor;

#[cfg(test)]
mod tests {
    use meteor::{parse_token_stream, Context};

    #[test]
    fn sanity_meteor_basic_compilation() {
        // Basic test infrastructure validation
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn sanity_meteor_basic_token_parsing() {
        let result = meteor::parse_token_stream("key=value");
        assert!(result.is_ok());
        let bucket = result.unwrap();
        assert_eq!(bucket.get("", "key"), Some("value"));
    }

    #[test]
    fn sanity_meteor_namespaced_parsing() {
        let result = meteor::parse_token_stream("ui:button=click");
        assert!(result.is_ok());
        let bucket = result.unwrap();
        assert_eq!(bucket.get("ui", "button"), Some("click"));
    }

    #[test]
    fn sanity_meteor_bracket_notation() {
        // Test basic bracket transformation
        let result = meteor::parse_token_stream("list[0]=item");
        assert!(result.is_ok());
        let bucket = result.unwrap();
        assert_eq!(bucket.get("", "list__i_0"), Some("item"));

        // Test multiple indices
        let result = meteor::parse_token_stream("grid[2,3]=cell");
        assert!(result.is_ok());
        let bucket = result.unwrap();
        assert_eq!(bucket.get("", "grid__i_2_3"), Some("cell"));

        // Test empty brackets (append)
        let result = meteor::parse_token_stream("queue[]=append");
        assert!(result.is_ok());
        let bucket = result.unwrap();
        assert_eq!(bucket.get("", "queue__i_APPEND"), Some("append"));
    }

    #[test]
    fn sanity_meteor_context_isolation() {
        // Parse tokens with context switches
        let result = meteor::parse_token_stream("ctx=app; ui:button=click; ctx=user; ui:button=save");
        assert!(result.is_ok());

        let mut bucket = result.unwrap();

        // Current context should be user (last switched)
        assert_eq!(bucket.context().name(), "user");
        assert_eq!(bucket.get("ui", "button"), Some("save"));

        // Switch to app context to see app data
        bucket.switch_context(Context::app());
        assert_eq!(bucket.get("ui", "button"), Some("click"));
    }

    #[test]
    fn sanity_meteor_value_parsing_quotes() {
        // Double quoted values
        let result = meteor::parse_token_stream("message=\"hello world\"");
        assert!(result.is_ok());
        let bucket = result.unwrap();
        assert_eq!(bucket.get("", "message"), Some("hello world"));

        // Single quoted values
        let result = meteor::parse_token_stream("name='John Doe'");
        assert!(result.is_ok());
        let bucket = result.unwrap();
        assert_eq!(bucket.get("", "name"), Some("John Doe"));
    }

    #[test]
    fn sanity_meteor_value_parsing_escapes() {
        // Escaped quotes
        let result = meteor::parse_token_stream("text=\"She said \\\"hello\\\"\"");
        assert!(result.is_ok());
        let bucket = result.unwrap();
        assert_eq!(bucket.get("", "text"), Some("She said \"hello\""));

        // Escaped backslashes
        let result = meteor::parse_token_stream("path=\"C:\\\\Program Files\\\\\"");
        assert!(result.is_ok());
        let bucket = result.unwrap();
        assert_eq!(bucket.get("", "path"), Some("C:\\Program Files\\"));
    }

    #[test]
    fn sanity_meteor_error_handling() {
        // Empty input
        let result = meteor::parse_token_stream("");
        assert!(result.is_err());

        // Invalid token format
        let result = meteor::parse_token_stream("invalid");
        assert!(result.is_err());

        // Namespace too deep
        let result = meteor::parse_token_stream("ui.widgets.buttons.primary:icon=arrow");
        assert!(result.is_err());
    }

    #[test]
    fn sanity_meteor_multiple_tokens() {
        let result = meteor::parse_token_stream("key1=value1; key2=value2; ui:button=click");
        assert!(result.is_ok());
        let bucket = result.unwrap();
        assert_eq!(bucket.get("", "key1"), Some("value1"));
        assert_eq!(bucket.get("", "key2"), Some("value2"));
        assert_eq!(bucket.get("ui", "button"), Some("click"));
        assert_eq!(bucket.len(), 3);
    }
}