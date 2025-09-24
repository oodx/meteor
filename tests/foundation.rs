//! Foundation test suite for Meteor core types
//!
//! Run with: cargo test --test foundation

// Include the foundation test modules
#[path = "foundation/token_key.rs"]
mod token_key_tests;

#[path = "foundation/token.rs"]
mod token_tests;

#[path = "foundation/meteor.rs"]
mod meteor_tests;