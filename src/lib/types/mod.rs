//! Core type definitions for Meteor
//!
//! Organized by type hierarchy:
//! - Basic types (Context, Namespace, Token) form the foundation
//! - Composite type (Meteor) combines all components
//! - Storage types (MeteorShower) manage collections
//! - Support types (MeteorError) provide infrastructure

mod context;
mod error;
mod key;
mod meteor;
mod namespace;
mod token;

// Re-export all public types
pub use context::Context;
pub use error::MeteorError;
pub use key::{
    extract_base_name, has_brackets, reverse_transform_key, transform_key, BracketNotation,
    TokenKey,
};
pub use meteor::{
    ControlCommand, Meteor, MeteorEngine, MeteorShower, StorageData, METEOR_DELIMITER,
};
pub use namespace::{
    Namespace, MAX_NAMESPACE_PART_LENGTH, NAMESPACE_ERROR_DEPTH, NAMESPACE_WARNING_DEPTH,
};
pub use token::{Token, TokenBucket};
