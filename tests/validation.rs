//! Meteor Validation Tests
//!
//! TICKET-003: Comprehensive validation tests for MeteorShower storage implementation.
//! These tests validate that MeteorShower can fully replace TokenBucket as primary storage.

extern crate meteor;

// Include the validation test module
mod validation {
    include!("validation/meteorshower_storage.rs");
}