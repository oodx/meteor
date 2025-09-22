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
    token: Token,
}

impl Meteor {
    /// Create a new Meteor with all components
    pub fn new(context: Context, namespace: Namespace, token: Token) -> Self {
        Meteor {
            context,
            namespace,
            token,
        }
    }

    /// Create with default context (app)
    pub fn with_default_context(namespace: Namespace, token: Token) -> Self {
        Meteor {
            context: Context::default(),
            namespace,
            token,
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

    /// Get the token
    pub fn token(&self) -> &Token {
        &self.token
    }

    /// Parse from full format: "context:namespace:key=value"
    pub fn parse(s: &str) -> Result<Self, String> {
        // Count colons to determine format
        let colon_count = s.chars().filter(|&c| c == ':').count();

        match colon_count {
            0 => {
                // No context or namespace, just key=value
                let token = Token::parse(s)?;
                Ok(Meteor::new(Context::default(), Namespace::default(), token))
            }
            1 => {
                // Either context:key=value or namespace:key=value
                let parts: Vec<&str> = s.splitn(2, ':').collect();

                // Check if second part contains '='
                if parts[1].contains('=') {
                    // Assume first part is namespace (no context specified)
                    let namespace = Namespace::from_string(parts[0]);
                    let token = Token::parse(parts[1])?;
                    Ok(Meteor::new(Context::default(), namespace, token))
                } else {
                    return Err(format!("Invalid meteor format: {}", s));
                }
            }
            2 => {
                // Full format: context:namespace:key=value
                let parts: Vec<&str> = s.splitn(3, ':').collect();

                let context = Context::from_str(parts[0])?;
                let namespace = Namespace::from_string(parts[1]);
                let token = Token::parse(parts[2])?;

                Ok(Meteor::new(context, namespace, token))
            }
            _ => {
                Err(format!("Too many colons in meteor format: {}", s))
            }
        }
    }

    /// Get the full address string
    pub fn to_address(&self) -> String {
        format!("{}:{}:{}={}",
            self.context,
            self.namespace,
            self.token.transformed_key(),
            self.token.value()
        )
    }
}

impl fmt::Display for Meteor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_address())
    }
}

impl FromStr for Meteor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Meteor::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meteor_parse_full() {
        let meteor = Meteor::parse("app:ui.widgets:button=submit").unwrap();
        assert_eq!(meteor.context().name(), "app");
        assert_eq!(meteor.namespace().to_string(), "ui.widgets");
        assert_eq!(meteor.token().key(), "button");
        assert_eq!(meteor.token().value(), "submit");
    }

    #[test]
    fn test_meteor_parse_no_context() {
        let meteor = Meteor::parse("ui.widgets:button=submit").unwrap();
        assert_eq!(meteor.context().name(), "app"); // Default
        assert_eq!(meteor.namespace().to_string(), "ui.widgets");
    }

    #[test]
    fn test_meteor_parse_minimal() {
        let meteor = Meteor::parse("button=submit").unwrap();
        assert_eq!(meteor.context().name(), "app"); // Default
        assert_eq!(meteor.namespace().to_string(), ""); // Root
        assert_eq!(meteor.token().key(), "button");
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