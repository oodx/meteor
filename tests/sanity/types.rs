//! Types Module Sanity Tests
//!
//! RSB-compliant sanity tests for the types module.
//! Validates core type functionality: Context, Namespace, Key, TokenBucket, MeteorError.

extern crate meteor;

#[cfg(test)]
mod tests {
    use meteor::{Context, Namespace, Key, TokenBucket, MeteorError};
    use std::str::FromStr;

    #[test]
    fn sanity_types_context_creation() {
        let ctx = Context::new("app");
        assert_eq!(ctx.name(), "app");

        let system = Context::system();
        assert!(system.is_privileged());

        let app = Context::app();
        assert!(!app.is_privileged());

        let user = Context::user();
        assert!(user.is_privileged());
    }

    #[test]
    fn sanity_types_context_from_str() {
        let ctx = Context::from_str("custom").unwrap();
        assert_eq!(ctx.name(), "custom");

        let result = Context::from_str("");
        assert!(result.is_err());
    }

    #[test]
    fn sanity_types_namespace_depth() {
        let root = Namespace::root();
        assert_eq!(root.depth(), 0);
        assert!(!root.should_warn());

        let shallow = Namespace::from_string("ui.widgets");
        assert_eq!(shallow.depth(), 2);
        assert!(!shallow.should_warn());

        let warning_level = Namespace::from_string("ui.widgets.buttons");
        assert_eq!(warning_level.depth(), 3);
        assert!(warning_level.should_warn());

        let deep = Namespace::from_string("ui.widgets.buttons.primary");
        assert_eq!(deep.depth(), 4);
        assert!(deep.should_warn());
        assert!(deep.is_too_deep());
    }

    #[test]
    fn sanity_types_namespace_hierarchy() {
        let parent = Namespace::from_string("ui");
        let child = Namespace::from_string("ui.widgets");
        let unrelated = Namespace::from_string("data");

        assert!(parent.is_parent_of(&child));
        assert!(!child.is_parent_of(&parent));
        assert!(!parent.is_parent_of(&unrelated));
    }

    #[test]
    fn sanity_types_key_creation() {
        let simple = Key::new("button");
        assert_eq!(simple.base(), "button");
        assert_eq!(simple.transformed(), "button");
        assert!(!simple.has_brackets());

        let bracket = Key::new("list[0]");
        assert_eq!(bracket.base(), "list[0]");
        assert_eq!(bracket.transformed(), "list__i_0");
        assert!(bracket.has_brackets());
    }

    #[test]
    fn sanity_types_key_bracket_transformation() {
        let list_key = Key::new("list[0]");
        assert_eq!(list_key.transformed(), "list__i_0");

        let grid_key = Key::new("grid[2,3]");
        assert_eq!(grid_key.transformed(), "grid__i_2_3");

        let append_key = Key::new("queue[]");
        assert_eq!(append_key.transformed(), "queue__i_APPEND");
    }

    #[test]
    fn sanity_types_token_bucket_basic() {
        let mut bucket = TokenBucket::new();

        // Set and get
        bucket.set("", "key", "value".to_string());
        assert_eq!(bucket.get("", "key"), Some("value"));

        // Namespaced
        bucket.set("ui", "button", "click".to_string());
        assert_eq!(bucket.get("ui", "button"), Some("click"));

        // Non-existent
        assert_eq!(bucket.get("ui", "missing"), None);

        // Count
        assert_eq!(bucket.len(), 2);
    }

    #[test]
    fn sanity_types_token_bucket_context_isolation() {
        let mut bucket = TokenBucket::new();

        // Set in app context
        bucket.set("", "key", "app_value".to_string());

        // Switch to user context
        bucket.switch_context(Context::user());

        // Should not see app context data
        assert_eq!(bucket.get("", "key"), None);

        // Set in user context
        bucket.set("", "key", "user_value".to_string());
        assert_eq!(bucket.get("", "key"), Some("user_value"));

        // Switch back to app context
        bucket.switch_context(Context::app());

        // Should see original app data
        assert_eq!(bucket.get("", "key"), Some("app_value"));
    }

    #[test]
    fn sanity_types_token_bucket_namespaces() {
        let mut bucket = TokenBucket::new();
        bucket.set("", "key1", "value1".to_string());
        bucket.set("ui", "button", "click".to_string());
        bucket.set("ui", "label", "text".to_string());

        let namespaces = bucket.namespaces();
        assert_eq!(namespaces.len(), 2);
        assert!(namespaces.contains(&"".to_string()));
        assert!(namespaces.contains(&"ui".to_string()));

        let ui_keys = bucket.keys_in_namespace("ui");
        assert_eq!(ui_keys.len(), 2);
        assert!(ui_keys.contains(&"button".to_string()));
        assert!(ui_keys.contains(&"label".to_string()));
    }

    #[test]
    fn sanity_types_meteor_error_creation() {
        let parse_err = MeteorError::parse(10, "unexpected character");
        assert!(parse_err.to_string().contains("Parse error at position 10"));

        let token_err = MeteorError::invalid_token("bad=", "empty value");
        assert!(token_err.to_string().contains("Invalid token"));

        let depth_err = MeteorError::namespace_too_deep("ui.widgets.buttons.primary", 4);
        assert!(depth_err.to_string().contains("too deep"));
    }

    #[test]
    fn sanity_types_meteor_error_variants() {
        let context_err = MeteorError::context_violation("app", "system", "insufficient privileges");
        assert!(context_err.to_string().contains("Context violation"));

        let bracket_err = MeteorError::invalid_bracket("list[", "unclosed bracket");
        assert!(bracket_err.to_string().contains("Invalid bracket"));

        let char_err = MeteorError::invalid_char('[', 5, "unexpected bracket");
        assert!(char_err.to_string().contains("Invalid character"));

        let empty_err = MeteorError::empty("key");
        assert!(empty_err.to_string().contains("Empty input"));
    }
}