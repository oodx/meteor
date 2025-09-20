# RSB Token Implementation Reference

This directory contains a complete copy of the RSB token implementation and tests for reference during meteor development.

## Source Code (`rsb_token_src/`)

Complete RSB token module implementation:

- **`bucket.rs`** - TokenBucket storage and organization
- **`error.rs`** - Error types and conversions
- **`format.rs`** - Token formatting utilities
- **`helpers.rs`** - Internal helper functions
- **`macros.rs`** - Token-related macros
- **`mod.rs`** - Module orchestrator and exports
- **`parse.rs`** - Token parsing logic
- **`types.rs`** - Core token types and structs
- **`utils.rs`** - Public utility functions

## Test Coverage (`rsb_token_tests/`)

Comprehensive test suite for token functionality:

- **`sanity_token.rs`** - Core functionality validation tests
- **`uat_token.rs`** - User acceptance tests with visual demos
- **`comprehensive.rs`** - Comprehensive test coverage (copied from unit_tokens/)
- **`features_tokens.rs`** - Feature wrapper that includes comprehensive.rs
- **`token.rs`** - Additional token tests
- **`unit_tokens/comprehensive.rs`** - Original comprehensive tests location

## Usage Notes

### For Meteor Development:
1. **Reference implementation** - Use rsb_token_src/ as starting point for meteor types/
2. **Test patterns** - Use rsb_token_tests/ as template for meteor test structure
3. **API patterns** - Follow RSB string-biased interfaces and error handling
4. **Functionality scope** - Keep core parsing/bucket logic, adapt for context-namespace-key pattern

### Key Differences to Implement:
- **Context management** - Add context origin tracking (app, user, system, file1, etc.)
- **Namespace hierarchy** - Support dot notation with depth warnings
- **Bracket notation** - Transform `key[0,1]` → `key__i_0_1` patterns
- **Simplified API** - Focus on essential operations, remove RSB-specific features

### Migration Strategy:
1. Start with types.rs and bucket.rs as foundation
2. Adapt parse.rs for context:namespace:key pattern
3. Simplify utils.rs to essential meteor operations
4. Create new error.rs focused on meteor concerns
5. Skip macros.rs and format.rs unless needed

## Token Namespace Concept

See `TOKEN_NAMESPACE_CONCEPT.md` for the complete specification of meteor's addressing scheme that builds on this RSB foundation.