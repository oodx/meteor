//! Meteor-specific configuration constants
//!
//! Configuration for MeteorEngine, MeteorShower, and related components.
//! These values are read from meteor.toml at build time.

/// Maximum namespace part length
pub const MAX_NAMESPACE_PART_LENGTH: usize = {
    if cfg!(meteor_enterprise) {
        128 // Enterprise: longer names allowed
    } else if cfg!(meteor_embedded) {
        32 // Embedded: memory constrained
    } else if cfg!(meteor_strict) {
        16 // Strict: minimal names
    } else {
        64 // Default: balanced
    }
};

/// Namespace depth warning threshold
/// Above this depth, warnings are issued but processing continues
pub const NAMESPACE_WARNING_DEPTH: usize = {
    if cfg!(meteor_enterprise) {
        6 // Enterprise: deeper hierarchies (5 clear, 6 warning)
    } else if cfg!(meteor_embedded) {
        3 // Embedded: shallow hierarchies (2 clear, 3 warning)
    } else if cfg!(meteor_strict) {
        3 // Strict: minimal depth (2 clear, 3 warning)
    } else {
        5 // Default: balanced (4 clear, 5 warning)
    }
};

/// Namespace depth error threshold
/// Above this depth, processing fails with error
pub const NAMESPACE_ERROR_DEPTH: usize = {
    if cfg!(meteor_enterprise) {
        8 // Enterprise: deep hierarchies allowed (6+ error at 8+)
    } else if cfg!(meteor_embedded) {
        4 // Embedded: strict depth limit (3+ error at 4+)
    } else if cfg!(meteor_strict) {
        4 // Strict: strict depth limit (3+ error at 4+)
    } else {
        6 // Default: balanced (5+ error at 6+)
    }
};
