//! Parser Module - Validation and Delegation
//!
//! This module provides portable parsing logic that validates input
//! and delegates state/data operations to MeteorEngine.
//!
//! Parser modules handle:
//! - Input validation (format, syntax, escapes)
//! - Control token recognition (ctl:delete=path, ctl:reset=cursor)
//! - Delegation to MeteorEngine for state/data changes
//! - ENG-41: Token aggregation and meteor-aware parsing with hardened constructors
//!
//! Parser modules DO NOT handle:
//! - State management (that's MeteorEngine's job)
//! - Storage operations (that's MeteorEngine's job)
//! - Command history (that's MeteorEngine's job)
//!
//! ## Architecture Updates
//!
//! ### ENG-40: Hardened Constructor Integration
//! Parsers now leverage hardened meteor constructors for better validation:
//! - `Meteor::try_new_with_tokens()` for safe construction with error handling
//! - Namespace consistency validation across all tokens in a meteor
//! - Descriptive error messages for debugging and user feedback
//!
//! ### ENG-41: Parser Alignment with Meteor Aggregation
//! Enhanced parsing methods group tokens by (context, namespace) before meteor creation:
//! - `process_with_aggregation()` methods for both TokenStreamParser and MeteorStreamParser
//! - Automatic token grouping reduces meteor construction overhead
//! - Consistent error handling through MeteorError instead of String errors
//! - Legacy methods preserved for backward compatibility

pub mod escape;
pub mod meteor_stream;
pub mod split;
pub mod token_stream;

pub use escape::{parse_escaped_value, validate_escapes};
pub use meteor_stream::MeteorStreamParser;
pub use split::{smart_split, smart_split_borrowed, smart_split_multi_char, smart_split_semicolons, SplitConfig};
pub use token_stream::TokenStreamParser;
