//! Utils Module Sanity Tests
//!
//! RSB-compliant sanity tests for the utils module.
//! Validates data flow ordinality: parse → transform → organize → access.

extern crate meteor;

#[cfg(test)]
mod tests {
    use meteor::parser::{parse, transform, organize};
    use meteor::utils::access;

    #[test]
    fn sanity_utils_parse_basic() {
        let result = parse::parse_token_stream("key=value");
        assert!(result.is_ok());
        let bucket = result.unwrap();
        assert_eq!(bucket.get("", "key"), Some("value"));
    }

    #[test]
    fn sanity_utils_parse_complex() {
        let input = "list[0]=first; ui:button=\"Save File\"; ctx=user; data=test";
        let result = parse::parse_token_stream(input);
        assert!(result.is_ok());
        let bucket = result.unwrap();

        // Should be in user context
        assert_eq!(bucket.context().name(), "user");
        assert_eq!(bucket.get("", "data"), Some("test"));
    }

    #[test]
    fn sanity_utils_transform_key() {
        // Basic transformation
        assert_eq!(transform::transform_key("simple").unwrap(), "simple");
        assert_eq!(transform::transform_key("list[0]").unwrap(), "list__i_0");
        assert_eq!(transform::transform_key("grid[x,y]").unwrap(), "grid__i_x_y");
        assert_eq!(transform::transform_key("queue[]").unwrap(), "queue__i_APPEND");
    }

    #[test]
    fn sanity_utils_transform_token() {
        let (key, value) = transform::transform_token("list[0]=item").unwrap();
        assert_eq!(key, "list__i_0");
        assert_eq!(value, "item");

        let (key, value) = transform::transform_token("simple=value").unwrap();
        assert_eq!(key, "simple");
        assert_eq!(value, "value");
    }

    #[test]
    fn sanity_utils_transform_batch() {
        let result = transform::transform_token_batch("key1=value1; list[0]=item; ctx=app; key2=value2").unwrap();

        assert_eq!(result.len(), 3); // ctx=app is skipped
        assert_eq!(result[0], ("key1".to_string(), "value1".to_string()));
        assert_eq!(result[1], ("list__i_0".to_string(), "item".to_string()));
        assert_eq!(result[2], ("key2".to_string(), "value2".to_string()));
    }

    #[test]
    fn sanity_utils_transform_helpers() {
        assert!(!transform::needs_transformation("simple"));
        assert!(transform::needs_transformation("list[0]"));

        assert_eq!(transform::extract_base_name("simple").unwrap(), "simple");
        assert_eq!(transform::extract_base_name("list[0]").unwrap(), "list");
    }

    #[test]
    fn sanity_utils_organize_tokens() {
        let tokens = vec![
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
        ];

        let bucket = organize::organize_tokens(tokens).unwrap();
        assert_eq!(bucket.get("", "key1"), Some("value1"));
        assert_eq!(bucket.get("", "key2"), Some("value2"));
    }

    #[test]
    fn sanity_utils_organize_namespaced() {
        let tokens = vec![
            ("".to_string(), "key1".to_string(), "value1".to_string()),
            ("ui".to_string(), "button".to_string(), "click".to_string()),
        ];

        let bucket = organize::organize_namespaced_tokens(tokens).unwrap();
        assert_eq!(bucket.get("", "key1"), Some("value1"));
        assert_eq!(bucket.get("ui", "button"), Some("click"));
    }

    #[test]
    fn sanity_utils_organize_contextual() {
        let tokens = vec![
            organize::create_contextual_token("", "key1", "value1"),
            organize::create_context_switch("user"),
            organize::create_contextual_token("ui", "button", "save"),
        ];

        let bucket = organize::organize_contextual_tokens(tokens).unwrap();

        // Should be in user context now
        assert_eq!(bucket.context().name(), "user");
        assert_eq!(bucket.get("ui", "button"), Some("save"));
    }

    #[test]
    fn sanity_utils_organize_validation() {
        let valid_tokens = vec![
            organize::create_contextual_token("ui", "button", "click"),
            organize::create_context_switch("user"),
        ];

        assert!(organize::validate_token_organization(&valid_tokens).is_ok());

        let invalid_tokens = vec![
            organize::create_contextual_token("ui", "", "click"), // empty key
        ];

        assert!(organize::validate_token_organization(&invalid_tokens).is_err());
    }

