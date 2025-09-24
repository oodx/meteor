//! Build-time configuration via Cargo features
//!
//! Provides different limit profiles for various deployment scenarios:
//! - Default: Balanced limits for general use
//! - Enterprise: Higher limits for large-scale deployments
//! - Embedded: Lower limits for memory-constrained environments
//! - Strict: Minimal limits for high-security environments

/// Maximum meteors per MeteorShower collection
#[cfg(feature = "enterprise")]
pub const MAX_METEORS_PER_SHOWER: usize = 10_000;   // Enterprise: large collections
#[cfg(feature = "embedded")]
pub const MAX_METEORS_PER_SHOWER: usize = 100;      // Embedded: memory limited
#[cfg(feature = "strict")]
pub const MAX_METEORS_PER_SHOWER: usize = 50;       // Strict: minimal collections
#[cfg(not(any(feature = "enterprise", feature = "embedded", feature = "strict")))]
pub const MAX_METEORS_PER_SHOWER: usize = 1_000;    // Default: balanced

/// Maximum command history entries in MeteorEngine
#[cfg(feature = "enterprise")]
pub const MAX_COMMAND_HISTORY: usize = 10_000;      // Enterprise: extensive audit trail
#[cfg(feature = "embedded")]
pub const MAX_COMMAND_HISTORY: usize = 100;         // Embedded: minimal history
#[cfg(feature = "strict")]
pub const MAX_COMMAND_HISTORY: usize = 500;         // Strict: moderate history
#[cfg(not(any(feature = "enterprise", feature = "embedded", feature = "strict")))]
pub const MAX_COMMAND_HISTORY: usize = 1_000;       // Default: balanced

/// Maximum contexts in StorageData
#[cfg(feature = "enterprise")]
pub const MAX_CONTEXTS: usize = 1_000;              // Enterprise: many contexts
#[cfg(feature = "embedded")]
pub const MAX_CONTEXTS: usize = 10;                 // Embedded: few contexts
#[cfg(feature = "strict")]
pub const MAX_CONTEXTS: usize = 5;                  // Strict: minimal contexts
#[cfg(not(any(feature = "enterprise", feature = "embedded", feature = "strict")))]
pub const MAX_CONTEXTS: usize = 100;                // Default: balanced

/// Maximum key length for tokens
#[cfg(feature = "enterprise")]
pub const MAX_TOKEN_KEY_LENGTH: usize = 256;        // Enterprise: long descriptive keys
#[cfg(feature = "embedded")]
pub const MAX_TOKEN_KEY_LENGTH: usize = 32;         // Embedded: short keys
#[cfg(feature = "strict")]
pub const MAX_TOKEN_KEY_LENGTH: usize = 16;         // Strict: minimal keys
#[cfg(not(any(feature = "enterprise", feature = "embedded", feature = "strict")))]
pub const MAX_TOKEN_KEY_LENGTH: usize = 128;        // Default: balanced

/// Maximum value length for tokens
#[cfg(feature = "enterprise")]
pub const MAX_TOKEN_VALUE_LENGTH: usize = 8_192;    // Enterprise: large data values
#[cfg(feature = "embedded")]
pub const MAX_TOKEN_VALUE_LENGTH: usize = 256;      // Embedded: small values
#[cfg(feature = "strict")]
pub const MAX_TOKEN_VALUE_LENGTH: usize = 128;      // Strict: minimal values
#[cfg(not(any(feature = "enterprise", feature = "embedded", feature = "strict")))]
pub const MAX_TOKEN_VALUE_LENGTH: usize = 2_048;    // Default: balanced

/// Display current configuration profile
pub fn config_profile() -> &'static str {
    #[cfg(feature = "enterprise")]
    { "enterprise" }
    #[cfg(feature = "embedded")]
    { "embedded" }
    #[cfg(feature = "strict")]
    { "strict" }
    #[cfg(not(any(feature = "enterprise", feature = "embedded", feature = "strict")))]
    { "default" }
}

/// Display current configuration limits
pub fn config_summary() -> String {
    format!("Meteor Configuration Profile: {}\n\
             - Max namespace part length: {}\n\
             - Namespace warning depth: {}\n\
             - Namespace error depth: {}\n\
             - Max meteors per shower: {}\n\
             - Max command history: {}\n\
             - Max contexts: {}\n\
             - Max token key length: {}\n\
             - Max token value length: {}",
        config_profile(),
        crate::types::MAX_NAMESPACE_PART_LENGTH,
        crate::types::NAMESPACE_WARNING_DEPTH,
        crate::types::NAMESPACE_ERROR_DEPTH,
        MAX_METEORS_PER_SHOWER,
        MAX_COMMAND_HISTORY,
        MAX_CONTEXTS,
        MAX_TOKEN_KEY_LENGTH,
        MAX_TOKEN_VALUE_LENGTH
    )
}