//! Token transformation infrastructure
//!
//! This module contained token transformation logic but bracket notation
//! is now handled by the TokenKey type via the BracketTransform trait.
//! GUTTED: Logic moved to proper TokenKey implementation.

// TODO: Consider if any shared transformation utilities are needed
// Most transformation is now handled by TokenKey::transform_key() and reverse_transform_key()