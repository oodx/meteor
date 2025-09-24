//! Sup Module Sanity Tests
//!
//! RSB-compliant sanity tests for the sup (support) module.
//! Validates internal complexity isolation: bracket notation algorithms.

extern crate meteor;

#[cfg(test)]
mod tests {
    use meteor::types::{transform_key, reverse_transform_key};

    // Create a bracket module alias for compatibility with existing tests
    mod bracket {
        pub use meteor::types::{transform_key, reverse_transform_key, has_brackets, extract_base_name};
    }

    #[test]
    fn sanity_sup_bracket_transform_simple() {
        assert_eq!(bracket::transform_key("list[0]").unwrap(), "list__i_0");
        assert_eq!(bracket::transform_key("items[42]").unwrap(), "items__i_42");
    }

    #[test]
    fn sanity_sup_bracket_transform_multiple() {
        assert_eq!(bracket::transform_key("grid[2,3]").unwrap(), "grid__i_2_3");
        assert_eq!(bracket::transform_key("matrix[x,y,z]").unwrap(), "matrix__i_x_y_z");
    }

    #[test]
    fn sanity_sup_bracket_transform_empty() {
        assert_eq!(bracket::transform_key("queue[]").unwrap(), "queue__i_APPEND");
        assert_eq!(bracket::transform_key("list[]").unwrap(), "list__i_APPEND");
    }

    #[test]
    fn sanity_sup_bracket_transform_no_brackets() {
        assert_eq!(bracket::transform_key("normal_key").unwrap(), "normal_key");
        assert_eq!(bracket::transform_key("snake_case").unwrap(), "snake_case");
    }

    #[test]
    fn sanity_sup_bracket_transform_errors() {
        // Missing brackets
        assert!(bracket::transform_key("list[").is_err());
        assert!(bracket::transform_key("list]").is_err());

        // Empty base name
        assert!(bracket::transform_key("[0]").is_err());

        // Nested brackets (not supported)
        assert!(bracket::transform_key("list[grid[0]]").is_err());
    }

    #[test]
    fn sanity_sup_bracket_has_brackets() {
        assert!(bracket::has_brackets("list[0]"));
        assert!(bracket::has_brackets("grid[x,y]"));
        assert!(!bracket::has_brackets("normal_key"));
        assert!(!bracket::has_brackets("key_with_underscore"));
    }

    #[test]
    fn sanity_sup_bracket_extract_base_name() {
        assert_eq!(bracket::extract_base_name("list[0]").unwrap(), "list");
        assert_eq!(bracket::extract_base_name("grid[x,y]").unwrap(), "grid");
        assert_eq!(bracket::extract_base_name("normal_key").unwrap(), "normal_key");
    }

    #[test]
    fn sanity_sup_bracket_whitespace_handling() {
        assert_eq!(bracket::transform_key("list[ 0 ]").unwrap(), "list__i_0");
        assert_eq!(bracket::transform_key("grid[ x , y ]").unwrap(), "grid__i_x_y");
    }

    #[test]
    fn sanity_sup_bracket_index_validation() {
        // Valid indices
        assert!(bracket::transform_key("list[0]").is_ok());
        assert!(bracket::transform_key("list[variable_name]").is_ok());
        assert!(bracket::transform_key("list[x]").is_ok());

        // Invalid characters in indices would be caught by transform_key
        // These tests verify the overall transformation pipeline
        assert_eq!(bracket::transform_key("list[test-key]").unwrap(), "list__i_test-key");
        assert_eq!(bracket::transform_key("list[_underscore]").unwrap(), "list__i__underscore");
    }

    #[test]
    fn sanity_sup_bracket_complex_cases() {
        // Multiple comma-separated indices
        assert_eq!(bracket::transform_key("multi[a,b,c,d]").unwrap(), "multi__i_a_b_c_d");

        // Mix of numeric and string indices
        assert_eq!(bracket::transform_key("mixed[0,name,42]").unwrap(), "mixed__i_0_name_42");

        // Single character indices
        assert_eq!(bracket::transform_key("coords[x]").unwrap(), "coords__i_x");
    }
}