//! Core type definitions for Meteor
//!
//! Organized by ordinality hierarchy:
//! - Primary types (Context, Namespace, Key) form the foundation
//! - Secondary types (TokenBucket) depend on primary types
//! - Support types (MeteorError) provide infrastructure

mod primary;
mod bucket;
mod error;

// Re-export all public types
pub use primary::{Context, Namespace, Key};
pub use bucket::TokenBucket;
pub use error::MeteorError;