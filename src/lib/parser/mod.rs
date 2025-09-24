//! Parser Module - Validation and Delegation
//!
//! This module provides portable parsing logic that validates input
//! and delegates state/data operations to MeteorEngine.
//!
//! Parser modules handle:
//! - Input validation (format, syntax, escapes)
//! - Control token recognition (ctl:delete=path, ctl:reset=cursor)
//! - Delegation to MeteorEngine for state/data changes
//!
//! Parser modules DO NOT handle:
//! - State management (that's MeteorEngine's job)
//! - Storage operations (that's MeteorEngine's job)
//! - Command history (that's MeteorEngine's job)

pub mod token_stream;
pub mod meteor_stream;
pub mod escape;

pub use token_stream::TokenStreamParser;
pub use meteor_stream::MeteorStreamParser;
pub use escape::{parse_escaped_value, validate_escapes};