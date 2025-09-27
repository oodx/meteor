# Profile Testing Guide

## Overview

Meteor's namespace depth validation is **profile-aware**, meaning depth limits vary based on the active deployment profile (`default`, `enterprise`, `embedded`, `strict`). This guide explains how to test namespace depth behavior across all profiles.

## Namespace Depth Limits by Profile

| Profile | Warning Depth | Error Depth | Use Case |
|---------|---------------|-------------|----------|
| **default** | 5 | 6 | General-purpose applications |
| **enterprise** | 6 | 8 | Large-scale production deployments |
| **embedded** | 3 | 4 | IoT/memory-constrained environments |
| **strict** | 3 | 4 | High-security environments |

**Depth Rules:**
- **Clear Zone**: `depth < warning_depth` - No warnings, fully valid
- **Warning Zone**: `warning_depth <= depth < error_depth` - Valid but triggers warnings
- **Error Zone**: `depth >= error_depth` - Validation fails with error

## Testing Locally

### Test with Default Profile

```bash
# Default profile from meteor.toml
cargo test --test test_namespace_depth_regression
```

### Test with Specific Profile

```bash
# Test enterprise profile (higher limits)
METEOR_PROFILE=enterprise cargo test --test test_namespace_depth_regression

# Test strict profile (lower limits)
METEOR_PROFILE=strict cargo test --test test_namespace_depth_regression

# Test embedded profile (minimal limits)
METEOR_PROFILE=embedded cargo test --test test_namespace_depth_regression
```

### Test All Profiles in Sequence

```bash
# Bash script to test all profiles
for profile in default enterprise embedded strict; do
  echo "Testing profile: $profile"
  METEOR_PROFILE=$profile cargo clean
  METEOR_PROFILE=$profile cargo test --test test_namespace_depth_regression
done
```

### Verify Active Profile Configuration

```bash
# Check current profile limits
cargo run --bin meteor-config

# Check specific profile limits
METEOR_PROFILE=enterprise cargo run --bin meteor-config
METEOR_PROFILE=strict cargo run --bin meteor-config
```

## Regression Test Suite

The namespace depth regression test suite (`tests/test_namespace_depth_regression.rs`) validates:

### Core Profile Tests

1. **test_depth_profile_respects_clear_threshold**
   - Validates namespaces below warning threshold pass without warnings

2. **test_depth_profile_respects_warning_threshold**
   - Validates namespaces at warning threshold trigger warnings but remain valid

3. **test_depth_profile_respects_error_threshold**
   - Validates namespaces at/above error threshold fail validation

4. **test_depth_profile_beyond_error_fails**
   - Validates namespaces beyond error threshold consistently fail

### Profile-Specific Tests

5. **test_profile_default_mode**
   - Validates default profile has reasonable limits (4-8 error depth)

6. **test_profile_enterprise_mode**
   - Validates enterprise profile has relaxed limits (≥6 error depth)

7. **test_profile_embedded_mode**
   - Validates embedded profile has tight limits (≤4 error depth)

8. **test_profile_strict_mode**
   - Validates strict profile has minimal limits (≤4 error depth)

### Progressive Validation Tests

9. **test_progressive_depth_validation**
   - Validates all depths 1-10 behave correctly relative to profile limits

10. **test_edge_case_max_depth_minus_one**
    - Validates the boundary case at `error_depth - 1` succeeds

11. **test_real_world_namespace_patterns**
    - Validates common namespace patterns (`ui`, `ui.widgets`, etc.)

### Integration Tests

12. **test_profile_default_depth_assumptions**
    - Validates profile consistency (error ≥ warning, both positive)

13. **test_profile_consistency_with_namespace_validation**
    - Validates Namespace type respects depth limits during validation

## Continuous Integration

### GitHub Actions Workflow

The `.github/workflows/profile-regression.yml` workflow automatically tests all profiles on every push/PR:

```yaml
strategy:
  matrix:
    profile: [default, enterprise, embedded, strict]
```

**CI Validation Steps:**
1. **Build**: Compile with each profile using `METEOR_PROFILE` env var
2. **Test**: Run full test suite with profile-specific limits
3. **Regression**: Run namespace depth regression suite
4. **Lint**: Check formatting and clippy (default profile only)

### Local CI Simulation

Run the same tests CI runs:

```bash
# Simulate GitHub Actions matrix testing
for profile in default enterprise embedded strict; do
  echo "=== Testing profile: $profile ==="

  # Clean build
  METEOR_PROFILE=$profile cargo clean

  # Build with profile
  METEOR_PROFILE=$profile cargo build --verbose

  # Run all tests
  METEOR_PROFILE=$profile cargo test --verbose

  # Run namespace regression specifically
  METEOR_PROFILE=$profile cargo test --test test_namespace_depth_regression --verbose

  echo "✅ Profile $profile passed"
  echo ""
done
```

## Writing Profile-Aware Tests

### Using cfg Attributes

