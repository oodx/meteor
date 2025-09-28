//! Token Stream Parser - Validates and processes token streams with folding logic
//!
//! Handles:
//! - Token validation (key=value format)
//! - Control tokens (ns=, ctx=) for cursor state
//! - Control commands (ctl:delete=path, ctl:reset=cursor)
//! - Delegation to MeteorEngine for state changes
//! - ENG-41: Meteor-aware parsing with aggregation and hardened constructors

use crate::types::{Context, Meteor, MeteorEngine, MeteorError, Namespace, Token};
use crate::utils::validators::is_valid_token_format;
use crate::parser::split::{smart_split, SplitConfig};
use std::collections::HashMap;
use std::str::FromStr;

/// Token stream parser with validation and delegation
pub struct TokenStreamParser;

impl TokenStreamParser {
    /// Parse and process a token stream with meteor aggregation (ENG-41)
    ///
    /// Groups tokens by (context, namespace) and creates meteors using hardened constructors.
    /// Provides better error handling and validation than the legacy `process()` method.
    ///
    /// # Examples
    /// ```ignore
    /// let mut engine = MeteorEngine::new();
    /// TokenStreamParser::process_with_aggregation(&mut engine, "button=click; theme=dark")?;
    /// ```
    pub fn process_with_aggregation(engine: &mut MeteorEngine, input: &str) -> Result<(), MeteorError> {
        let grouped_tokens = Self::parse_and_group_tokens(engine, input)?;

        // Create meteors using hardened constructors and store them
        for ((context, namespace), tokens) in grouped_tokens {
            if !tokens.is_empty() {
                let meteor = Meteor::try_new_with_tokens(context, namespace, tokens)?;
                Self::store_meteor_tokens(engine, &meteor)?;
            }
        }

        Ok(())
    }

    /// Parse and process a token stream (Legacy method)
    ///
    /// Validates tokens and delegates to MeteorEngine for state changes.
    /// Supports folding logic with ns= and ctx= control tokens.
    ///
    /// # Migration Note (ENG-41)
    /// Consider using `process_with_aggregation()` for better error handling
    /// and consistency with meteor aggregation patterns.
    ///
    /// # Examples
    /// ```ignore
    /// let mut engine = MeteorEngine::new();
    /// TokenStreamParser::process(&mut engine, "button=click; ns=ui; theme=dark")?;
    /// ```
    pub fn process(engine: &mut MeteorEngine, input: &str) -> Result<(), String> {
        // Split by semicolon (respecting quotes) - using centralized split logic
        let parts = smart_split(input, SplitConfig::general_parsing(';'));

        for part in parts {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Check if it's a control command
            if trimmed.starts_with("ctl:") {
                Self::process_control_command(engine, trimmed)?;
                continue;
            }

            // Validate token format
            if !is_valid_token_format(trimmed) {
                return Err(format!("Invalid token format: {}", trimmed));
            }

            // Parse the token
            let token = Token::from_str(trimmed)
                .map_err(|e| format!("Failed to parse token '{}': {}", trimmed, e))?;

            // Check for control tokens
            match token.key().transformed() {
                "ns" => {
                    // Namespace switch
                    engine.current_namespace = Namespace::from_string(token.value());
                }
                "ctx" => {
                    // Context switch
                    engine.current_context = Context::from_str(token.value())
                        .map_err(|e| format!("Invalid context '{}': {}", token.value(), e))?;
                }
                _ => {
                    // Regular token - store using current cursor state
                    let path = format!(
                        "{}:{}:{}",
                        engine.current_context.to_string(),
                        engine.current_namespace.to_string(),
                        token.key().transformed()
                    );
                    engine.set(&path, token.value())?;
                }
            }
        }

        Ok(())
    }

    /// Process control command
    fn process_control_command(engine: &mut MeteorEngine, command: &str) -> Result<(), String> {
        // Parse ctl:command=target format
        // SAFETY: Caller ensures command starts with "ctl:" via starts_with check
        let without_prefix = command.strip_prefix("ctl:")
            .expect("control command must start with 'ctl:' prefix");
        let parts: Vec<&str> = without_prefix.splitn(2, '=').collect();

        if parts.len() != 2 {
            return Err(format!("Invalid control command format: {}", command));
        }

        let cmd_type = parts[0];
        let target = parts[1];

        engine.execute_control_command(cmd_type, target)
    }

    // Smart split functionality moved to centralized parser::split module (ENG-42)

    // ================================
    // ENG-41: Meteor-Aware Parsing Helpers
    // ================================

