//! Meteor Sanity Tests
//!
//! Core functionality validation with no ceremony - following RSB patterns.
//! These tests validate basic functionality works as expected.

#[cfg(test)]
mod tests {
    // Import meteor functionality
    extern crate meteor;
    use meteor::{parse_token_stream, Context};

    #[test]
    fn test_basic_compilation() {
        // Placeholder test to verify test infrastructure works
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_basic_token_parsing() {
        let result = meteor::parse_token_stream("key=value");
        assert!(result.is_ok());
        let bucket = result.unwrap();
        assert_eq!(bucket.get("", "key"), Some("value"));
    }

    #[test]
    fn test_context_namespace_key_parsing() {
        let result = meteor::parse_token_stream("ui:button=click");
        assert!(result.is_ok());
        let bucket = result.unwrap();
        assert_eq!(bucket.get("ui", "button"), Some("click"));
    }

    #[test]
    fn test_bracket_notation_transformation() {
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
    fn test_context_isolation() {
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
}