```rust
#[test]
fn test_profile_specific_behavior() {
    if cfg!(meteor_enterprise) {
        // Enterprise-specific assertions
        assert!(NAMESPACE_ERROR_DEPTH >= 6);
    }

    if cfg!(meteor_strict) {
        // Strict-specific assertions
        assert!(NAMESPACE_ERROR_DEPTH <= 4);
    }
}
```

### Using Compile-Time Constants

```rust
use meteor::types::{NAMESPACE_ERROR_DEPTH, NAMESPACE_WARNING_DEPTH};

#[test]
fn test_depth_aware_validation() {
    let depth = 5;
    let should_error = depth >= NAMESPACE_ERROR_DEPTH;

    // Test logic adapts to active profile limits
    if should_error {
        assert!(Namespace::try_from_string("a.b.c.d.e").is_err());
    } else {
        assert!(Namespace::try_from_string("a.b.c.d.e").is_ok());
    }
}
```

### Runtime Profile Checking

```rust
use meteor::config::config_profile;

#[test]
fn test_runtime_profile_detection() {
    let profile = config_profile();
    match profile {
        "enterprise" => { /* enterprise test logic */ },
        "strict" => { /* strict test logic */ },
        _ => { /* default test logic */ }
    }
}
```

## Troubleshooting

### Test Fails in One Profile Only

If tests pass in `default` but fail in `strict`:

```bash
# 1. Check the profile limits
METEOR_PROFILE=strict cargo run --bin meteor-config

# 2. Run specific test with verbose output
METEOR_PROFILE=strict cargo test test_namespace_depth -- --nocapture

# 3. Check if namespace depth exceeds strict limits (error_depth=4)
```

### Incorrect Profile Applied

If tests don't reflect expected limits:

```bash
# Force rebuild to pick up profile
METEOR_PROFILE=enterprise cargo clean
METEOR_PROFILE=enterprise cargo build

# Verify profile was applied
METEOR_PROFILE=enterprise cargo run --bin meteor-config
```

### CI Fails but Local Passes

```bash
# Ensure clean build like CI does
cargo clean

# Test exactly as CI does
METEOR_PROFILE=strict cargo test --verbose

# Check for profile-dependent constants
grep -r "NAMESPACE_ERROR_DEPTH\|NAMESPACE_WARNING_DEPTH" src/
```

## Best Practices

### For Test Authors

1. **Profile Agnostic**: Write tests that work across all profiles by using constants (`NAMESPACE_ERROR_DEPTH`) instead of hardcoded values
2. **Boundary Testing**: Test at `warning_depth - 1`, `warning_depth`, `error_depth - 1`, and `error_depth`
3. **Progressive Validation**: Test multiple depths in sequence to catch off-by-one errors
4. **Clear Assertions**: Include the expected threshold in assertion messages for debugging

### For CI Configuration

1. **Matrix Testing**: Always test all profiles (`default`, `enterprise`, `embedded`, `strict`)
2. **Clean Builds**: Run `cargo clean` between profile tests to avoid stale constants
3. **Explicit Verification**: Display active profile config at start of each CI job
4. **Fail Fast**: Use `fail-fast: false` to see results from all profiles even if one fails

### For Downstream Consumers

1. **Profile Selection**: Choose the profile that matches your deployment environment
2. **Test Before Deploy**: Run regression tests with your target profile before deployment
3. **CI Integration**: Configure CI to test with your production profile
4. **Validation**: Use `Namespace::try_from_string()` for validation before passing to engine

## Example: Testing a New Namespace Feature

```rust
#[test]
fn test_my_namespace_feature() {
    // ✅ Good: Profile-agnostic using constants
    let valid_depth = NAMESPACE_WARNING_DEPTH - 1;
    let ns = create_namespace(valid_depth);
    assert!(ns.is_ok());

    // ❌ Bad: Hardcoded depth (fails in strict/embedded profiles)
    let ns = Namespace::try_from_string("a.b.c.d.e.f");
    assert!(ns.is_ok()); // Fails when error_depth < 6
}
```

## Profile Override Environment Variable

The `METEOR_PROFILE` environment variable overrides the `meteor.toml` active profile:

```bash
# Override at build time
METEOR_PROFILE=enterprise cargo build

# Override at test time (requires rebuild)
METEOR_PROFILE=enterprise cargo clean && cargo test
```

**Important**: The profile is **compiled into the binary**, so changing `METEOR_PROFILE` requires a rebuild. Tests run against the limits from the last build.

## Summary

- **4 Profiles**: default (balanced), enterprise (relaxed), embedded/strict (tight)
- **Depth Validation**: Enforced at compile-time via build.rs reading meteor.toml
- **Test Suite**: 13 regression tests validate depth behavior across profiles
- **CI/CD**: GitHub Actions matrix tests all profiles automatically
- **Local Testing**: Use `METEOR_PROFILE=<profile>` with `cargo clean && cargo test`
- **Best Practice**: Write profile-agnostic tests using `NAMESPACE_ERROR_DEPTH` constants

For more details on configuration profiles, see `docs/CONFIGURATION.md`.