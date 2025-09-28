//! ENG-42 Regression Tests - Centralized Smart-Split Edge Cases
//!
//! Comprehensive regression coverage for escaped quotes, control tokens,
//! and edge cases in the centralized smart-split implementation.

use meteor::parser::split::{smart_split, smart_split_multi_char, smart_split_semicolons, SplitConfig};

#[test]
fn test_escaped_quotes_various_contexts() {
    // Test escaped quotes in different delimiter contexts
    let config = SplitConfig::general_parsing(';');

    // Basic escaped quotes
    let result = smart_split("key=\"value with \\\"quotes\\\"\"; theme=dark", config.clone());
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "key=\"value with \\\"quotes\\\"\"");
    assert_eq!(result[1], "theme=dark");

    // Multiple escaped quotes
    let result = smart_split("message=\"\\\"Hello\\\", said \\\"world\\\"\"; status=ok", config.clone());
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "message=\"\\\"Hello\\\", said \\\"world\\\"\"");
    assert_eq!(result[1], "status=ok");

    // Escaped quotes at boundaries
    let result = smart_split("start=\"\\\"beginning\"; end=\"ending\\\"\"; middle=value", config);
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], "start=\"\\\"beginning\"");
    assert_eq!(result[1], "end=\"ending\\\"\"");
    assert_eq!(result[2], "middle=value");
}

#[test]
fn test_escaped_quotes_meteor_style() {
    // Test meteor-style escaping (only inside quotes)
    let config = SplitConfig::meteor_streams(';');

    let result = smart_split("key=\"value with \\\"inner quotes\\\"\"; theme=dark", config.clone());
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "key=\"value with \\\"inner quotes\\\"\"");
    assert_eq!(result[1], "theme=dark");

    // Escaping outside quotes should be treated differently
    let result = smart_split("key=value\\; theme=\"dark with \\\"quotes\\\"\"", config);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "key=value\\");
    assert_eq!(result[1], "theme=\"dark with \\\"quotes\\\"\"");
}

#[test]
fn test_control_tokens_preservation() {
    // Control tokens should be preserved exactly through splitting
    let config = SplitConfig::general_parsing(';');

    let result = smart_split("button=click; ns=ui; ctl:delete=app:main:test; theme=dark", config.clone());
    assert_eq!(result.len(), 4);
    assert_eq!(result[0], "button=click");
    assert_eq!(result[1], "ns=ui");
    assert_eq!(result[2], "ctl:delete=app:main:test");
    assert_eq!(result[3], "theme=dark");

    // Control tokens with complex paths
    let result = smart_split("ctl:reset=cursor; data=value; ctl:delete=app:nested:complex:path[0]", config.clone());
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], "ctl:reset=cursor");
    assert_eq!(result[1], "data=value");
    assert_eq!(result[2], "ctl:delete=app:nested:complex:path[0]");

    // Control tokens in quoted values (should not be interpreted as control)
    let result = smart_split("message=\"Use ctl:delete=path to delete\"; action=ctl:reset=all", config);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "message=\"Use ctl:delete=path to delete\"");
    assert_eq!(result[1], "action=ctl:reset=all");
}

#[test]
fn test_complex_bracket_notation_edge_cases() {
    let config = SplitConfig::general_parsing(';');

    // Nested brackets
    let result = smart_split("matrix[outer[inner]]=value; simple=test", config.clone());
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "matrix[outer[inner]]=value");
    assert_eq!(result[1], "simple=test");

    // Brackets with quotes
    let result = smart_split("config[\"key with spaces\"]=value; theme=dark", config.clone());
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "config[\"key with spaces\"]=value");
    assert_eq!(result[1], "theme=dark");

    // Multiple dimensional arrays
    let result = smart_split("grid[1,2,3]=cell; tensor[x,y,z,t]=spacetime", config);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "grid[1,2,3]=cell");
    assert_eq!(result[1], "tensor[x,y,z,t]=spacetime");
}

#[test]
fn test_meteor_delimiter_edge_cases() {
    let config = SplitConfig::meteor_streams(':');

    // Meteor delimiter in quotes should not split
    let result = smart_split_multi_char(
        "app:ui:message=\"This contains :;: delimiter\" :;: app:ui:theme=dark",
        ":;:",
        config.clone()
    );
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "app:ui:message=\"This contains :;: delimiter\"");
    assert_eq!(result[1], "app:ui:theme=dark");

    // Partial delimiter matches should not split
    let result = smart_split_multi_char(
        "app:ui:test=:value :;: app:ui:other=;value",
        ":;:",
        config.clone()
    );
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "app:ui:test=:value");
    assert_eq!(result[1], "app:ui:other=;value");

    // Empty segments should be filtered
    let result = smart_split_multi_char(
        "app:ui:first=value :;: :;: app:ui:second=value",
        ":;:",
        config
    );
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "app:ui:first=value");
    assert_eq!(result[1], "app:ui:second=value");
}

