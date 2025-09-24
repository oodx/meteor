# Meteor Configuration Examples

## Quick Demo

Run the configuration demo to see how meteor.toml profiles work:

```bash
# Default profile
cargo run --example config_demo

# Enterprise profile (requires clean build)
cargo clean && METEOR_PROFILE=enterprise cargo run --example config_demo

# Strict profile (requires clean build)
cargo clean && METEOR_PROFILE=strict cargo run --example config_demo

# Embedded profile (requires clean build)
cargo clean && METEOR_PROFILE=embedded cargo run --example config_demo
```

## Configuration Inspection

```bash
# Check current configuration
cargo run --bin meteor-config

# With different profiles
METEOR_PROFILE=enterprise cargo clean && cargo run --bin meteor-config
METEOR_PROFILE=strict cargo clean && cargo run --bin meteor-config
```

## Profile Comparison

| Profile | Meteors | History | Contexts | Key Len | Value Len |
|---------|---------|---------|----------|---------|-----------|
| default | 1,000   | 1,000   | 100      | 128     | 2,048     |
| enterprise | 10,000 | 10,000  | 1,000    | 256     | 8,192     |
| embedded | 100     | 100     | 10       | 32      | 256       |
| strict   | 50      | 500     | 5        | 16      | 128       |

## Security Features

✅ **Build-time configuration**: Limits compiled into binary
✅ **No runtime tampering**: Configuration cannot be changed at runtime
✅ **Environment override**: `METEOR_PROFILE` affects build time only
✅ **Profile isolation**: Different binaries for different security postures

## Documentation

📖 [Full Configuration Guide](../docs/CONFIGURATION.md)