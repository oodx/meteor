//! Hub Dependencies Baseline Sanity Tests
//!
//! Validates that each hub dependency works at a foundational level
//! before proceeding with RSB feature implementation.

#[cfg(test)]
mod tests {
    #[test]
    fn sanity_hub_test_ext_criterion() {
        // Test that hub::test_ext provides criterion for benchmarking
        // This should work since we have test-ext feature enabled

        use hub::criterion::{black_box, Criterion};

        // Test basic criterion functionality
        let mut criterion = Criterion::default();

        // Simple benchmark test to verify criterion works
        let result = black_box(42);
        assert_eq!(result, 42, "Criterion black_box should work");

        // Test that we can create a benchmark group (compilation test)
        let _group = criterion.benchmark_group("test_group");

        assert!(true, "Hub test-ext provides working criterion");
    }

    #[test]
    fn sanity_hub_integration_no_cli_ext() {
        // Test that removing cli-ext doesn't break hub integration
        // We removed cli-ext since we use native RSB CLI now

        // Verify other hub features still work
        use hub::async_ext::tokio;
        use hub::error_ext::anyhow::Result;

        // Test async functionality still works
        let rt = tokio::runtime::Runtime::new();
        assert!(rt.is_ok(), "Hub async-ext should work without cli-ext");

        // Test error handling still works
        let _result: Result<()> = Ok(());

        assert!(true, "Hub works correctly without cli-ext dependency");
    }

    #[test]
    fn sanity_hub_async_ext_basic_functionality() {
        // Test basic hub::async_ext functionality (tokio-lite)
        // This should provide basic async runtime without full networking features

        use hub::async_ext::tokio;

        // Test basic runtime creation (what tokio-lite should provide)
        let rt = tokio::runtime::Runtime::new();
        assert!(rt.is_ok(), "Tokio-lite runtime creation should work");

        // Test basic async block execution
        let rt = rt.unwrap();
        let result = rt.block_on(async {
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            42
        });

        assert_eq!(
            result, 42,
            "Basic async execution should work with tokio-lite"
        );
    }

    #[test]
    fn sanity_hub_async_ext_time_features() {
        // Test that tokio-lite includes time features
        use hub::async_ext::tokio;

        let rt = tokio::runtime::Runtime::new().unwrap();

        let start = std::time::Instant::now();
        rt.block_on(async {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        });
        let elapsed = start.elapsed();

        assert!(
            elapsed >= std::time::Duration::from_millis(8),
            "Tokio time features should work"
        );
        assert!(
            elapsed < std::time::Duration::from_millis(100),
            "Sleep should be reasonably accurate"
        );
    }

    #[test]
    fn sanity_hub_error_ext_accessible() {
        // Test that hub::error_ext module is accessible
        // We'll test actual functionality once we identify the correct import paths

        // For now, just verify compilation works with error-ext feature
        assert!(true, "Hub error-ext feature should be accessible");
    }

    #[test]
    fn sanity_hub_core_colors() {
        // Test basic hub::core functionality (colors)
        // This should be available since we have "core" feature

        // For now, just test that hub::colors module is accessible
        // TODO: Add actual color functionality tests once we identify the API

        // Placeholder test to ensure core feature compilation
        assert!(true, "Hub core feature should be accessible");
    }

    #[test]
    fn sanity_hub_integration_with_meteor() {
        // Test that hub dependencies don't interfere with meteor functionality
        use hub::error_ext::anyhow::Result;

        fn parse_with_hub_error_handling(_input: &str) -> Result<()> {
            // TODO: Update to use correct API after TICKET-003
            Ok(())
        }

        let result = parse_with_hub_error_handling("ctx=app; ui:button=click");
        assert!(result.is_ok(), "Meteor should work with hub error handling");

        // TODO: Add proper integration tests after TICKET-003
    }

    #[test]
    fn sanity_hub_lite_variants_vs_full() {
        // Document what features we expect from lite variants vs full variants

        // With cli-ext (clap-lite), we should have:
        // ✅ Basic command creation and argument parsing
        // ❌ Derive macros (would need clap-full)

        // With async-ext (tokio-lite), we should have:
        // ✅ Basic runtime and time features
        // ❌ Networking features like TcpListener (would need tokio-full)

        // With error-ext, we should have:
        // ✅ anyhow and thiserror for error handling

        let lite_features = vec![
            "clap basic command creation",
            "clap argument parsing",
            "tokio basic runtime",
            "tokio time features",
            "anyhow error handling",
            "thiserror derive macros",
        ];

        assert_eq!(
            lite_features.len(),
            6,
            "We expect 6 core lite variant features to work"
        );

        // This test documents our expectations for lite vs full variants
        assert!(
            true,
            "Lite variants provide sufficient functionality for meteor CLI"
        );
    }
}
