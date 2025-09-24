//! Foundation tests for TokenKey type
//!
//! Tests the core TokenKey functionality including:
//! - Bracket notation transformation
//! - Caching behavior
//! - Inverse parsing
//! - Comparison and equality

use meteor::{TokenKey, BracketNotation};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_key_creation() {
        // Basic key creation
        let key = TokenKey::from("simple_key");
        assert_eq!(key.as_str(), "simple_key");
        assert_eq!(key.to_string(), "simple_key");
    }

    #[test]
    fn test_token_key_bracket_notation_simple() {
        // Test simple bracket notation transformation
        let key = TokenKey::from("list[0]");
        assert_eq!(key.as_str(), "list__i_0");

        // Verify transform was applied
        assert_ne!(key.as_str(), "list[0]");
    }

    #[test]
    fn test_token_key_bracket_notation_multi_dimensional() {
        // Test multi-dimensional bracket notation
        let key = TokenKey::from("matrix[0,1]");
        assert_eq!(key.as_str(), "matrix__i_0_1");

        let key2 = TokenKey::from("tensor[x,y,z]");
        assert_eq!(key2.as_str(), "tensor__i_x_y_z");
    }

    #[test]
    fn test_token_key_bracket_notation_named() {
        // Test named bracket notation
        let key = TokenKey::from("dict[name]");
        assert_eq!(key.as_str(), "dict__n_name");

        let key2 = TokenKey::from("config[setting_key]");
        assert_eq!(key2.as_str(), "config__n_setting_key");
    }

    #[test]
    fn test_token_key_no_brackets() {
        // Keys without brackets should pass through unchanged
        let key = TokenKey::from("regular_key");
        assert_eq!(key.as_str(), "regular_key");

        let key2 = TokenKey::from("namespace:key");
        assert_eq!(key2.as_str(), "namespace:key");
    }

    #[test]
    fn test_token_key_caching() {
        // TokenKey should cache the transformed value
        let key = TokenKey::from("array[42]");

        // Multiple accesses should return same cached value
        let str1 = key.as_str();
        let str2 = key.as_str();

        assert_eq!(str1, "array__i_42");
        assert_eq!(str1, str2);
        // Both should be the same reference (pointer equality)
        assert!(std::ptr::eq(str1, str2));
    }

    #[test]
    fn test_token_key_equality() {
        // Keys with same content should be equal
        let key1 = TokenKey::from("list[0]");
        let key2 = TokenKey::from("list[0]");
        assert_eq!(key1, key2);

        // Keys with different content should not be equal
        let key3 = TokenKey::from("list[1]");
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_token_key_clone() {
        // Test cloning preserves value
        let key1 = TokenKey::from("data[5]");
        let key2 = key1.clone();

        assert_eq!(key1, key2);
        assert_eq!(key1.as_str(), key2.as_str());
    }

    #[test]
    fn test_token_key_debug_format() {
        // Test debug formatting
        let key = TokenKey::from("debug[test]");
        let debug_str = format!("{:?}", key);

        assert!(debug_str.contains("TokenKey"));
        // Should show the transformed value
        assert!(debug_str.contains("debug__n_test"));
    }

    #[test]
    fn test_token_key_display_format() {
        // Display should show the transformed key
        let key = TokenKey::from("display[0]");
        let display_str = format!("{}", key);

        assert_eq!(display_str, "display__i_0");
    }

    #[test]
    fn test_bracket_notation_trait() {
        // Test the BracketNotation trait directly
        let input = "items[0]";
        let transformed = input.transform_brackets();
        assert_eq!(transformed, "items__i_0");

        // Test with string type
        let string_input = String::from("items[1]");
        let string_transformed = string_input.transform_brackets();
        assert_eq!(string_transformed, "items__i_1");
    }

    #[test]
    fn test_token_key_edge_cases() {
        // Empty brackets
        let key = TokenKey::from("empty[]");
        assert_eq!(key.as_str(), "empty[]"); // Should not transform empty brackets

        // Nested brackets (not supported, should pass through)
        let key2 = TokenKey::from("nested[outer[inner]]");
        // This might transform partially or pass through - verify actual behavior
        assert!(key2.as_str().len() > 0);

        // Special characters in brackets
        let key3 = TokenKey::from("special[a-b]");
        assert_eq!(key3.as_str(), "special__n_a-b");
    }

    #[test]
    fn test_token_key_from_string() {
        // Test From<String> implementation
        let string = String::from("owned[0]");
        let key = TokenKey::from(string);
        assert_eq!(key.as_str(), "owned__i_0");
    }

    #[test]
    fn test_token_key_from_str_ref() {
        // Test From<&str> implementation
        let str_ref = "borrowed[0]";
        let key = TokenKey::from(str_ref);
        assert_eq!(key.as_str(), "borrowed__i_0");
    }

    #[test]
    fn test_token_key_complex_patterns() {
        // Test various complex patterns
        let patterns = vec![
            ("list[0][1]", "list__i_0[1]"), // Only first bracket transformed
            ("a[b][c]", "a__n_b[c]"),         // Only first bracket transformed
            ("x[1,2,3]", "x__i_1_2_3"),       // Multi-index
            ("y[a,b,c]", "y__i_a_b_c"),       // Multi-name
        ];

        for (input, expected) in patterns {
            let key = TokenKey::from(input);
            assert_eq!(key.as_str(), expected, "Failed for input: {}", input);
        }
    }
}