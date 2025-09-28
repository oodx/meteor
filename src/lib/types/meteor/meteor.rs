//! Meteor type - the complete token addressing structure

use crate::types::{Context, MeteorError, Namespace, Token};
use std::fmt;
use std::str::FromStr;

/// Complete Meteor token with full addressing: context:namespace:key=value
///
/// This is the primary type that holds the complete token structure according to
/// the TOKEN_NAMESPACE_CONCEPT specification. As of ENG-40, meteors enforce the
/// single (context, namespace) invariant to ensure data consistency.
///
/// Format: `context:namespace:key=value`
/// Example: `app:ui.widgets:button[0]=submit`
///
/// # Constructor Safety (ENG-40)
///
/// Starting with ENG-40, meteor constructors enforce strict validation to prevent
/// meteors from containing tokens with conflicting namespaces. This ensures the
/// single (context, namespace) invariant is maintained.
///
/// ## Recommended Constructors (ENG-40+)
///
/// - [`Meteor::try_new()`] - Safe constructor that returns `Result<Meteor, MeteorError>`
/// - [`Meteor::try_new_with_tokens()`] - Safe multi-token constructor
/// - [`Meteor::try_with_default_context()`] - Safe constructor with default context
///
/// ```rust
/// use meteor::types::{Context, Namespace, Token, Meteor};
///
/// // ✅ Recommended: Safe constructor with error handling
/// let meteor = Meteor::try_new(
///     Context::user(),
///     Namespace::from_string("settings"),
///     Token::new("theme", "dark")
/// )?;
/// # Ok::<(), meteor::types::MeteorError>(())
/// ```
///
/// ## Legacy Constructors (Backward Compatible)
///
/// The original constructors are preserved for backward compatibility but may panic
/// on validation errors:
///
/// - [`Meteor::new()`] - Panics on validation failure
/// - [`Meteor::new_with_tokens()`] - Panics on validation failure
/// - [`Meteor::with_default_context()`] - Panics on validation failure
///
/// ```rust
/// use meteor::types::{Context, Namespace, Token, Meteor};
///
/// // ⚠️ Legacy: Panics on validation errors
/// let meteor = Meteor::new(
///     Context::user(),
///     Namespace::from_string("settings"),
///     Token::new("theme", "dark")
/// );
/// ```
///
/// ## Validation Rules (ENG-40)
///
/// 1. **Single Namespace Invariant**: All tokens in a meteor must belong to the same namespace
/// 2. **Explicit Namespace Matching**: Tokens with explicit namespaces must match the meteor's namespace
/// 3. **Implicit Namespace Inheritance**: Tokens without namespaces inherit the meteor's namespace
/// 4. **Empty Token Prevention**: Meteors cannot be created with empty token lists
///
/// ```rust
/// use meteor::types::{Context, Namespace, Token, Meteor};
///
/// let namespace = Namespace::from_string("ui");
/// let tokens = vec![
///     Token::new("button", "click"),                    // ✅ Inherits "ui"
///     Token::new_with_namespace(namespace.clone(), "theme", "dark"), // ✅ Matches "ui"
///     // Token::new_with_namespace(Namespace::from_string("other"), "font", "Arial"), // ❌ Would fail
/// ];
///
/// let meteor = Meteor::try_new_with_tokens(
///     Context::app(),
///     namespace,
///     tokens
/// )?;
/// # Ok::<(), meteor::types::MeteorError>(())
/// ```
///
/// ## Migration Guide (ENG-40)
///
/// ### For New Code
/// Always use the `try_*` constructors for better error handling:
///
/// ```rust
/// use meteor::types::{Context, Namespace, Token, Meteor, MeteorError};
///
/// fn create_user_settings() -> Result<Meteor, MeteorError> {
///     Meteor::try_new_with_tokens(
///         Context::user(),
///         Namespace::from_string("settings"),
///         vec![
///             Token::new("theme", "dark"),
///             Token::new("lang", "en"),
///         ]
///     )
/// }
/// ```
///
/// ### For Existing Code
/// Existing code using `new()` and `new_with_tokens()` continues to work unchanged
/// for valid meteors. Only meteors that violate the namespace invariant will now panic
/// instead of creating inconsistent data.
///
/// ```rust
/// use meteor::types::{Context, Namespace, Token, Meteor};
///
/// // This continues to work exactly as before
/// let meteor = Meteor::new_with_tokens(
///     Context::app(),
///     Namespace::from_string("ui"),
///     vec![
///         Token::new("button", "click"),
///         Token::new("theme", "dark"),
///     ]
/// );
/// ```
///
/// ### For Edge Cases
/// If you encounter validation errors with existing data, temporary migration
/// helpers are available (but discouraged):
///
/// ```rust,ignore
/// use meteor::types::{Context, Namespace, Token, Meteor};
///
/// // ⚠️ Only for migration - bypasses validation
/// #[allow(deprecated)]
/// let meteor = Meteor::force_create_unchecked(context, namespace, tokens);
/// ```
///
/// ## Error Handling
///
/// The new constructors provide detailed error information:
///
/// ```rust
/// use meteor::types::{Context, Namespace, Token, Meteor, MeteorError};
///
/// let result = Meteor::try_new_with_tokens(
///     Context::app(),
///     Namespace::from_string("ui"),
///     vec![] // Empty tokens
/// );
///
/// match result {
///     Err(MeteorError::EmptyTokens) => {
///         println!("Cannot create meteor with empty token list");
///     }
///     Err(MeteorError::TokenNamespaceMismatch { meteor_namespace, token_namespace, token_key }) => {
///         println!("Token '{}' has namespace '{}' but meteor requires '{}'",
///                  token_key, token_namespace, meteor_namespace);
///     }
///     Ok(meteor) => {
///         // Process valid meteor
///     }
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Meteor {
    context: Context,
    namespace: Namespace,
    tokens: Vec<Token>,
}

