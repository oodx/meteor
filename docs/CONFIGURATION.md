# Meteor Configuration System

## Overview

Meteor uses a **build-time configuration system** via `meteor.toml` that compiles security limits directly into the binary. This prevents runtime tampering while providing flexible deployment profiles for different use cases.

## Configuration File: `meteor.toml`

The `meteor.toml` file defines deployment profiles and security settings that are read during compilation:

```toml
# Meteor Configuration File
# Build-time configuration for deployment profiles and security limits

[profile]
# Active profile: "default", "enterprise", "embedded", or "strict"
# Can be overridden with METEOR_PROFILE environment variable
active = "default"

[limits.default]
# Balanced limits for general use
max_namespace_part_length = 64
namespace_warning_depth = 3
namespace_error_depth = 4
max_meteors_per_shower = 1000
max_command_history = 1000
max_contexts = 100
max_token_key_length = 128
max_token_value_length = 2048

[limits.enterprise]
# Higher limits for large-scale deployments
max_namespace_part_length = 128
namespace_warning_depth = 5
namespace_error_depth = 8
max_meteors_per_shower = 10000
max_command_history = 10000
max_contexts = 1000
max_token_key_length = 256
max_token_value_length = 8192

[limits.embedded]
# Lower limits for memory-constrained environments
max_namespace_part_length = 32
namespace_warning_depth = 2
namespace_error_depth = 3
max_meteors_per_shower = 100
max_command_history = 100
max_contexts = 10
max_token_key_length = 32
max_token_value_length = 256

[limits.strict]
# Minimal limits for high-security environments
max_namespace_part_length = 16
namespace_warning_depth = 2
namespace_error_depth = 3
max_meteors_per_shower = 50
max_command_history = 500
max_contexts = 5
max_token_key_length = 16
max_token_value_length = 128

[security]
# Security enforcement settings
prevent_runtime_tampering = true
validate_namespace_characters = true
enforce_reserved_words = true
enable_command_audit_trail = true

[features]
# Feature flags for optional functionality
enable_bracket_notation = true
enable_control_commands = true
enable_dot_notation_api = true
enable_stream_continuity = true
```

## Deployment Profiles

### Default Profile
- **Use case**: General-purpose applications
- **Limits**: Balanced for typical workloads (1k meteors, 128 char keys, 2k values)
- **Memory usage**: Moderate
- **Performance**: Good balance of safety and performance

### Enterprise Profile
- **Use case**: Large-scale production deployments
- **Limits**: 10x higher than default (10k meteors, 256 char keys, 8k values)
- **Memory usage**: Higher memory allocation allowed
- **Performance**: Optimized for high-throughput scenarios

### Embedded Profile
- **Use case**: IoT devices, microcontrollers, memory-constrained environments
- **Limits**: Minimal memory footprint (100 meteors, 32 char keys, 256 values)
- **Memory usage**: Heavily constrained
- **Performance**: Optimized for low resource usage

### Strict Profile
- **Use case**: High-security environments, financial systems, crypto applications
- **Limits**: Minimal attack surface (50 meteors, 16 char keys, 128 values)
- **Memory usage**: Tightly controlled
- **Performance**: Security over performance

## Build-Time Configuration

### How It Works

1. **Compilation Phase**: The `build.rs` script reads `meteor.toml`
2. **Code Generation**: Sets compile-time constants based on active profile
3. **Binary Output**: Limits are **permanently baked into the compiled binary**
4. **Runtime**: Configuration cannot be changed - values are compile-time constants

### Build Examples

```bash
# Build with default profile from meteor.toml
cargo build --release

# Override profile via environment variable
METEOR_PROFILE=enterprise cargo build --release

# Build different profiles for different deployments
METEOR_PROFILE=embedded cargo build --release --target=thumbv7em-none-eabihf
METEOR_PROFILE=strict cargo build --release --features=security-hardened
```

### Configuration Inspection

Use the `meteor-config` binary to inspect current configuration:

