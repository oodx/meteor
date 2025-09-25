//! BracketNotation trait for bracket notation handling

/// Trait for types that can handle bracket notation transformations
///
/// This trait provides a standard interface for converting between
/// bracket notation (e.g., `list[0]`) and flat notation (e.g., `list__i_0`).
pub trait BracketNotation {
    /// Transform flat key back to bracket notation (best effort)
    ///
    /// Attempts to reconstruct the original bracket notation from a flat key.
    /// May not be perfect for all cases, but handles common patterns.
    fn to_bracket(&self) -> String;

    /// Check if contains bracket notation patterns
    fn has_brackets(&self) -> bool;
}

impl BracketNotation for str {
    fn to_bracket(&self) -> String {
        super::bracket::reverse_transform_key(self).unwrap_or_else(|| self.to_string())
    }

    fn has_brackets(&self) -> bool {
        self.contains('[') && self.contains(']')
    }
}

impl BracketNotation for String {
    fn to_bracket(&self) -> String {
        self.as_str().to_bracket()
    }

    fn has_brackets(&self) -> bool {
        self.as_str().has_brackets()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_bracket_notation() {
        let original = "list[0]";
        assert!(original.has_brackets());

        let flat = "list__i_0";
        assert!(!flat.has_brackets());
        assert_eq!(flat.to_bracket(), "list[0]");
    }

    #[test]
    fn test_string_no_brackets() {
        let simple = "button";
        assert!(!simple.has_brackets());
        assert_eq!(simple.to_bracket(), "button");
    }
}