impl Meteor {
    /// Create a new Meteor with all components (ENG-40: Hardened constructor)
    ///
    /// This constructor enforces the single (context, namespace) invariant:
    /// - All tokens must belong to the same conceptual namespace
    /// - Tokens with explicit namespaces must match the meteor's namespace
    /// - Tokens without namespaces are assumed to belong to the meteor's namespace
    ///
    /// # Panics
    /// Panics if tokens violate the single namespace invariant.
    /// Use `try_new()` for non-panicking validation.
    pub fn new(context: Context, namespace: Namespace, token: Token) -> Self {
        Self::try_new(context, namespace, token)
            .expect("Token namespace must match meteor namespace")
    }

    /// Create a new Meteor with multiple tokens (ENG-40: Hardened constructor)
    ///
    /// This constructor enforces the single (context, namespace) invariant:
    /// - All tokens must belong to the same conceptual namespace
    /// - Tokens with explicit namespaces must match the meteor's namespace
    /// - Tokens without namespaces are assumed to belong to the meteor's namespace
    ///
    /// # Panics
    /// Panics if tokens violate the single namespace invariant.
    /// Use `try_new_with_tokens()` for non-panicking validation.
    pub fn new_with_tokens(context: Context, namespace: Namespace, tokens: Vec<Token>) -> Self {
        Self::try_new_with_tokens(context, namespace, tokens)
            .expect("All tokens must belong to the same namespace as the meteor")
    }

    /// Create with default context (app) (ENG-40: Hardened constructor)
    ///
    /// This constructor enforces the single (context, namespace) invariant.
    ///
    /// # Panics
    /// Panics if token violates the namespace invariant.
    /// Use `try_with_default_context()` for non-panicking validation.
    pub fn with_default_context(namespace: Namespace, token: Token) -> Self {
        Self::try_with_default_context(namespace, token)
            .expect("Token namespace must match meteor namespace")
    }

    /// Try to create a new Meteor with validation (ENG-40: Safe constructor)
    ///
    /// Returns `Err` if the token's namespace doesn't match the meteor's namespace.
    /// This is the preferred constructor for new code.
    pub fn try_new(context: Context, namespace: Namespace, token: Token) -> Result<Self, MeteorError> {
        Self::try_new_with_tokens(context, namespace, vec![token])
    }

    /// Try to create a new Meteor with multiple tokens (ENG-40: Safe constructor)
    ///
    /// Returns `Err` if any token violates the single namespace invariant.
    /// This is the preferred constructor for new code.
    pub fn try_new_with_tokens(
        context: Context,
        namespace: Namespace,
        tokens: Vec<Token>,
    ) -> Result<Self, MeteorError> {
        if tokens.is_empty() {
            return Err(MeteorError::EmptyTokens);
        }

        Self::validate_tokens_strict(&namespace, &tokens)?;
        Ok(Meteor {
            context,
            namespace,
            tokens,
        })
    }