    #[test]
    fn sanity_utils_access_basic() {
        let bucket = parse::parse_token_stream("key=value; ui:button=click").unwrap();

        assert_eq!(access::get_value(&bucket, "", "key"), Some("value".to_string()));
        assert_eq!(access::get_value(&bucket, "ui", "button"), Some("click".to_string()));
        assert_eq!(access::get_value(&bucket, "", "missing"), None);
    }

    #[test]
    fn sanity_utils_access_with_defaults() {
        let bucket = parse::parse_token_stream("key=value").unwrap();

        assert_eq!(access::get_value_or(&bucket, "", "key", "default"), "value");
        assert_eq!(access::get_value_or(&bucket, "", "missing", "default"), "default");
    }

    #[test]
    fn sanity_utils_access_namespace_values() {
        let bucket = parse::parse_token_stream("ui:button=click; ui:label=text; data:key=value").unwrap();

        let ui_values = access::get_namespace_values(&bucket, "ui");
        assert_eq!(ui_values.len(), 2);
        assert_eq!(ui_values.get("button"), Some(&"click".to_string()));
        assert_eq!(ui_values.get("label"), Some(&"text".to_string()));
    }

    #[test]
    fn sanity_utils_access_pattern_matching() {
        let bucket = parse::parse_token_stream("user_name=john; user_email=john@example.com; app_version=1.0").unwrap();

        let user_keys = access::find_keys_matching(&bucket, "", "user");
        assert_eq!(user_keys.len(), 2);
        assert!(user_keys.contains(&"user_name".to_string()));
        assert!(user_keys.contains(&"user_email".to_string()));
    }

    #[test]
    fn sanity_utils_access_bracket_keys() {
        let bucket = parse::parse_token_stream("list[0]=item1; list[1]=item2; simple=value").unwrap();

        let bracket_keys = access::find_bracket_keys(&bucket, "");
        assert_eq!(bracket_keys.len(), 2);
        assert!(bracket_keys.contains(&"list__i_0".to_string()));
        assert!(bracket_keys.contains(&"list__i_1".to_string()));
    }

    #[test]
    fn sanity_utils_access_array_values() {
        let bucket = parse::parse_token_stream("list[0]=first; list[2]=third; list[1]=second").unwrap();

        let array_values = access::get_array_values(&bucket, "", "list");
        assert_eq!(array_values.len(), 3);

        // Should be sorted by index
        assert_eq!(array_values[0], ("0".to_string(), "first".to_string()));
        assert_eq!(array_values[1], ("1".to_string(), "second".to_string()));
        assert_eq!(array_values[2], ("2".to_string(), "third".to_string()));
    }

    #[test]
    fn sanity_utils_access_bucket_stats() {
        let bucket = parse::parse_token_stream("key=value; ui:button=click; ui:label=text").unwrap();

        let stats = access::get_bucket_stats(&bucket);
        assert_eq!(stats.context_name, "app");
        assert_eq!(stats.total_tokens, 3);
        assert_eq!(stats.namespace_count, 2);
        assert!(stats.namespaces.contains(&"".to_string()));
        assert!(stats.namespaces.contains(&"ui".to_string()));
    }

    #[test]
    fn sanity_utils_access_existence_checks() {
        let bucket = parse::parse_token_stream("key=value; ui:button=click").unwrap();

        assert!(access::has_value(&bucket, "", "key"));
        assert!(!access::has_value(&bucket, "", "missing"));

        assert!(access::has_namespace(&bucket, "ui"));
        assert!(!access::has_namespace(&bucket, "missing"));
    }

    #[test]
    fn sanity_utils_access_cross_namespace() {
        let bucket = parse::parse_token_stream("title=main; ui:title=header; data:title=content").unwrap();

        let titles = access::get_key_across_namespaces(&bucket, "title");
        assert_eq!(titles.len(), 3);
        assert_eq!(titles.get(""), Some(&"main".to_string()));
        assert_eq!(titles.get("ui"), Some(&"header".to_string()));
        assert_eq!(titles.get("data"), Some(&"content".to_string()));
    }
}