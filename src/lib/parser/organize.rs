//! Token organization infrastructure
//!
//! This module contained token organization logic but is now handled
//! by TokenBucket and MeteorShower collection types directly.
//! GUTTED: Logic moved to proper collection implementations.

// TODO: Consider if any shared organization utilities are needed
// Organization is now handled by TokenBucket::parse() and MeteorShower::parse()