//! Parser module for Meteor token processing
//!
//! GUTTED: Removing outdated exports, will be replaced with modern shared parsing infrastructure.
//!
//! Future structure:
//! - token.rs: Shared token parsing logic
//! - meteor.rs: Shared meteor parsing logic
//! - validate.rs: Validation utilities
//! - macros.rs: Parse and validation macros
//! - bracket.rs: Bracket notation (already correct)

pub mod bracket;
// TODO: Add new modules:
// pub mod token;
// pub mod meteor;
// pub mod validate;
// pub mod macros;

// Keep existing bracket exports (these are correct)
pub use bracket::{transform_key, reverse_transform_key};

// TODO: Add new exports:
// pub use token::parse_token;
// pub use meteor::parse_meteor;
// pub use validate::{validate_token, validate_meteor};
// pub use macros::*;