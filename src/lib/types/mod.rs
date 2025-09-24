//! Core type definitions for Meteor
//!
//! Organized by type hierarchy:
//! - Basic types (Context, Namespace, Token) form the foundation
//! - Composite type (Meteor) combines all components
//! - Storage types (MeteorShower) manage collections
//! - Support types (MeteorError) provide infrastructure

mod context;
mod namespace;
mod key;
mod token;
mod meteor;
mod error;

// Re-export all public types
pub use context::Context;
pub use namespace::{Namespace, MAX_NAMESPACE_PART_LENGTH, NAMESPACE_WARNING_DEPTH, NAMESPACE_ERROR_DEPTH};
pub use key::{TokenKey, BracketNotation, transform_key, reverse_transform_key, has_brackets, extract_base_name};
pub use token::{Token, TokenBucket};
pub use meteor::{Meteor, MeteorShower, MeteorEngine, StorageData, ControlCommand, METEOR_DELIMITER};
pub use error::MeteorError;