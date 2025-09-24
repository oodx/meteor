//! Foundation tests for core Meteor types using correct APIs

use meteor::{TokenKey, Token, Context, Namespace, Meteor, MeteorShower};

#[cfg(test)]
mod token_key_tests {
    use super::*;

    #[test]
    fn test_token_key_creation() {
        let key = TokenKey::new("simple_key");
        assert_eq!(key.base(), "simple_key");
        assert_eq!(key.transformed(), "simple_key");
        assert!(!key.has_brackets());
    }

    #[test]
    fn test_token_key_bracket_notation() {
        let key = TokenKey::new("list[0]");
        assert_eq!(key.base(), "list[0]");
        assert_eq!(key.transformed(), "list__i_0");
        assert!(key.has_brackets());
    }

    #[test]
    fn test_token_key_multi_dimensional() {
        let key = TokenKey::new("matrix[2,3]");
        assert_eq!(key.base(), "matrix[2,3]");
        assert_eq!(key.transformed(), "matrix__i_2_3");
        assert!(key.has_brackets());
    }

    #[test]
    fn test_token_key_equality() {
        let key1 = TokenKey::new("test");
        let key2 = TokenKey::new("test");
        assert_eq!(key1, key2);
    }
}

#[cfg(test)]
mod token_tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let token = Token::new("key", "value");
        assert_eq!(token.key().base(), "key");
        assert_eq!(token.value(), "value");
        assert_eq!(token.key_notation(), "key");
        assert_eq!(token.key_str(), "key");
    }

    #[test]
    fn test_token_with_brackets() {
        let token = Token::new("list[0]", "first_item");
        assert_eq!(token.key().base(), "list[0]");
        assert_eq!(token.key().transformed(), "list__i_0");
        assert_eq!(token.value(), "first_item");
        assert_eq!(token.key_notation(), "list[0]");
        assert_eq!(token.key_str(), "list__i_0");
    }

    #[test]
    fn test_token_equality() {
        let token1 = Token::new("key", "value");
        let token2 = Token::new("key", "value");
        assert_eq!(token1, token2);
    }
}

#[cfg(test)]
mod meteor_tests {
    use super::*;

    #[test]
    fn test_meteor_creation() {
        let meteor = Meteor::new(
            Context::new("app"),
            Namespace::from_string("ui"),
            Token::new("button", "click")
        );

        assert_eq!(meteor.context().name(), "app");
        assert_eq!(meteor.namespace().to_string(), "ui");
        assert_eq!(meteor.token().value(), "click");
    }

    #[test]
    fn test_meteor_with_default_context() {
        let meteor = Meteor::with_default_context(
            Namespace::from_string("config"),
            Token::new("setting", "value")
        );

        assert_eq!(meteor.context().name(), "app");
        assert_eq!(meteor.namespace().to_string(), "config");
    }

    #[test]
    fn test_meteor_with_brackets() {
        let meteor = Meteor::new(
            Context::new("app"),
            Namespace::from_string("data"),
            Token::new("list[0]", "first")
        );

        assert_eq!(meteor.token().key_notation(), "list[0]");
        assert_eq!(meteor.token().key_str(), "list__i_0");
        assert_eq!(meteor.token().value(), "first");
    }
}

#[cfg(test)]
mod meteor_shower_tests {
    use super::*;

    #[test]
    fn test_meteor_shower_creation() {
        let shower = MeteorShower::new();
        assert_eq!(shower.len(), 0);
        assert!(shower.contexts().is_empty());
    }

    #[test]
    fn test_meteor_shower_add_meteor() {
        let mut shower = MeteorShower::new();
        let meteor = Meteor::new(
            Context::new("app"),
            Namespace::from_string("ui"),
            Token::new("button", "click")
        );

        shower.add(meteor);
        assert_eq!(shower.len(), 1);
        assert_eq!(shower.contexts().len(), 1);
    }

    #[test]
    fn test_meteor_shower_multiple_contexts() {
        let mut shower = MeteorShower::new();

        shower.add(Meteor::new(
            Context::new("app"),
            Namespace::from_string("ui"),
            Token::new("theme", "dark")
        ));

        shower.add(Meteor::new(
            Context::new("sys"),
            Namespace::from_string("config"),
            Token::new("debug", "true")
        ));

        assert_eq!(shower.len(), 2);
        assert_eq!(shower.contexts().len(), 2);
    }

    #[test]
    fn test_meteor_shower_clone() {
        let mut shower = MeteorShower::new();
        shower.add(Meteor::new(
            Context::new("test"),
            Namespace::from_string("data"),
            Token::new("key", "value")
        ));

        let cloned = shower.clone();
        assert_eq!(shower.len(), cloned.len());
        assert_eq!(shower.contexts(), cloned.contexts());
    }
}