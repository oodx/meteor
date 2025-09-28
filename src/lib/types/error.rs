//! Error types for Meteor
//!
//! MeteorError provides detailed error information for parsing and validation failures.

use std::fmt;

/// Main error type for Meteor operations
#[derive(Debug, Clone, PartialEq)]
pub enum MeteorError {
    /// Parse error with position and description
    ParseError { position: usize, message: String },

    /// Invalid token format
    InvalidToken { token: String, reason: String },

    /// Context violation (attempting cross-context access)
    ContextViolation {
        from_context: String,
        to_context: String,
        message: String,
    },

    /// Namespace depth exceeded limits
    NamespaceTooDeep {
        namespace: String,
        depth: usize,
        max_depth: usize,
    },

    /// Invalid bracket notation
    InvalidBracketNotation { key: String, reason: String },

    /// Invalid character in key or value
    InvalidCharacter {
        found: char,
        position: usize,
        context: String,
    },

    /// Empty input or component
    EmptyInput { component: String },

    /// Empty token list when creating meteor (ENG-40)
    EmptyTokens,

    /// Token namespace mismatch in meteor (ENG-40)
    TokenNamespaceMismatch {
        meteor_namespace: String,
        token_namespace: String,
        token_key: String,
    },

    /// Mixed namespaces in meteor tokens (ENG-40)
    MixedTokenNamespaces {
        meteor_namespace: String,
        conflicting_namespaces: Vec<String>,
    },

    /// Generic error for other cases
    Other(String),
}

impl MeteorError {
    /// Create a parse error
    pub fn parse(position: usize, message: impl Into<String>) -> Self {
        MeteorError::ParseError {
            position,
            message: message.into(),
        }
    }

    /// Create an invalid token error
    pub fn invalid_token(token: impl Into<String>, reason: impl Into<String>) -> Self {
        MeteorError::InvalidToken {
            token: token.into(),
            reason: reason.into(),
        }
    }

    /// Create a context violation error
    pub fn context_violation(
        from: impl Into<String>,
        to: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        MeteorError::ContextViolation {
            from_context: from.into(),
            to_context: to.into(),
            message: message.into(),
        }
    }

    /// Create a namespace depth error
    pub fn namespace_too_deep(namespace: impl Into<String>, depth: usize) -> Self {
        MeteorError::NamespaceTooDeep {
            namespace: namespace.into(),
            depth,
            max_depth: 3,
        }
    }

    /// Create a bracket notation error
    pub fn invalid_bracket(key: impl Into<String>, reason: impl Into<String>) -> Self {
        MeteorError::InvalidBracketNotation {
            key: key.into(),
            reason: reason.into(),
        }
    }

    /// Create an invalid character error
    pub fn invalid_char(found: char, position: usize, context: impl Into<String>) -> Self {
        MeteorError::InvalidCharacter {
            found,
            position,
            context: context.into(),
        }
    }

    /// Create an empty input error
    pub fn empty(component: impl Into<String>) -> Self {
        MeteorError::EmptyInput {
            component: component.into(),
        }
    }

    /// Create a generic error
    pub fn other(message: impl Into<String>) -> Self {
        MeteorError::Other(message.into())
    }

    /// Create an empty tokens error (ENG-40)
    pub fn empty_tokens() -> Self {
        MeteorError::EmptyTokens
    }

    /// Create a token namespace mismatch error (ENG-40)
    pub fn token_namespace_mismatch(
        meteor_namespace: impl Into<String>,
        token_namespace: impl Into<String>,
        token_key: impl Into<String>,
    ) -> Self {
        MeteorError::TokenNamespaceMismatch {
            meteor_namespace: meteor_namespace.into(),
            token_namespace: token_namespace.into(),
            token_key: token_key.into(),
        }
    }

    /// Create a mixed token namespaces error (ENG-40)
    pub fn mixed_token_namespaces(
        meteor_namespace: impl Into<String>,
        conflicting_namespaces: Vec<String>,
    ) -> Self {
        MeteorError::MixedTokenNamespaces {
            meteor_namespace: meteor_namespace.into(),
            conflicting_namespaces,
        }
    }
}

impl fmt::Display for MeteorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MeteorError::ParseError { position, message } => {
                write!(f, "Parse error at position {}: {}", position, message)
            }
            MeteorError::InvalidToken { token, reason } => {
                write!(f, "Invalid token '{}': {}", token, reason)
            }
            MeteorError::ContextViolation {
                from_context,
                to_context,
                message,
            } => {
                write!(
                    f,
                    "Context violation from '{}' to '{}': {}",
                    from_context, to_context, message
                )
            }
            MeteorError::NamespaceTooDeep {
                namespace,
                depth,
                max_depth,
            } => {
                write!(
                    f,
                    "Namespace '{}' too deep ({} levels, max {})",
                    namespace, depth, max_depth
                )
            }
            MeteorError::InvalidBracketNotation { key, reason } => {
                write!(f, "Invalid bracket notation in '{}': {}", key, reason)
            }
            MeteorError::InvalidCharacter {
                found,
                position,
                context,
            } => {
                write!(
                    f,
                    "Invalid character '{}' at position {} in {}",
                    found, position, context
                )
            }
            MeteorError::EmptyInput { component } => {
                write!(f, "Empty input for {}", component)
            }
            MeteorError::EmptyTokens => {
                write!(f, "Cannot create meteor with empty token list")
            }
            MeteorError::TokenNamespaceMismatch {
                meteor_namespace,
                token_namespace,
                token_key,
            } => {
                write!(
                    f,
                    "Token '{}' has namespace '{}' but meteor requires namespace '{}'",
                    token_key, token_namespace, meteor_namespace
                )
            }
            MeteorError::MixedTokenNamespaces {
                meteor_namespace,
                conflicting_namespaces,
            } => {
                write!(
                    f,
                    "Meteor with namespace '{}' cannot contain tokens from namespaces: {}",
                    meteor_namespace,
                    conflicting_namespaces.join(", ")
                )
            }
            MeteorError::Other(message) => write!(f, "{}", message),
        }
    }
}

impl std::error::Error for MeteorError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = MeteorError::parse(10, "unexpected character");
        assert_eq!(
            err.to_string(),
            "Parse error at position 10: unexpected character"
        );

        let err = MeteorError::namespace_too_deep("ui.widgets.buttons.primary.icon", 5);
        assert!(err.to_string().contains("too deep"));

        let err = MeteorError::invalid_bracket("list[", "unclosed bracket");
        assert!(err.to_string().contains("Invalid bracket"));
    }
}