#[test]
fn test_quote_escaping_regression() {
    // Regression test for quote handling edge cases

    // Escaped backslashes before quotes
    let config = SplitConfig::general_parsing(';');
    let result = smart_split("path=\"C:\\\\Program Files\\\\\"; theme=dark", config.clone());
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "path=\"C:\\\\Program Files\\\\\"");
    assert_eq!(result[1], "theme=dark");

    // Sequential escaped quotes
    let result = smart_split("test=\"\\\"\\\"multiple quotes\\\"\\\"\"; value=ok", config.clone());
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "test=\"\\\"\\\"multiple quotes\\\"\\\"\"");
    assert_eq!(result[1], "value=ok");

    // Mixed escaping
    let result = smart_split("mixed=\"text \\\"quoted\\\" and \\\\backslash\"; done=true", config);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "mixed=\"text \\\"quoted\\\" and \\\\backslash\"");
    assert_eq!(result[1], "done=true");
}

#[test]
fn test_unclosed_quotes_error_handling() {
    // Test that unclosed quotes are properly detected

    let result = smart_split_semicolons("key=\"unclosed quote");
    assert!(result.is_none(), "Should return None for unclosed quotes");

    let result = smart_split_semicolons("key=value; message=\"unclosed");
    assert!(result.is_none(), "Should return None for unclosed quotes in middle");

    let result = smart_split_semicolons("key=\"properly closed\"; value=ok");
    assert!(result.is_some(), "Should succeed with properly closed quotes");
}

#[test]
fn test_empty_and_whitespace_handling() {
    let config = SplitConfig::general_parsing(';');

    // Empty segments
    let result = smart_split(";;key=value;;theme=dark;;", config.clone());
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "key=value");
    assert_eq!(result[1], "theme=dark");

    // Whitespace-only segments
    let result = smart_split("; ; key=value ; ; theme=dark ; ;", config.clone());
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "key=value");
    assert_eq!(result[1], "theme=dark");

    // Mixed empty and content
    let result = smart_split("first=value; ; ; second=other; ; ;", config);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "first=value");
    assert_eq!(result[1], "second=other");
}

#[test]
fn test_special_characters_in_values() {
    let config = SplitConfig::general_parsing(';');

    // Special shell characters
    let result = smart_split("command=\"rm -rf /tmp/*\"; safe=true", config.clone());
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "command=\"rm -rf /tmp/*\"");
    assert_eq!(result[1], "safe=true");

    // Unicode characters
    let result = smart_split("message=\"Hello 世界 🌍\"; lang=unicode", config.clone());
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "message=\"Hello 世界 🌍\"");
    assert_eq!(result[1], "lang=unicode");

    // JSON-like structures
    let result = smart_split("json=\"{\\\"key\\\": \\\"value\\\", \\\"nested\\\": {\\\"inner\\\": 42}}\"; format=json", config);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "json=\"{\\\"key\\\": \\\"value\\\", \\\"nested\\\": {\\\"inner\\\": 42}}\"");
    assert_eq!(result[1], "format=json");
}

#[test]
fn test_meteor_format_compliance() {
    // Ensure split results can be used to construct valid meteors
    let config = SplitConfig::meteor_streams(';');

    let result = smart_split_multi_char(
        "app:ui:button=click :;: app:ui:theme=\"dark with spaces\" :;: user:profile:name=\"John Doe\"",
        ":;:",
        SplitConfig::meteor_streams(':')
    );

    assert_eq!(result.len(), 3);

    // Each result should be a valid meteor format
    for meteor_str in &result {
        // Each should contain exactly 2 colons (context:namespace:key=value)
        let colon_count = meteor_str.matches(':').count();
        assert!(colon_count >= 2, "Meteor format should have at least 2 colons: {}", meteor_str);

        // Should contain exactly one equals sign
        let equals_count = meteor_str.matches('=').count();
        assert_eq!(equals_count, 1, "Meteor should have exactly one equals sign: {}", meteor_str);
    }
}

#[test]
fn test_performance_with_large_inputs() {
    // Test that split operations perform reasonably with larger inputs
    let config = SplitConfig::general_parsing(';');

    // Generate a large input with many segments
    let mut large_input = String::new();
    for i in 0..1000 {
        if i > 0 {
            large_input.push(';');
        }
        large_input.push_str(&format!("key{}=value{}", i, i));
    }

    let result = smart_split(&large_input, config);
    assert_eq!(result.len(), 1000);
    assert_eq!(result[0], "key0=value0");
    assert_eq!(result[999], "key999=value999");
}

#[test]
fn test_compatibility_with_legacy_parsers() {
    // Ensure centralized split produces same results as legacy implementations

    // Test compatibility with validators.rs style
    let input = "key=value; message=\"hello; world\"; theme=dark";
    let result = smart_split_semicolons(input).unwrap();

    // Should match expected legacy behavior (preserving whitespace)
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], "key=value");
    assert!(result[1].contains("message="));
    assert!(result[2].contains("theme="));

    // Test compatibility with token_stream.rs style
    let config = SplitConfig::general_parsing(';');
    let result = smart_split(input, config);
    assert_eq!(result.len(), 3);
    // Should be trimmed
    assert_eq!(result[0], "key=value");
    assert_eq!(result[1], "message=\"hello; world\"");
    assert_eq!(result[2], "theme=dark");
}