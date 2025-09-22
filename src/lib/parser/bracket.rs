//! Bracket notation transformation algorithms
//!
//! This module implements the complex bracket notation parsing and transformation
//! logic, isolated from the public API to maintain clean interfaces.
//!
//! # Transformations
//!
//! - `list[0]` → `list__i_0`
//! - `grid[2,3]` → `grid__i_2_3`
//! - `queue[]` → `queue__i_APPEND`
//! - `matrix[x,y,z]` → `matrix__i_x_y_z`

use crate::types::MeteorError;

/// Transform a key with bracket notation to dunder notation
///
/// # Examples
///
/// ```ignore
/// use meteor::sup::bracket::transform_key;
///
/// assert_eq!(transform_key("list[0]").unwrap(), "list__i_0");
/// assert_eq!(transform_key("grid[2,3]").unwrap(), "grid__i_2_3");
/// assert_eq!(transform_key("queue[]").unwrap(), "queue__i_APPEND");
/// assert_eq!(transform_key("normal_key").unwrap(), "normal_key");
/// ```
pub fn transform_key(key: &str) -> Result<String, MeteorError> {
    // If no brackets, return as-is
    if !key.contains('[') && !key.contains(']') {
        return Ok(key.to_string());
    }

    // If only one bracket type is present, that's an error
    if key.contains('[') != key.contains(']') {
        return Err(MeteorError::invalid_bracket(key, "mismatched brackets"));
    }

    // Parse the bracket notation
    let (base, indices) = parse_bracket_notation(key)?;

    // Transform to dunder notation
    if indices.is_empty() {
        // Empty brackets: list[] → list__i_APPEND
        Ok(format!("{}__i_APPEND", base))
    } else {
        // Indexed brackets: list[0,1] → list__i_0_1
        Ok(format!("{}__i_{}", base, indices.join("_")))
    }
}

/// Parse bracket notation into base name and indices
///
/// Returns (base_name, Vec<index_strings>)
fn parse_bracket_notation(key: &str) -> Result<(String, Vec<String>), MeteorError> {
    // Find bracket positions
    let open_pos = key.find('[')
        .ok_or_else(|| MeteorError::invalid_bracket(key, "missing opening bracket"))?;

    let close_pos = key.rfind(']')
        .ok_or_else(|| MeteorError::invalid_bracket(key, "missing closing bracket"))?;

    if close_pos <= open_pos {
        return Err(MeteorError::invalid_bracket(key, "malformed bracket order"));
    }

    // Extract base name and bracket content
    let base = key[..open_pos].to_string();
    let bracket_content = &key[open_pos + 1..close_pos];

    // Check for nested brackets (not supported in basic implementation)
    if bracket_content.contains('[') || bracket_content.contains(']') {
        return Err(MeteorError::invalid_bracket(key, "nested brackets not supported"));
    }

    // Parse indices
    let indices = if bracket_content.is_empty() {
        // Empty brackets: []
        Vec::new()
    } else {
        // Split by comma and validate each index
        bracket_content
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| validate_index(s, key))
            .collect::<Result<Vec<String>, MeteorError>>()?
    };

    // Validate base name
    if base.is_empty() {
        return Err(MeteorError::invalid_bracket(key, "empty base name"));
    }

    Ok((base, indices))
}

/// Validate a single index in bracket notation
fn validate_index(index: &str, original_key: &str) -> Result<String, MeteorError> {
    if index.is_empty() {
        return Err(MeteorError::invalid_bracket(original_key, "empty index"));
    }

    // For now, allow any non-empty string as an index
    // TODO: Add more sophisticated validation based on requirements
    // - Numeric indices: 0, 1, 42
    // - String indices: "key", "name"
    // - Variable indices: x, y, variable_name

    // Basic character validation - no special characters that could break parsing
    for (pos, ch) in index.char_indices() {
        match ch {
            // Allow alphanumeric, underscore, hyphen
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' => continue,
            _ => {
                return Err(MeteorError::invalid_char(
                    ch,
                    pos,
                    format!("bracket index in {}", original_key),
                ));
            }
        }
    }

    Ok(index.to_string())
}

/// Check if a key contains bracket notation
pub fn has_brackets(key: &str) -> bool {
    key.contains('[') && key.contains(']')
}

