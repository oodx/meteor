//! Meteor type - the complete token addressing structure

use crate::types::{Context, Namespace, Token};
use std::fmt;
use std::str::FromStr;

/// Complete Meteor token with full addressing: context:namespace:key=value
///
/// This is the primary type that holds the complete token structure according to
/// the TOKEN_NAMESPACE_CONCEPT specification.
///
/// Format: `context:namespace:key=value`
/// Example: `app:ui.widgets:button[0]=submit`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Meteor {
    context: Context,
    namespace: Namespace,
    tokens: Vec<Token>,
}

impl Meteor {
    /// Create a new Meteor with all components
    pub fn new(context: Context, namespace: Namespace, token: Token) -> Self {
        Meteor {
            context,
            namespace,
            tokens: vec![token],
        }
    }

    /// Create a new Meteor with multiple tokens
    pub fn new_with_tokens(context: Context, namespace: Namespace, tokens: Vec<Token>) -> Self {
        Meteor {
            context,
            namespace,
            tokens,
        }
    }

    /// Create with default context (app)
    pub fn with_default_context(namespace: Namespace, token: Token) -> Self {
        Meteor {
            context: Context::default(),
            namespace,
            tokens: vec![token],
        }
    }

    /// Get the context
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Get the namespace
    pub fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    /// Get the first token (for backward compatibility)
    pub fn token(&self) -> &Token {
        &self.tokens[0]
    }

    /// Get all tokens
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    /// Parse from full format: "context:namespace:key=value;key2=value2"
    /// Returns Vec<Meteor> to support multiple meteor specifications
    pub fn parse(s: &str) -> Result<Vec<Self>, String> {
        let meteor = Self::parse_single(s)?;
        Ok(vec![meteor])
    }

    /// Parse the first meteor from a string (convenience method)
    pub fn first(s: &str) -> Result<Self, String> {
        let meteors = Self::parse(s)?;
        Ok(meteors.into_iter().next().unwrap()) // Safe because parse() ensures non-empty vec
    }

    /// Parse a single meteor from a string (internal method)
    fn parse_single(s: &str) -> Result<Self, String> {
        // Count colons to determine format
        let colon_count = s.chars().filter(|&c| c == ':').count();

        match colon_count {
            0 => {
                // No context or namespace, just token(s) - use default namespace
                let tokens = Self::parse_tokens(s)?;
                Ok(Meteor::new_with_tokens(Context::default(), Namespace::default(), tokens))
            }
            1 => {
                // Format: namespace:token(s)
                let parts: Vec<&str> = s.splitn(2, ':').collect();

                // Check if second part contains '='
                if parts[1].contains('=') {
                    // Assume first part is namespace (no context specified)
                    let namespace = Namespace::from_string(parts[0]);
                    let tokens = Self::parse_tokens(parts[1])?;
                    Ok(Meteor::new_with_tokens(Context::default(), namespace, tokens))
                } else {
                    return Err(format!("Invalid meteor format: {}", s));
                }
            }
            2 => {
                // Full format: context:namespace:token(s)
                let parts: Vec<&str> = s.splitn(3, ':').collect();

                let context = Context::from_str(parts[0])?;
                let namespace = Namespace::from_string(parts[1]);
                let tokens = Self::parse_tokens(parts[2])?;

                Ok(Meteor::new_with_tokens(context, namespace, tokens))
            }
            _ => {
                Err(format!("Too many colons in meteor format: {}", s))
            }
        }
    }

    /// Parse semicolon-separated tokens
    fn parse_tokens(tokens_str: &str) -> Result<Vec<Token>, String> {
        let token_parts = tokens_str.split(';').map(|s| s.trim()).filter(|s| !s.is_empty());
        let mut tokens = Vec::new();

        for token_str in token_parts {
            let token = Token::first(token_str)?;
            tokens.push(token);
        }

        if tokens.is_empty() {
            return Err("No valid tokens found".to_string());
        }

        Ok(tokens)
    }

}

impl fmt::Display for Meteor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tokens_str = self.tokens.iter()
            .map(|token| format!("{}={}", token.key().to_string(), token.value()))
            .collect::<Vec<_>>()
            .join(";");

        write!(f, "{}:{}:{}",
            self.context.to_string(),
            self.namespace.to_string(),
            tokens_str
        )
    }
}

impl FromStr for Meteor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Meteor::first(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meteor_parse_full() {
        let meteors = Meteor::parse("app:ui.widgets:button=submit").unwrap();
        assert_eq!(meteors.len(), 1);
        let meteor = &meteors[0];
        assert_eq!(meteor.context().name(), "app");
        assert_eq!(meteor.namespace().to_string(), "ui.widgets");
        assert_eq!(meteor.token().key_notation(), "button");
        assert_eq!(meteor.token().value(), "submit");
    }

    #[test]
    fn test_meteor_parse_no_context() {
        let meteors = Meteor::parse("ui.widgets:button=submit").unwrap();
        assert_eq!(meteors.len(), 1);
        let meteor = &meteors[0];
        assert_eq!(meteor.context().name(), "app"); // Default
        assert_eq!(meteor.namespace().to_string(), "ui.widgets");
    }

    #[test]
    fn test_meteor_parse_minimal() {
        let meteors = Meteor::parse("button=submit").unwrap();
        assert_eq!(meteors.len(), 1);
        let meteor = &meteors[0];
        assert_eq!(meteor.context().name(), "app"); // Default
        assert_eq!(meteor.namespace().to_string(), "main"); // Default namespace
        assert_eq!(meteor.token().key_notation(), "button");
    }

    #[test]
    fn test_meteor_first() {
        let meteor = Meteor::first("app:ui.widgets:button=submit").unwrap();
        assert_eq!(meteor.context().name(), "app");
        assert_eq!(meteor.namespace().to_string(), "ui.widgets");
        assert_eq!(meteor.token().key_notation(), "button");
        assert_eq!(meteor.token().value(), "submit");

        // Test first from single meteor
        let meteor = Meteor::first("button=submit").unwrap();
        assert_eq!(meteor.context().name(), "app"); // Default
        assert_eq!(meteor.namespace().to_string(), "main"); // Default
        assert_eq!(meteor.token().key_notation(), "button");
    }

    #[test]
    fn test_meteor_display() {
        let meteor = Meteor::new(
            Context::user(),
            Namespace::from_string("settings"),
            Token::new("theme", "dark")
        );
        assert_eq!(meteor.to_string(), "user:settings:theme=dark");
    }
}