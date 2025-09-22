//! Meteor Sanity Tests
//!
//! RSB-compliant sanity test entry point. These tests validate core functionality
//! with no ceremony following RSB testing patterns.
//!
//! Tests are organized in tests/sanity/ subdirectory for detailed functionality.

// Re-export all sanity tests from subdirectory modules
// This allows both `cargo test --test sanity` and `test.sh sanity` to work

extern crate meteor;

#[cfg(test)]
mod tests {
    // Import the actual meteor module to run basic validation
    use meteor::{parse, parse_shower, TokenKey, BracketNotation};

    /// Sanity check: meteor compiles and basic API works
    #[test]
    fn sanity_meteor_compiles_and_basic_api() {
        // Basic compilation and API access test
        let bucket = meteor::parse("key=value").unwrap();
        assert_eq!(bucket.get("", "key"), Some("value"));
    }

    /// Sanity check: bracket notation works
    #[test]
    fn sanity_bracket_notation_basic() {
        let bucket = meteor::parse("list[0]=item").unwrap();
        assert_eq!(bucket.get("", "list__i_0"), Some("item"));
    }

    /// Sanity check: BracketNotation trait works
    #[test]
    fn sanity_bracket_notation_trait() {
        use meteor::BracketNotation;

        let flat = "list__i_0";
        assert_eq!(flat.to_bracket(), "list[0]");
    }

    /// Sanity check: MeteorShower collection works
    #[test]
    fn sanity_meteor_shower_basic() {
        let shower = meteor::parse_shower("app:ui:button=click").unwrap();
        assert_eq!(shower.len(), 1);

        let meteors = shower.by_context("app");
        assert_eq!(meteors.len(), 1);
    }
}