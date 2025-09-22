//! Core type definitions for Meteor
//!
//! Organized by type hierarchy:
//! - Basic types (Context, Namespace, Token) form the foundation
//! - Composite type (Meteor) combines all components
//! - Storage types (TokenBucket) manage collections
//! - Support types (MeteorError) provide infrastructure

mod context;
mod namespace;
mod key;
mod token;
mod meteor;
mod error;

// Re-export all public types
pub use context::Context;
pub use namespace::Namespace;
pub use key::{TokenKey, BracketNotation};
pub use token::{Token, TokenBucket};
pub use meteor::{Meteor, MeteorShower, StorageData};
pub use error::MeteorError;