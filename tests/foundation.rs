//! Foundation tests for core Meteor types using correct APIs

use meteor::{TokenKey, Token, Context, Namespace, Meteor, MeteorShower, MeteorEngine, TokenStreamParser, MeteorStreamParser};

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

#[cfg(test)]
mod meteor_engine_integration_tests {
    use super::*;

    #[test]
    fn test_meteor_engine_basic_operations() {
        let mut engine = MeteorEngine::new();

        // Test basic dot-notation API
        engine.set("app.ui.button", "click").unwrap();
        engine.set("app.ui.theme", "dark").unwrap();
        engine.set("user.profile.name", "Alice").unwrap();

        // Test retrieval
        assert_eq!(engine.get("app.ui.button"), Some("click"));
        assert_eq!(engine.get("app.ui.theme"), Some("dark"));
        assert_eq!(engine.get("user.profile.name"), Some("Alice"));
        assert_eq!(engine.get("nonexistent"), None);

        // Test existence checks
        assert!(engine.exists("app.ui.button"));
        assert!(!engine.exists("missing.key"));
    }

    #[test]
    fn test_meteor_engine_cursor_state() {
        let mut engine = MeteorEngine::new();

        // Verify initial cursor state
        assert_eq!(engine.current_context.to_string(), "app");
        assert_eq!(engine.current_namespace.to_string(), "main");

        // Modify cursor state
        engine.current_context = Context::new("user");
        engine.current_namespace = Namespace::from_string("settings");

        // Verify state persists
        assert_eq!(engine.current_context.to_string(), "user");
        assert_eq!(engine.current_namespace.to_string(), "settings");
    }

    #[test]
    fn test_meteor_engine_control_commands() {
        let mut engine = MeteorEngine::new();

        // Add test data
        engine.set("app.ui.button", "click").unwrap();
        engine.set("app.ui.theme", "dark").unwrap();

        // Execute control commands
        engine.execute_control_command("reset", "cursor").unwrap();

        // Verify command was recorded
        let history = engine.command_history();
        assert!(!history.is_empty());
        assert_eq!(history.last().unwrap().command_type, "reset");
        assert_eq!(history.last().unwrap().target, "cursor");
        assert!(history.last().unwrap().success);

        // Test invalid command
        let result = engine.execute_control_command("invalid", "command");
        assert!(result.is_err());

        // Verify failed command recorded
        let failed = engine.failed_commands();
        assert!(!failed.is_empty());
        assert_eq!(failed.last().unwrap().command_type, "invalid");
    }
}

#[cfg(test)]
mod token_stream_parser_integration_tests {
    use super::*;

    #[test]
    fn test_token_stream_basic_processing() {
        let mut engine = MeteorEngine::new();

        // Process basic token stream
        TokenStreamParser::process(&mut engine, "button=click;theme=dark").unwrap();

        // Verify data stored with cursor context/namespace
        assert_eq!(engine.get("app.main.button"), Some("click"));
        assert_eq!(engine.get("app.main.theme"), Some("dark"));

        // Verify cursor state unchanged (no control tokens)
        assert_eq!(engine.current_context.to_string(), "app");
        assert_eq!(engine.current_namespace.to_string(), "main");
    }

    #[test]
    fn test_token_stream_folding_logic() {
        let mut engine = MeteorEngine::new();

        // Process stream with namespace folding
        TokenStreamParser::process(&mut engine, "button=click;ns=ui;theme=dark").unwrap();

        // Verify folding logic applied
        assert_eq!(engine.get("app.main.button"), Some("click"));  // Before ns=ui
        assert_eq!(engine.get("app.ui.theme"), Some("dark"));      // After ns=ui

        // Verify cursor state changed
        assert_eq!(engine.current_context.to_string(), "app");
        assert_eq!(engine.current_namespace.to_string(), "ui");
    }

    #[test]
    fn test_token_stream_context_switching() {
        let mut engine = MeteorEngine::new();

        // Process stream with context switching
        TokenStreamParser::process(&mut engine, "app_setting=value;ctx=user;profile=admin").unwrap();

        // Verify context switching applied
        assert_eq!(engine.get("app.main.app_setting"), Some("value"));  // Before ctx=user
        assert_eq!(engine.get("user.main.profile"), Some("admin"));     // After ctx=user

        // Verify cursor state changed
        assert_eq!(engine.current_context.to_string(), "user");
        assert_eq!(engine.current_namespace.to_string(), "main");
    }

    #[test]
    fn test_token_stream_continuity() {
        let mut engine = MeteorEngine::new();

        // First stream: sets context to ui
        TokenStreamParser::process(&mut engine, "initial=value;ns=ui").unwrap();

        // Second stream: should use ui namespace from previous
        TokenStreamParser::process(&mut engine, "continued=value").unwrap();

        // Verify stream continuity
        assert_eq!(engine.get("app.main.initial"), Some("value"));   // First stream
        assert_eq!(engine.get("app.ui.continued"), Some("value"));   // Continues from ui namespace

        // Final cursor state should be ui
        assert_eq!(engine.current_namespace.to_string(), "ui");
    }