    /// Try to create with default context (ENG-40: Safe constructor)
    ///
    /// Returns `Err` if the token's namespace doesn't match the meteor's namespace.
    pub fn try_with_default_context(namespace: Namespace, token: Token) -> Result<Self, MeteorError> {
        Self::try_new(Context::default(), namespace, token)
    }

    // ================================
    // Legacy Compatibility Shims (ENG-40)
    // ================================

    /// Create meteor from parts with legacy validation (ENG-40: Legacy support)
    ///
    /// This method provides backward compatibility for existing code that might
    /// create meteors with mixed validation patterns. New code should use
    /// `try_new_with_tokens()` instead.
    ///
    /// # Migration Guide
    ///
    /// **Before (legacy):**
    /// ```rust,ignore
    /// let meteor = Meteor::from_parts_legacy(context, namespace, tokens)
    ///     .expect("validation failed");
    /// ```
    ///
    /// **After (recommended):**
    /// ```rust,ignore
    /// let meteor = Meteor::try_new_with_tokens(context, namespace, tokens)?;
    /// ```
    #[deprecated(
        since = "0.2.0",
        note = "Use try_new_with_tokens() for better error handling and strict validation"
    )]
    pub fn from_parts_legacy(
        context: Context,
        namespace: Namespace,
        tokens: Vec<Token>,
    ) -> Result<Self, String> {
        Self::from_parts(context, namespace, tokens)
    }

    /// Create meteor with lenient validation (ENG-40: Legacy support)
    ///
    /// This method uses the original validation logic for backward compatibility.
    /// New code should use `try_new_with_tokens()` for stricter validation.
    ///
    /// # Warning
    /// This method may accept meteors that violate the single namespace invariant
    /// in edge cases. Use with caution.
    #[deprecated(
        since = "0.2.0",
        note = "Use try_new_with_tokens() for hardened constructor validation"
    )]
    pub fn new_with_lenient_validation(
        context: Context,
        namespace: Namespace,
        tokens: Vec<Token>,
    ) -> Result<Self, String> {
        if tokens.is_empty() {
            return Err("Cannot create meteor with empty token list".to_string());
        }
        Self::validate_tokens(&namespace, &tokens)?;
        Ok(Meteor {
            context,
            namespace,
            tokens,
        })
    }

    /// Force create meteor without validation (ENG-40: Emergency escape hatch)
    ///
    /// **WARNING**: This method bypasses all validation and can create meteors
    /// that violate the single namespace invariant. Only use this as a last resort
    /// for migration scenarios where strict validation blocks existing functionality.
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - Tokens belong to the same conceptual namespace
    /// - The meteor will not be used in contexts that expect validated meteors
    ///
    /// # Migration Path
    /// This method should only be used temporarily during migration to ENG-40.
    /// Fix the underlying data consistency issues and migrate to `try_new_with_tokens()`.
    #[deprecated(
        since = "0.2.0",
        note = "Bypasses validation - fix data consistency and use try_new_with_tokens()"
    )]
    pub fn force_create_unchecked(
        context: Context,
        namespace: Namespace,
        tokens: Vec<Token>,
    ) -> Self {
        Meteor {
            context,
            namespace,
            tokens,
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
                Self::from_parts(Context::default(), Namespace::default(), tokens)
            }
            1 => {
                // Format: namespace:token(s)
                let parts: Vec<&str> = s.splitn(2, ':').collect();

                // Check if second part contains '='
                if parts[1].contains('=') {
                    // Assume first part is namespace (no context specified)
                    let namespace = Namespace::from_string(parts[0]);
                    let tokens = Self::parse_tokens(parts[1])?;
                    Self::from_parts(Context::default(), namespace, tokens)
                } else {
                    Err(format!("Invalid meteor format: {}", s))
                }
            }
            2 => {
                // Full format: context:namespace:token(s)
                let parts: Vec<&str> = s.splitn(3, ':').collect();

                let context = Context::from_str(parts[0])?;
                let namespace = Namespace::from_string(parts[1]);
                let tokens = Self::parse_tokens(parts[2])?;

                Self::from_parts(context, namespace, tokens)
            }
            _ => Err(format!("Too many colons in meteor format: {}", s)),
        }
    }

    /// Parse semicolon-separated tokens
    fn parse_tokens(tokens_str: &str) -> Result<Vec<Token>, String> {
        let parts = crate::parser::split::smart_split_semicolons(tokens_str)
            .ok_or_else(|| "Unbalanced quotes in token string".to_string())?;
        let mut tokens = Vec::new();

        for token_str in parts {
            let trimmed = token_str.trim();
            if trimmed.is_empty() {
                continue;
            }
            let token = Token::first(trimmed)?;
            tokens.push(token);
        }

        if tokens.is_empty() {
            return Err("No valid tokens found".to_string());
        }

        Ok(tokens)
    }

    fn from_parts(
        context: Context,
        namespace: Namespace,
        tokens: Vec<Token>,
    ) -> Result<Self, String> {
        Self::validate_tokens(&namespace, &tokens)?;
        Ok(Meteor {
            context,
            namespace,
            tokens,
        })
    }

    fn validate_tokens(namespace: &Namespace, tokens: &[Token]) -> Result<(), String> {
        for token in tokens {
            if let Some(token_namespace) = token.namespace() {
                if token_namespace != namespace {
                    return Err(format!(
                        "Token namespace '{}' does not match meteor namespace '{}'",
                        token_namespace, namespace
                    ));
                }
            }
        }

        Ok(())
    }

    /// Strict validation for ENG-40 hardened constructors
    ///
    /// Enforces the single (context, namespace) invariant with detailed error reporting:
    /// - All tokens with explicit namespaces must match the meteor's namespace
    /// - Tokens without namespaces are assumed to belong to the meteor's namespace
    /// - Provides detailed error messages for debugging
    fn validate_tokens_strict(namespace: &Namespace, tokens: &[Token]) -> Result<(), MeteorError> {
        let mut conflicting_namespaces = Vec::new();
        let mut first_mismatch: Option<(String, String)> = None; // (namespace, key)

        // Collect all conflicting namespaces in one pass
        for token in tokens {
            if let Some(token_namespace) = token.namespace() {
                if token_namespace != namespace {
                    let ns_string = token_namespace.to_string();
                    if !conflicting_namespaces.contains(&ns_string) {
                        conflicting_namespaces.push(ns_string);
                    }

                    // Remember the first mismatch for detailed error reporting
                    if first_mismatch.is_none() {
                        first_mismatch = Some((token_namespace.to_string(), token.key_notation().to_string()));
                    }
                }
            }
        }

        // No conflicts found
        if conflicting_namespaces.is_empty() {
            return Ok(());
        }

        // Single namespace conflict - provide detailed error with specific token
        if conflicting_namespaces.len() == 1 {
            let (token_namespace, token_key) = first_mismatch.unwrap();
            return Err(MeteorError::token_namespace_mismatch(
                namespace.to_string(),
                token_namespace,
                token_key,
            ));
        }

        // Multiple namespace conflicts - provide overview of all conflicts
        Err(MeteorError::mixed_token_namespaces(
            namespace.to_string(),
            conflicting_namespaces,
        ))
    }
}

impl fmt::Display for Meteor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tokens_str = self
            .tokens
            .iter()
            .map(|token| format!("{}={}", token.key_notation(), token.value()))
            .collect::<Vec<_>>()
            .join(";");

        write!(
            f,
            "{}:{}:{}",
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
    fn test_meteor_parse_mismatched_namespace() {
        let result = Meteor::parse("app:ui.widgets:button=submit;profile:user=name");
        assert!(result.is_err());
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
            Token::new("theme", "dark"),
        );
        assert_eq!(meteor.to_string(), "user:settings:theme=dark");
    }

    #[test]
    fn test_meteor_parse_with_quoted_semicolon() {
        let meteors = Meteor::parse("app:ui.widgets:message=\"Hello; World\"").unwrap();
        assert_eq!(meteors.len(), 1);
        let meteor = &meteors[0];
        assert_eq!(meteor.namespace().to_string(), "ui.widgets");
        assert_eq!(meteor.tokens().len(), 1);
        assert_eq!(meteor.tokens()[0].value(), "\"Hello; World\"");
    }

    #[test]
    fn test_meteor_parse_unbalanced_quotes() {
        let result = Meteor::parse("app:ui.widgets:message=\"Hello; World");
        assert!(result.is_err());
    }
}