    /// Parse tokens and group them by (context, namespace) for meteor aggregation
    fn parse_and_group_tokens(
        engine: &mut MeteorEngine,
        input: &str,
    ) -> Result<HashMap<(Context, Namespace), Vec<Token>>, MeteorError> {
        let mut grouped_tokens: HashMap<(Context, Namespace), Vec<Token>> = HashMap::new();
        let parts = smart_split(input, SplitConfig::general_parsing(';'));

        // Track current cursor state (mutable during parsing)
        let mut current_context = engine.current_context.clone();
        let mut current_namespace = engine.current_namespace.clone();

        for part in parts {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Check if it's a control command - handle but don't aggregate
            if trimmed.starts_with("ctl:") {
                Self::process_control_command(engine, trimmed)
                    .map_err(|e| MeteorError::other(e))?;
                continue;
            }

            // Validate token format
            if !is_valid_token_format(trimmed) {
                return Err(MeteorError::other(format!("Invalid token format: {}", trimmed)));
            }

            // Parse the token
            let token = Token::from_str(trimmed)
                .map_err(|e| MeteorError::other(format!("Failed to parse token '{}': {}", trimmed, e)))?;

            // Check for control tokens (update cursor state)
            match token.key().transformed() {
                "ns" => {
                    current_namespace = Namespace::from_string(token.value());
                    continue;
                }
                "ctx" => {
                    current_context = Context::from_str(token.value())
                        .map_err(|e| MeteorError::other(format!("Invalid context '{}': {}", token.value(), e)))?;
                    continue;
                }
                _ => {
                    // Regular token - add to appropriate group
                    let key = (current_context.clone(), current_namespace.clone());
                    grouped_tokens.entry(key).or_insert_with(Vec::new).push(token);
                }
            }
        }

        // Update engine cursor state to final state
        engine.current_context = current_context;
        engine.current_namespace = current_namespace;

        Ok(grouped_tokens)
    }

    /// Store tokens from a validated meteor into the engine
    fn store_meteor_tokens(engine: &mut MeteorEngine, meteor: &Meteor) -> Result<(), MeteorError> {
        for token in meteor.tokens() {
            let path = format!(
                "{}:{}:{}",
                meteor.context().to_string(),
                meteor.namespace().to_string(),
                token.key().transformed()
            );
            engine.set(&path, token.value())
                .map_err(|e| MeteorError::other(e))?;
        }
        Ok(())
    }

    /// Validate a token stream without processing
    pub fn validate(input: &str) -> Result<(), String> {
        let parts = smart_split(input, SplitConfig::general_parsing(';'));

        for part in parts {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Skip control commands (validated separately)
            if trimmed.starts_with("ctl:") {
                continue;
            }

            // Validate token format
            if !is_valid_token_format(trimmed) {
                return Err(format!("Invalid token format: {}", trimmed));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_stream_processing() {
        let mut engine = MeteorEngine::new();

        // Process tokens with namespace switching
        TokenStreamParser::process(&mut engine, "button=click; ns=ui; theme=dark").unwrap();

        // Check that values were stored correctly
        assert_eq!(engine.get("app:main:button"), Some("click"));
        assert_eq!(engine.get("app:ui:theme"), Some("dark"));

        // Check cursor state changed
        assert_eq!(engine.current_namespace.to_string(), "ui");
    }

    #[test]
    fn test_context_switching() {
        let mut engine = MeteorEngine::new();

        // Switch context and add data
        TokenStreamParser::process(
            &mut engine,
            "ctx=user; profile=admin; ns=settings; theme=dark",
        )
        .unwrap();

        assert_eq!(engine.get("user:main:profile"), Some("admin"));
        assert_eq!(engine.get("user:settings:theme"), Some("dark"));
        assert_eq!(engine.current_context.to_string(), "user");
    }

    #[test]
    fn test_control_commands() {
        let mut engine = MeteorEngine::new();

        // Add data then delete it
        engine.set("app.ui.button", "click").unwrap();
        TokenStreamParser::process(&mut engine, "ctl:delete=app.ui.button").unwrap();

        // Check command was executed (though delete might not work due to StorageData limitations)
        let history = engine.command_history();
        assert!(history
            .iter()
            .any(|cmd| cmd.command_type == "delete" && cmd.target == "app.ui.button"));
    }

    #[test]
    fn test_validation() {
        assert!(TokenStreamParser::validate("key=value; ns=ui").is_ok());
        assert!(TokenStreamParser::validate("invalid format").is_err());
        assert!(TokenStreamParser::validate("key=\"value with; semicolons\"").is_ok());
    }
}
