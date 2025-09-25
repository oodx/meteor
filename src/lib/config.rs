//! Build-time configuration via meteor.toml
//!
//! Provides different limit profiles for various deployment scenarios:
//! - Default: Balanced limits for general use
//! - Enterprise: Higher limits for large-scale deployments
//! - Embedded: Lower limits for memory-constrained environments
//! - Strict: Minimal limits for high-security environments
//!
//! Configuration is read from meteor.toml at compile time via build.rs

/// Maximum meteors per MeteorShower collection
pub const MAX_METEORS_PER_SHOWER: usize = {
    if cfg!(meteor_enterprise) {
        10000 // Enterprise: large collections
    } else if cfg!(meteor_embedded) {
        100 // Embedded: memory limited
    } else if cfg!(meteor_strict) {
        50 // Strict: minimal collections
    } else {
        1000 // Default: balanced
    }
};

/// Maximum command history entries in MeteorEngine
pub const MAX_COMMAND_HISTORY: usize = {
    if cfg!(meteor_enterprise) {
        10000 // Enterprise: extensive audit trail
    } else if cfg!(meteor_embedded) {
        100 // Embedded: minimal history
    } else if cfg!(meteor_strict) {
        500 // Strict: moderate history
    } else {
        1000 // Default: balanced
    }
};

/// Maximum contexts in StorageData
pub const MAX_CONTEXTS: usize = {
    if cfg!(meteor_enterprise) {
        1000 // Enterprise: many contexts
    } else if cfg!(meteor_embedded) {
        10 // Embedded: few contexts
    } else if cfg!(meteor_strict) {
        5 // Strict: minimal contexts
    } else {
        100 // Default: balanced
    }
};

/// Maximum key length for tokens
pub const MAX_TOKEN_KEY_LENGTH: usize = {
    if cfg!(meteor_enterprise) {
        256 // Enterprise: long descriptive keys
    } else if cfg!(meteor_embedded) {
        32 // Embedded: short keys
    } else if cfg!(meteor_strict) {
        16 // Strict: minimal keys
    } else {
        128 // Default: balanced
    }
};

/// Maximum value length for tokens
pub const MAX_TOKEN_VALUE_LENGTH: usize = {
    if cfg!(meteor_enterprise) {
        8192 // Enterprise: large data values
    } else if cfg!(meteor_embedded) {
        256 // Embedded: small values
    } else if cfg!(meteor_strict) {
        128 // Strict: minimal values
    } else {
        2048 // Default: balanced
    }
};

/// Display current configuration profile
pub fn config_profile() -> &'static str {
    if cfg!(meteor_enterprise) {
        "enterprise"
    } else if cfg!(meteor_embedded) {
        "embedded"
    } else if cfg!(meteor_strict) {
        "strict"
    } else {
        "default"
    }
}

/// Check if runtime tampering prevention is enabled
pub fn prevent_runtime_tampering() -> bool {
    option_env!("METEOR_SECURITY_PREVENT_RUNTIME_TAMPERING")
        .map(|v| v == "true")
        .unwrap_or(true) // Default to secure
}

/// Check if namespace character validation is enabled
pub fn validate_namespace_characters() -> bool {
    option_env!("METEOR_SECURITY_VALIDATE_NAMESPACE_CHARACTERS")
        .map(|v| v == "true")
        .unwrap_or(true) // Default to secure
}

/// Check if reserved word enforcement is enabled
pub fn enforce_reserved_words() -> bool {
    option_env!("METEOR_SECURITY_ENFORCE_RESERVED_WORDS")
        .map(|v| v == "true")
        .unwrap_or(true) // Default to secure
}

/// Check if command audit trail is enabled
pub fn enable_command_audit_trail() -> bool {
    option_env!("METEOR_SECURITY_ENABLE_COMMAND_AUDIT_TRAIL")
        .map(|v| v == "true")
        .unwrap_or(true) // Default enabled
}

/// Display current configuration limits
pub fn config_summary() -> String {
    format!(
        "Meteor Configuration Profile: {} (from meteor.toml)\n\
             - Max namespace part length: {}\n\
             - Namespace warning depth: {}\n\
             - Namespace error depth: {}\n\
             - Max meteors per shower: {}\n\
             - Max command history: {}\n\
             - Max contexts: {}\n\
             - Max token key length: {}\n\
             - Max token value length: {}\n\
             - Runtime tampering prevention: {}\n\
             - Namespace character validation: {}\n\
             - Reserved word enforcement: {}\n\
             - Command audit trail: {}",
        config_profile(),
        crate::types::MAX_NAMESPACE_PART_LENGTH,
        crate::types::NAMESPACE_WARNING_DEPTH,
        crate::types::NAMESPACE_ERROR_DEPTH,
        MAX_METEORS_PER_SHOWER,
        MAX_COMMAND_HISTORY,
        MAX_CONTEXTS,
        MAX_TOKEN_KEY_LENGTH,
        MAX_TOKEN_VALUE_LENGTH,
        prevent_runtime_tampering(),
        validate_namespace_characters(),
        enforce_reserved_words(),
        enable_command_audit_trail()
    )
}