```bash
# Check current configuration
cargo run --bin meteor-config

# Check different profiles
METEOR_PROFILE=enterprise cargo run --bin meteor-config
METEOR_PROFILE=strict cargo run --bin meteor-config
```

Example output:
```
Meteor Configuration Profile: enterprise (from meteor.toml)
- Max namespace part length: 128
- Namespace warning depth: 5
- Namespace error depth: 8
- Max meteors per shower: 10000
- Max command history: 10000
- Max contexts: 1000
- Max token key length: 256
- Max token value length: 8192
- Runtime tampering prevention: true
- Namespace character validation: true
- Reserved word enforcement: true
- Command audit trail: true
```

## Usage in Applications

### As a Library Dependency

When meteor is used as a library, the configuration is fixed at the library's build time:

```rust
use meteor::{config, MeteorEngine};

fn main() {
    // These values are compile-time constants from meteor.toml
    println!("Profile: {}", config::config_profile());
    println!("Max meteors: {}", config::MAX_METEORS_PER_SHOWER);

    // Create engine with compiled limits
    let engine = MeteorEngine::new();

    // Limits are enforced by the compiled binary
    // No way to exceed MAX_METEORS_PER_SHOWER at runtime
}
```

### Custom Configuration Strategy

For applications needing custom limits:

**Option 1: Fork and Customize**
```bash
git clone https://github.com/oodx/meteor.git custom-meteor
cd custom-meteor
# Edit meteor.toml with custom values
[limits.custom]
max_meteors_per_shower = 50000  # Custom limit
```

**Option 2: Workspace Integration**
```
my-project/
├── Cargo.toml
├── meteor/           ← Git submodule or local copy
│   ├── meteor.toml   ← Customize this file
│   └── src/
└── src/
```

**Option 3: Build-Time Scripting**
```bash
# Automated deployment with different configs
case "$ENVIRONMENT" in
  "production")
    export METEOR_PROFILE=enterprise
    ;;
  "development")
    export METEOR_PROFILE=default
    ;;
  "iot")
    export METEOR_PROFILE=embedded
    ;;
esac
cargo build --release
```

## Security Model

### Build-Time Immutability

**✅ Secure**: Limits compiled into binary
```rust
// These are compile-time constants - cannot be changed
pub const MAX_METEORS_PER_SHOWER: usize = 1000;  // Fixed at build time
```

**❌ Insecure**: Runtime configuration files
```rust
// This would be insecure (we don't do this)
let config = read_config_file("meteor.conf");  // Attackers could modify
let limit = config.max_meteors;                // Runtime tampering possible
```

### Security Benefits

1. **No Configuration Files**: No runtime config files for attackers to modify
2. **No Environment Variables**: Runtime environment cannot change limits
3. **Binary Integrity**: Limits are part of the compiled code
4. **Profile Isolation**: Different security postures via different binaries
5. **Audit Trail**: All security settings visible via `meteor-config`

### Threat Model Protection

- **✅ Config File Tampering**: Prevented - no runtime config files
- **✅ Environment Injection**: Prevented - env vars only affect build time
- **✅ Memory Corruption**: Mitigated - compile-time bounds checking
- **✅ Resource Exhaustion**: Prevented - hard limits compiled in
- **✅ Privilege Escalation**: Mitigated - no runtime config parsing

## Distribution Strategies

### Library Distribution

```toml
[dependencies]
# Standard distribution with default profile
meteor = "0.1.0"

# Enterprise distribution (pre-built with enterprise profile)
meteor = { version = "0.1.0", features = ["enterprise"] }

# Custom distribution from fork
meteor = { git = "https://github.com/myorg/meteor-custom.git" }
```

### Binary Distribution

```bash
# Build different binaries for different deployments
METEOR_PROFILE=default cargo build --release   # General use
METEOR_PROFILE=enterprise cargo build --release # High-performance
METEOR_PROFILE=embedded cargo build --release   # IoT/embedded
METEOR_PROFILE=strict cargo build --release     # High-security

# Distribute appropriate binary for each environment
cp target/release/meteor-enterprise /opt/enterprise/bin/
cp target/release/meteor-embedded /opt/iot/bin/
cp target/release/meteor-strict /opt/security/bin/
```

