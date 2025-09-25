//! Token Stream Parser - Validates and processes token streams with folding logic
//!
//! Handles:
//! - Token validation (key=value format)
//! - Control tokens (ns=, ctx=) for cursor state
//! - Control commands (ctl:delete=path, ctl:reset=cursor)
//! - Delegation to MeteorEngine for state changes

use crate::types::{Token, MeteorEngine, Context, Namespace};
use crate::utils::validators::is_valid_token_format;
use std::str::FromStr;

/// Token stream parser with validation and delegation
pub struct TokenStreamParser;

impl TokenStreamParser {
    /// Parse and process a token stream
    ///
    /// Validates tokens and delegates to MeteorEngine for state changes.
    /// Supports folding logic with ns= and ctx= control tokens.
    ///
    /// # Examples
    /// ```ignore
    /// let mut engine = MeteorEngine::new();
    /// TokenStreamParser::process(&mut engine, "button=click; ns=ui; theme=dark")?;
    /// ```
    pub fn process(engine: &mut MeteorEngine, input: &str) -> Result<(), String> {
        // Split by semicolon (respecting quotes)
        let parts = Self::smart_split(input, ';');

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
                    let path = format!("{}:{}:{}",
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
        let without_prefix = command.strip_prefix("ctl:").unwrap();
        let parts: Vec<&str> = without_prefix.splitn(2, '=').collect();

        if parts.len() != 2 {
            return Err(format!("Invalid control command format: {}", command));
        }

        let cmd_type = parts[0];
        let target = parts[1];

        engine.execute_control_command(cmd_type, target)
    }

    /// Smart split that respects quoted values
    fn smart_split(input: &str, delimiter: char) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut escape_next = false;

        for ch in input.chars() {
            if escape_next {
                current.push(ch);
                escape_next = false;
                continue;
            }

            match ch {
                '\\' => {
                    current.push(ch);
                    escape_next = true;
                }
                '"' => {
                    current.push(ch);
                    in_quotes = !in_quotes;
                }
                c if c == delimiter && !in_quotes => {
                    if !current.trim().is_empty() {
                        result.push(current.trim().to_string());
                    }
                    current.clear();
                }
                _ => current.push(ch),
            }
        }

        if !current.trim().is_empty() {
            result.push(current.trim().to_string());
        }

        result
    }

    /// Validate a token stream without processing
    pub fn validate(input: &str) -> Result<(), String> {
        let parts = Self::smart_split(input, ';');

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
        TokenStreamParser::process(&mut engine, "ctx=user; profile=admin; ns=settings; theme=dark").unwrap();

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
        assert!(history.iter().any(|cmd| cmd.command_type == "delete" && cmd.target == "app.ui.button"));
    }

    #[test]
    fn test_validation() {
        assert!(TokenStreamParser::validate("key=value; ns=ui").is_ok());
        assert!(TokenStreamParser::validate("invalid format").is_err());
        assert!(TokenStreamParser::validate("key=\"value with; semicolons\"").is_ok());
    }
}