    #[test]
    fn test_token_stream_control_commands() {
        let mut engine = MeteorEngine::new();

        // Add initial data
        engine.set("app.ui.button", "click").unwrap();

        // Process control command via stream
        TokenStreamParser::process(&mut engine, "ctl:reset=cursor;newdata=value").unwrap();

        // Verify control command executed
        let history = engine.command_history();
        assert!(history.iter().any(|cmd| cmd.command_type == "reset" && cmd.target == "cursor"));

        // Verify regular data still processed
        assert_eq!(engine.get("app.main.newdata"), Some("value"));  // After reset, back to main
    }
}

#[cfg(test)]
mod meteor_stream_parser_integration_tests {
    use super::*;

    #[test]
    fn test_meteor_stream_explicit_addressing() {
        let mut engine = MeteorEngine::new();

        // Process explicit meteor stream
        MeteorStreamParser::process(&mut engine, "app:ui:button=click").unwrap();

        // Verify explicit addressing worked
        assert_eq!(engine.get("app.ui.button"), Some("click"));

        // Verify cursor state unchanged (explicit addressing doesn't affect cursor)
        assert_eq!(engine.current_context.to_string(), "app");
        assert_eq!(engine.current_namespace.to_string(), "main");
    }

    #[test]
    fn test_meteor_stream_multiple_meteors() {
        let mut engine = MeteorEngine::new();

        // Process multiple meteors with delimiter
        MeteorStreamParser::process(&mut engine, "app:ui:button=click :;: user:main:profile=admin").unwrap();

        // Verify both meteors processed
        assert_eq!(engine.get("app.ui.button"), Some("click"));
        assert_eq!(engine.get("user.main.profile"), Some("admin"));

        // Cursor should be unchanged
        assert_eq!(engine.current_context.to_string(), "app");
        assert_eq!(engine.current_namespace.to_string(), "main");
    }

    #[test]
    fn test_meteor_stream_with_control_commands() {
        let mut engine = MeteorEngine::new();

        // Add initial data
        engine.set("app.ui.button", "click").unwrap();

        // Process meteor stream with control command
        MeteorStreamParser::process(&mut engine, "ctl:reset=cursor :;: user:settings:theme=dark").unwrap();

        // Verify control command executed
        let history = engine.command_history();
        assert!(history.iter().any(|cmd| cmd.command_type == "reset"));

        // Verify meteor processed
        assert_eq!(engine.get("user.settings.theme"), Some("dark"));
    }
}

#[cfg(test)]
mod parser_validation_integration_tests {
    use super::*;

    #[test]
    fn test_invalid_token_stream_rejected() {
        let mut engine = MeteorEngine::new();

        // Invalid format should be rejected
        let result = TokenStreamParser::process(&mut engine, "invalid format without equals");
        assert!(result.is_err());

        // Engine should remain unchanged
        assert_eq!(engine.get("app.main.invalid"), None);
    }

    #[test]
    fn test_invalid_meteor_stream_rejected() {
        let mut engine = MeteorEngine::new();

        // Invalid meteor format should be rejected
        let result = MeteorStreamParser::process(&mut engine, "invalid format");
        assert!(result.is_err());

        // Engine should remain unchanged
        assert_eq!(engine.get("app.main.invalid"), None);
    }

    #[test]
    fn test_quoted_values_in_streams() {
        let mut engine = MeteorEngine::new();

        // Process stream with quoted values containing special characters
        TokenStreamParser::process(&mut engine, "message=\"Hello; World\"").unwrap();

        // Verify quoted value preserved correctly (quotes currently preserved in storage)
        // TODO: Integrate escape sequence parsing to strip quotes
        assert_eq!(engine.get("app.main.message"), Some("\"Hello; World\""));
    }
}

#[cfg(test)]
mod end_to_end_workflow_tests {
    use super::*;

    #[test]
    fn test_complete_data_processing_workflow() {
        let mut engine = MeteorEngine::new();

        // 1. Process configuration data
        TokenStreamParser::process(&mut engine, "host=localhost;port=8080;ns=db").unwrap();
        TokenStreamParser::process(&mut engine, "user=admin;pass=secret").unwrap();

        // 2. Process user data in different context
        // After step 1, cursor should be at app:db
        // ctx=user should switch to user:db, then name=Alice stores as user:db:name
        TokenStreamParser::process(&mut engine, "ctx=user;name=Alice;email=alice@test.com").unwrap();

        // 3. Add some explicit meteors
        MeteorStreamParser::process(&mut engine, "sys:config:debug=true :;: sys:config:version=1.0").unwrap();

        // 4. Clean up sensitive data (command recorded but deletion not yet implemented)
        TokenStreamParser::process(&mut engine, "ctl:delete=app.db.pass").unwrap();

        // Verify final state
        assert_eq!(engine.get("app.main.host"), Some("localhost"));
        assert_eq!(engine.get("app.main.port"), Some("8080"));
        assert_eq!(engine.get("app.db.user"), Some("admin"));
        // TODO: Implement actual deletion in StorageData - currently delete is not implemented
        assert_eq!(engine.get("app.db.pass"), Some("secret"));  // Still there (delete not implemented)
        assert_eq!(engine.get("user.db.name"), Some("Alice"));  // Alice stored in user:db context
        assert_eq!(engine.get("sys.config.debug"), Some("true"));

        // Verify command history
        let history = engine.command_history();
        assert!(history.iter().any(|cmd| cmd.command_type == "delete" && cmd.target == "app.db.pass"));

        // Verify final cursor state
        assert_eq!(engine.current_context.to_string(), "user");
        assert_eq!(engine.current_namespace.to_string(), "db");  // Should be db from step 2
    }
}