/// Extract just the base name from a potentially bracketed key
pub fn extract_base_name(key: &str) -> Result<String, MeteorError> {
    if !has_brackets(key) {
        return Ok(key.to_string());
    }

    let open_pos = key.find('[')
        .ok_or_else(|| MeteorError::invalid_bracket(key, "missing opening bracket"))?;

    Ok(key[..open_pos].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_simple_index() {
        assert_eq!(transform_key("list[0]").unwrap(), "list__i_0");
        assert_eq!(transform_key("items[42]").unwrap(), "items__i_42");
    }

    #[test]
    fn test_transform_multiple_indices() {
        assert_eq!(transform_key("grid[2,3]").unwrap(), "grid__i_2_3");
        assert_eq!(transform_key("matrix[x,y,z]").unwrap(), "matrix__i_x_y_z");
    }

    #[test]
    fn test_transform_empty_brackets() {
        assert_eq!(transform_key("queue[]").unwrap(), "queue__i_APPEND");
        assert_eq!(transform_key("list[]").unwrap(), "list__i_APPEND");
    }

    #[test]
    fn test_transform_no_brackets() {
        assert_eq!(transform_key("normal_key").unwrap(), "normal_key");
        assert_eq!(transform_key("snake_case").unwrap(), "snake_case");
    }

    #[test]
    fn test_transform_errors() {
        // Missing brackets
        assert!(transform_key("list[").is_err());
        assert!(transform_key("list]").is_err());

        // Empty base name
        assert!(transform_key("[0]").is_err());

        // Nested brackets (not supported)
        assert!(transform_key("list[grid[0]]").is_err());
    }

    #[test]
    fn test_has_brackets() {
        assert!(has_brackets("list[0]"));
        assert!(has_brackets("grid[x,y]"));
        assert!(!has_brackets("normal_key"));
        assert!(!has_brackets("key_with_underscore"));
    }

    #[test]
    fn test_extract_base_name() {
        assert_eq!(extract_base_name("list[0]").unwrap(), "list");
        assert_eq!(extract_base_name("grid[x,y]").unwrap(), "grid");
        assert_eq!(extract_base_name("normal_key").unwrap(), "normal_key");
    }

    #[test]
    fn test_validate_index() {
        assert!(validate_index("0", "list[0]").is_ok());
        assert!(validate_index("variable_name", "list[variable_name]").is_ok());
        assert!(validate_index("x", "grid[x]").is_ok());

        // Invalid characters
        assert!(validate_index("", "list[]").is_err());
        assert!(validate_index("a[b]", "list[a[b]]").is_err());
    }

    #[test]
    fn test_whitespace_handling() {
        assert_eq!(transform_key("list[ 0 ]").unwrap(), "list__i_0");
        assert_eq!(transform_key("grid[ x , y ]").unwrap(), "grid__i_x_y");
    }
}

/// Reverse transform a flat key back to bracket notation (best effort)
///
/// Attempts to reconstruct bracket notation from flat keys.
/// May not be perfect for all cases, but handles common patterns.
///
/// # Examples
///
/// ```ignore
/// use meteor::parser::bracket::reverse_transform_key;
///
/// assert_eq!(reverse_transform_key("list__i_0").unwrap(), "list[0]");
/// assert_eq!(reverse_transform_key("grid__i_2_3").unwrap(), "grid[2,3]");
/// assert_eq!(reverse_transform_key("queue__i_APPEND").unwrap(), "queue[]");
/// assert_eq!(reverse_transform_key("normal_key").unwrap(), "normal_key");
/// ```
pub fn reverse_transform_key(flat_key: &str) -> Option<String> {
    // Pattern for dunder notation: base__i_indices or base__name
    if let Some(dunder_pos) = flat_key.find("__") {
        let base = &flat_key[..dunder_pos];
        let suffix = &flat_key[dunder_pos + 2..];

        // Check if it's an index pattern (starts with "i_")
        if suffix.starts_with("i_") {
            let indices = &suffix[2..]; // Remove "i_" prefix

            if indices == "APPEND" {
                // Special case for append
                return Some(format!("{}[]", base));
            } else if indices.contains('_') {
                // Multi-dimensional indices
                let coords: Vec<&str> = indices.split('_').collect();
                return Some(format!("{}[{}]", base, coords.join(",")));
            } else {
                // Single index
                return Some(format!("{}[{}]", base, indices));
            }
        } else {
            // Named index (no "i_" prefix)
            return Some(format!("{}[{}]", base, suffix));
        }
    }

    // No dunder pattern found, return as-is
    Some(flat_key.to_string())
}

#[cfg(test)]
mod reverse_tests {
    use super::*;

    #[test]
    fn test_reverse_transform_numeric() {
        assert_eq!(reverse_transform_key("list__i_0").unwrap(), "list[0]");
        assert_eq!(reverse_transform_key("items__i_42").unwrap(), "items[42]");
    }

    #[test]
    fn test_reverse_transform_multi_dimensional() {
        assert_eq!(reverse_transform_key("grid__i_2_3").unwrap(), "grid[2,3]");
        assert_eq!(reverse_transform_key("matrix__i_x_y_z").unwrap(), "matrix[x,y,z]");
    }

    #[test]
    fn test_reverse_transform_append() {
        assert_eq!(reverse_transform_key("queue__i_APPEND").unwrap(), "queue[]");
        assert_eq!(reverse_transform_key("list__i_APPEND").unwrap(), "list[]");
    }

    #[test]
    fn test_reverse_transform_named() {
        assert_eq!(reverse_transform_key("person__name").unwrap(), "person[name]");
        assert_eq!(reverse_transform_key("config__database").unwrap(), "config[database]");
    }

    #[test]
    fn test_reverse_transform_no_dunder() {
        assert_eq!(reverse_transform_key("simple_key").unwrap(), "simple_key");
        assert_eq!(reverse_transform_key("button").unwrap(), "button");
    }
}