## Best Practices

### Development Workflow

1. **Development**: Use `default` profile for development
2. **Testing**: Test with target deployment profile before release
3. **Staging**: Use same profile as production for staging
4. **Production**: Use appropriate profile (enterprise/strict/embedded)

### Security Hardening

1. **Profile Selection**: Choose most restrictive profile that meets requirements
2. **Custom Limits**: Set minimum viable limits in custom profiles
3. **Binary Verification**: Use `meteor-config` to verify deployed configuration
4. **Update Process**: Rebuild and redeploy when changing security limits

### Performance Optimization

1. **Profiling**: Use `default` for profiling, then optimize with custom limits
2. **Memory Planning**: Size limits based on available system memory
3. **Throughput Planning**: Size limits based on expected data volumes
4. **Monitoring**: Monitor actual usage vs configured limits

## Migration Guide

### From Cargo Features

If migrating from the previous Cargo features approach:

**Old approach**:
```bash
cargo build --features enterprise
```

**New approach**:
```bash
METEOR_PROFILE=enterprise cargo build
```

### Configuration Mapping

| Old Cargo Feature | New meteor.toml Profile |
|-------------------|-------------------------|
| `--features enterprise` | `METEOR_PROFILE=enterprise` |
| `--features embedded` | `METEOR_PROFILE=embedded` |
| `--features strict` | `METEOR_PROFILE=strict` |
| (default) | `METEOR_PROFILE=default` |

## Troubleshooting

### Common Issues

**Issue**: Configuration not taking effect
```bash
# Solution: Force rebuild to pick up meteor.toml changes
cargo clean && cargo build
```

**Issue**: Wrong profile in production
```bash
# Solution: Verify configuration before deployment
cargo run --bin meteor-config

# Should show expected profile and limits
```

**Issue**: Custom limits not working
```bash
# Solution: Verify meteor.toml syntax and active profile
cat meteor.toml | grep -A 10 "\[limits\."
```

### Debugging

```bash
# Check which profile is active
cargo run --bin meteor-config | head -1

# Verify all limits
cargo run --bin meteor-config

# Test with different profiles
METEOR_PROFILE=enterprise cargo clean && cargo run --bin meteor-config
```

## Advanced Configuration

### Custom Profiles

Add custom profiles to `meteor.toml`:

```toml
[limits.financial]
# Custom profile for financial applications
max_meteors_per_shower = 5000
max_command_history = 50000  # Extensive audit requirements
max_contexts = 50
max_token_key_length = 64
max_token_value_length = 1024

[limits.gaming]
# Custom profile for gaming applications
max_meteors_per_shower = 20000  # High throughput
max_command_history = 1000      # Moderate audit
max_contexts = 500              # Many game contexts
max_token_key_length = 32       # Short keys
max_token_value_length = 512    # Small values
```

### Conditional Compilation

Use profiles for conditional features:

```rust
#[cfg(meteor_enterprise)]
pub fn enterprise_only_feature() {
    // Only available in enterprise builds
}

#[cfg(not(meteor_embedded))]
pub fn memory_intensive_feature() {
    // Not available in embedded builds
}
```

### Integration Testing

Test with different profiles in CI:

```yaml
# .github/workflows/test.yml
strategy:
  matrix:
    profile: [default, enterprise, embedded, strict]
steps:
  - name: Test with ${{ matrix.profile }}
    run: |
      METEOR_PROFILE=${{ matrix.profile }} cargo test
      METEOR_PROFILE=${{ matrix.profile }} cargo run --bin meteor-config
```

---

This configuration system provides the perfect balance of **security** (build-time immutability), **flexibility** (multiple deployment profiles), and **usability** (simple TOML configuration) for the Meteor token transport library.