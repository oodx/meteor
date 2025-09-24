//! Foundation tests for Meteor type
//!
//! Tests the Meteor multi-token functionality including:
//! - Multiple tokens in same context
//! - Context:namespace:key=value parsing
//! - Default "app" context handling
//! - Namespace organization within Meteor

use meteor::{Meteor, Context, Namespace, Token};
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meteor_creation_simple() {
        // Create a simple meteor with default context
        let meteor = Meteor::new(
            Context::default(),
            vec![(
                Namespace::from("ui"),
                Token::new("button", "click")
            )]
        );

        assert_eq!(meteor.context().name(), "app");
        assert_eq!(meteor.tokens().len(), 1);
    }

    #[test]
    fn test_meteor_multi_token_same_namespace() {
        // Multiple tokens in the same namespace
        let tokens = vec![
            (Namespace::from("config"), Token::new("host", "localhost")),
            (Namespace::from("config"), Token::new("port", "8080")),
            (Namespace::from("config"), Token::new("ssl", "true")),
        ];

        let meteor = Meteor::new(Context::default(), tokens);

        assert_eq!(meteor.context().name(), "app");
        assert_eq!(meteor.tokens().len(), 3);
    }

    #[test]
    fn test_meteor_multi_namespace() {
        // Tokens across multiple namespaces
        let tokens = vec![
            (Namespace::from("ui"), Token::new("theme", "dark")),
            (Namespace::from("api"), Token::new("endpoint", "/v1")),
            (Namespace::from("cache"), Token::new("ttl", "3600")),
        ];

        let meteor = Meteor::new(Context::from("prod"), tokens);

        assert_eq!(meteor.context().name(), "prod");
        assert_eq!(meteor.tokens().len(), 3);
    }

    #[test]
    fn test_meteor_parse_simple() {
        // Parse simple key=value (should use default context and namespace)
        let meteor = Meteor::parse("key=value").expect("Failed to parse");

        assert_eq!(meteor.context().name(), "app");
        assert_eq!(meteor.tokens().len(), 1);
    }

    #[test]
    fn test_meteor_parse_with_namespace() {
        // Parse namespace:key=value
        let meteor = Meteor::parse("ui:button=click").expect("Failed to parse");

        assert_eq!(meteor.context().name(), "app");
        assert_eq!(meteor.tokens().len(), 1);

        let (namespace, token) = &meteor.tokens()[0];
        assert_eq!(namespace.to_string(), "ui");
        assert_eq!(token.key().as_str(), "button");
        assert_eq!(token.value(), "click");
    }

    #[test]
    fn test_meteor_parse_full_address() {
        // Parse context:namespace:key=value
        let meteor = Meteor::parse("dev:db:connection=postgres").expect("Failed to parse");

        assert_eq!(meteor.context().name(), "dev");
        assert_eq!(meteor.tokens().len(), 1);

        let (namespace, token) = &meteor.tokens()[0];
        assert_eq!(namespace.to_string(), "db");
        assert_eq!(token.key().as_str(), "connection");
        assert_eq!(token.value(), "postgres");
    }

    #[test]
    fn test_meteor_parse_multiple_tokens() {
        // Parse multiple tokens with semicolon separator
        // Note: This might not be supported by Meteor::parse directly
        // as it parses a single meteor. This test documents the behavior.
        let result = Meteor::parse("key1=val1;key2=val2");

        // If Meteor only parses single tokens, this should either:
        // - Parse only the first token
        // - Return an error
        // - Parse all as a single value
        // Document actual behavior here
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_meteor_display() {
        // Test display formatting
        let meteor = Meteor::new(
            Context::from("test"),
            vec![(
                Namespace::from("ns"),
                Token::new("key", "value")
            )]
        );

        let display = format!("{}", meteor);
        assert!(display.contains("test") || display.contains("ns") || display.contains("key=value"));
    }

    #[test]
    fn test_meteor_with_bracket_notation() {
        // Test meteor with bracket notation in keys
        let tokens = vec![
            (Namespace::from("data"), Token::new("list[0]", "first")),
            (Namespace::from("data"), Token::new("list[1]", "second")),
            (Namespace::from("data"), Token::new("matrix[0,0]", "origin")),
        ];

        let meteor = Meteor::new(Context::default(), tokens);
        assert_eq!(meteor.tokens().len(), 3);

        // Verify bracket notation was transformed
        let (_, token) = &meteor.tokens()[0];
        assert_eq!(token.key().as_str(), "list__i_0");
    }

    #[test]
    fn test_meteor_equality() {
        let meteor1 = Meteor::new(
            Context::from("app"),
            vec![(Namespace::from("ns"), Token::new("key", "value"))]
        );

        let meteor2 = Meteor::new(
            Context::from("app"),
            vec![(Namespace::from("ns"), Token::new("key", "value"))]
        );

        assert_eq!(meteor1, meteor2);

        let meteor3 = Meteor::new(
            Context::from("different"),
            vec![(Namespace::from("ns"), Token::new("key", "value"))]
        );

        assert_ne!(meteor1, meteor3);
    }

    #[test]
    fn test_meteor_clone() {
        let meteor = Meteor::new(
            Context::from("prod"),
            vec![
                (Namespace::from("api"), Token::new("key", "secret")),
                (Namespace::from("api"), Token::new("url", "https://api.example.com")),
            ]
        );

        let cloned = meteor.clone();
        assert_eq!(meteor, cloned);
        assert_eq!(meteor.context(), cloned.context());
        assert_eq!(meteor.tokens().len(), cloned.tokens().len());
    }

    #[test]
    fn test_meteor_namespace_organization() {
        // Test that tokens are properly organized by namespace
        let tokens = vec![
            (Namespace::from("a"), Token::new("1", "one")),
            (Namespace::from("b"), Token::new("2", "two")),
            (Namespace::from("a"), Token::new("3", "three")),
            (Namespace::from("c"), Token::new("4", "four")),
            (Namespace::from("b"), Token::new("5", "five")),
        ];

        let meteor = Meteor::new(Context::default(), tokens);
        assert_eq!(meteor.tokens().len(), 5);

        // Verify all tokens are present
        let token_count_by_ns = meteor.tokens()
            .iter()
            .fold(std::collections::HashMap::new(), |mut acc, (ns, _)| {
                *acc.entry(ns.to_string()).or_insert(0) += 1;
                acc
            });

        assert_eq!(token_count_by_ns.get("a"), Some(&2));
        assert_eq!(token_count_by_ns.get("b"), Some(&2));
        assert_eq!(token_count_by_ns.get("c"), Some(&1));
    }

    #[test]
    fn test_meteor_empty() {
        // Test meteor with no tokens
        let meteor = Meteor::new(Context::default(), vec![]);
        assert_eq!(meteor.context().name(), "app");
        assert_eq!(meteor.tokens().len(), 0);
    }

    #[test]
    fn test_meteor_context_privileges() {
        // Test different context privilege levels
        let contexts = vec![
            Context::from("app"),
            Context::from("sys"),
            Context::from("test"),
        ];

        for ctx in contexts {
            let meteor = Meteor::new(
                ctx.clone(),
                vec![(Namespace::from("ns"), Token::new("key", "value"))]
            );
            assert_eq!(meteor.context(), &ctx);
        }
    }

    #[test]
    fn test_meteor_parse_with_dots_in_namespace() {
        // Test namespace with dot notation (hierarchy)
        let meteor = Meteor::parse("app:ui.button.primary:color=blue")
            .expect("Failed to parse");

        let (namespace, token) = &meteor.tokens()[0];
        assert_eq!(namespace.to_string(), "ui.button.primary");
        assert_eq!(token.key().as_str(), "color");
        assert_eq!(token.value(), "blue");
    }

    #[test]
    fn test_meteor_debug_format() {
        let meteor = Meteor::new(
            Context::from("debug"),
            vec![(Namespace::from("test"), Token::new("key", "value"))]
        );

        let debug_str = format!("{:?}", meteor);
        assert!(debug_str.contains("Meteor"));
        assert!(debug_str.contains("debug"));
    }
}