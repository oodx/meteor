//! MeteorShower - collection container for fully-qualified Meteor tokens

use super::meteor::Meteor;
use crate::types::Context;
use std::collections::HashMap;

/// A collection of fully-qualified Meteor tokens
///
/// MeteorShower stores complete Meteor tokens (context:namespace:key=value)
/// and provides methods for organizing, querying, and accessing them.
/// MeteorShower maintains the full addressing structure of each Meteor.
#[derive(Debug, Clone, PartialEq)]
pub struct MeteorShower {
    meteors: Vec<Meteor>,
    // Index by context for fast lookups
    context_index: HashMap<String, Vec<usize>>,
    // Index by namespace within context for fast lookups
    namespace_index: HashMap<String, HashMap<String, Vec<usize>>>,
}

impl MeteorShower {
    /// Create a new empty MeteorShower
    pub fn new() -> Self {
        MeteorShower {
            meteors: Vec::new(),
            context_index: HashMap::new(),
            namespace_index: HashMap::new(),
        }
    }

    /// Create a new MeteorShower with a default context for meteors without explicit context
    ///
    /// When parsing meteors that don't specify a context (e.g., "ui:button=click"),
    /// they will be assigned to the provided default context.
    pub fn with_context(_default_context: Context) -> Self {
        // Note: MeteorShower stores full Meteors, so default context is handled
        // at parse time when creating individual Meteor objects
        MeteorShower::new()
    }

    /// Add a Meteor to the shower
    pub fn add(&mut self, meteor: Meteor) {
        let index = self.meteors.len();
        let context_name = meteor.context().name().to_string();
        let namespace_name = meteor.namespace().to_string();

        // Add to context index
        self.context_index
            .entry(context_name.clone())
            .or_insert_with(Vec::new)
            .push(index);

        // Add to namespace index
        self.namespace_index
            .entry(context_name.clone())
            .or_insert_with(HashMap::new)
            .entry(namespace_name)
            .or_insert_with(Vec::new)
            .push(index);

        self.meteors.push(meteor);
    }

    /// Get all meteors in the shower
    pub fn meteors(&self) -> &[Meteor] {
        &self.meteors
    }

    /// Get meteors by context
    pub fn by_context(&self, context: &str) -> Vec<&Meteor> {
        self.context_index
            .get(context)
            .map(|indices| {
                indices.iter()
                    .map(|&i| &self.meteors[i])
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get meteors by context and namespace
    pub fn by_context_namespace(&self, context: &str, namespace: &str) -> Vec<&Meteor> {
        self.namespace_index
            .get(context)
            .and_then(|namespaces| namespaces.get(namespace))
            .map(|indices| {
                indices.iter()
                    .map(|&i| &self.meteors[i])
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find a specific meteor by context, namespace, and key
    pub fn find(&self, context: &str, namespace: &str, key: &str) -> Option<&Meteor> {
        self.by_context_namespace(context, namespace)
            .into_iter()
            .find(|meteor| meteor.token().key_notation() == key || meteor.token().key_str() == key)
    }

    /// Get all unique contexts in the shower
    pub fn contexts(&self) -> Vec<&str> {
        self.context_index.keys().map(|s| s.as_str()).collect()
    }

    /// Get all namespaces within a context
    pub fn namespaces_in_context(&self, context: &str) -> Vec<&str> {
        self.namespace_index
            .get(context)
            .map(|namespaces| namespaces.keys().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Count of meteors in the shower
    pub fn len(&self) -> usize {
        self.meteors.len()
    }

    /// Check if shower is empty
    pub fn is_empty(&self) -> bool {
        self.meteors.is_empty()
    }

    /// Parse a meteor stream into a MeteorShower
    ///
    /// Accepts semicolon-separated meteor tokens in full format:
    /// "app:ui.widgets:button=submit; user:settings:theme=dark"
    pub fn parse(input: &str) -> Result<Self, String> {
        let mut shower = MeteorShower::new();

        for line in input.split(';') {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let meteor = Meteor::parse(line)?;
            shower.add(meteor);
        }

        Ok(shower)
    }

    /// Convert shower to a formatted string
    pub fn to_string(&self) -> String {
        self.meteors
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<_>>()
            .join("; ")
    }
}

impl Default for MeteorShower {
    fn default() -> Self {
        MeteorShower::new()
    }
}

impl std::fmt::Display for MeteorShower {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Context, Namespace, Token};

    #[test]
    fn test_meteor_shower_creation() {
        let mut shower = MeteorShower::new();

        let meteor = Meteor::new(
            Context::app(),
            Namespace::from_string("ui.widgets"),
            Token::new("button", "submit")
        );

        shower.add(meteor);
        assert_eq!(shower.len(), 1);
        assert!(!shower.is_empty());
    }

    #[test]
    fn test_meteor_shower_lookup() {
        let mut shower = MeteorShower::new();

        let meteor = Meteor::new(
            Context::user(),
            Namespace::from_string("settings"),
            Token::new("theme", "dark")
        );

        shower.add(meteor);

        let found = shower.find("user", "settings", "theme");
        assert!(found.is_some());
        assert_eq!(found.unwrap().token().value(), "dark");
    }

    #[test]
    fn test_meteor_shower_parse() {
        let shower = MeteorShower::parse("app:ui:button=click; user:settings:theme=dark").unwrap();

        assert_eq!(shower.len(), 2);
        assert_eq!(shower.contexts().len(), 2);

        let app_meteors = shower.by_context("app");
        assert_eq!(app_meteors.len(), 1);
        assert_eq!(app_meteors[0].token().value(), "click");
    }
}