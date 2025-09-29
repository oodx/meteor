//! Export/Import subsystem for namespace data virtualization
//!
//! ## Content Patterns (Bracket Notation)
//!
//! The export system recognizes standard bracket patterns for content organization.
//! These patterns provide type hints for export/import plugins while maintaining
//! flexibility - the system doesn't enforce usage, just provides detection.
//!
//! ### Document Patterns
//! - `section[intro]` - Document sections (singular form)
//! - `section[10_setup]` - Numbered sections for ordering
//! - `full` - Canonical whole document
//! - `metadata[type]` - Content metadata
//!
//! ### Script/Code Patterns
//! - `part[header]` - Script parts (singular form)
//! - `part[20_install]` - Numbered parts for ordering
//! - `chunk[12212121]` - Content chunks by numeric hash/ID
//! - `chunk[A18BfD]` - Content chunks by hex ID
//! - `function[parse]` - Function definitions (full form)
//! - `func[parse]` - Function definitions (short form)
//! - `library[utils]` - Library code (full form)
//! - `lib[utils]` - Library code (short form)
//! - `module[parser]` - Module code (full form)
//! - `mod[parser]` - Module code (short form)
//! - `blob[image_data]` - Binary/opaque data blobs
//! - `raw` - Raw script content
//! - `packed` - Packed/compressed content
//!
//! ### Simple Values
//! - `port`, `debug`, `enabled`, etc. - Plain key-value configuration
//!
//! ### Design Philosophy
//! - **Singular forms** for semantic clarity: `section[...]` not `sections[...]`
//! - **Short aliases** for common types: `func[]`, `lib[]`, `mod[]`
//! - **Type hints only** - export doesn't enforce, plugins use hints for smart behavior
//! - **Extensible** - any bracket pattern works, these are just recognized conventions

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentType {
    DocumentSection,
    ScriptPart,
    Chunk,
    Function,
    Library,
    Module,
    Blob,
    Metadata,
    Canonical,
    SimpleValue,
}

impl ContentType {
    pub fn from_key(key: &str) -> Self {
        if key == "full" || key == "raw" || key == "packed" {
            return ContentType::Canonical;
        }

        if !key.contains('[') {
            return ContentType::SimpleValue;
        }

        if key.starts_with("section[") {
            ContentType::DocumentSection
        } else if key.starts_with("part[") {
            ContentType::ScriptPart
        } else if key.starts_with("chunk[") {
            ContentType::Chunk
        } else if key.starts_with("function[") || key.starts_with("func[") {
            ContentType::Function
        } else if key.starts_with("library[") || key.starts_with("lib[") {
            ContentType::Library
        } else if key.starts_with("module[") || key.starts_with("mod[") {
            ContentType::Module
        } else if key.starts_with("blob[") {
            ContentType::Blob
        } else if key.starts_with("metadata[") {
            ContentType::Metadata
        } else {
            ContentType::SimpleValue
        }
    }

    pub fn is_content_part(&self) -> bool {
        matches!(
            self,
            ContentType::DocumentSection
                | ContentType::ScriptPart
                | ContentType::Chunk
                | ContentType::Function
                | ContentType::Library
                | ContentType::Module
                | ContentType::Blob
        )
    }

    pub fn is_canonical(&self) -> bool {
        matches!(self, ContentType::Canonical)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExportFormat {
    Text,
    Json,
}

#[derive(Debug, Clone)]
pub struct ExportMetadata {
    pub checksum: String,
    pub timestamp: u64,
    pub token_count: usize,
}

impl ExportMetadata {
    pub fn new(checksum: String, token_count: usize) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            checksum,
            timestamp,
            token_count,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExportData {
    pub context: String,
    pub namespace: String,
    pub tokens: Vec<(String, String)>,
    pub metadata: ExportMetadata,
    pub format: ExportFormat,
}

impl ExportData {
    pub fn new(
        context: String,
        namespace: String,
        tokens: Vec<(String, String)>,
        format: ExportFormat,
    ) -> Self {
        let checksum = Self::calculate_checksum(&context, &namespace, &tokens);
        let metadata = ExportMetadata::new(checksum, tokens.len());

        Self {
            context,
            namespace,
            tokens,
            metadata,
            format,
        }
    }

    fn calculate_checksum(context: &str, namespace: &str, tokens: &[(String, String)]) -> String {
        use hub::data_ext::base64::{engine::general_purpose, Engine};
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        context.hash(&mut hasher);
        namespace.hash(&mut hasher);
        for (key, value) in tokens {
            key.hash(&mut hasher);
            value.hash(&mut hasher);
        }
        let hash = hasher.finish();

        general_purpose::STANDARD_NO_PAD.encode(hash.to_le_bytes())
    }

    pub fn to_text(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("# Meteor Export\n"));
        output.push_str(&format!("# Context: {}\n", self.context));
        output.push_str(&format!("# Namespace: {}\n", self.namespace));
        output.push_str(&format!("# Checksum: {}\n", self.metadata.checksum));
        output.push_str(&format!("# Timestamp: {}\n", self.metadata.timestamp));
        output.push_str(&format!("# Token Count: {}\n\n", self.metadata.token_count));

        for (key, value) in &self.tokens {
            output.push_str(&format!("{}={}\n", key, value));
        }

        output
    }

    pub fn to_json(&self) -> Result<String, String> {
        let mut output = String::new();
        output.push_str("{\n");
        output.push_str(&format!("  \"context\": \"{}\",\n", self.context));
        output.push_str(&format!("  \"namespace\": \"{}\",\n", self.namespace));
        output.push_str("  \"tokens\": [\n");

        for (i, (key, value)) in self.tokens.iter().enumerate() {
            output.push_str("    {\n");
            output.push_str(&format!("      \"key\": \"{}\",\n", Self::escape_json(key)));
            output.push_str(&format!(
                "      \"value\": \"{}\"\n",
                Self::escape_json(value)
            ));
            if i < self.tokens.len() - 1 {
                output.push_str("    },\n");
            } else {
                output.push_str("    }\n");
            }
        }

        output.push_str("  ],\n");
        output.push_str("  \"metadata\": {\n");
        output.push_str(&format!(
            "    \"checksum\": \"{}\",\n",
            self.metadata.checksum
        ));
        output.push_str(&format!(
            "    \"timestamp\": {},\n",
            self.metadata.timestamp
        ));
        output.push_str(&format!(
            "    \"token_count\": {}\n",
            self.metadata.token_count
        ));
        output.push_str("  }\n");
        output.push_str("}");

        Ok(output)
    }

    fn escape_json(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }

    pub fn from_text(text: &str) -> Result<Self, String> {
        let mut context = String::new();
        let mut namespace = String::new();
        let mut checksum = String::new();
        let mut timestamp = 0u64;
        let mut tokens = Vec::new();

        for line in text.lines() {
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            if line.starts_with("# Context:") {
                context = line.strip_prefix("# Context:").unwrap().trim().to_string();
            } else if line.starts_with("# Namespace:") {
                namespace = line
                    .strip_prefix("# Namespace:")
                    .unwrap()
                    .trim()
                    .to_string();
            } else if line.starts_with("# Checksum:") {
                checksum = line.strip_prefix("# Checksum:").unwrap().trim().to_string();
            } else if line.starts_with("# Timestamp:") {
                let ts_str = line.strip_prefix("# Timestamp:").unwrap().trim();
                timestamp = ts_str
                    .parse()
                    .map_err(|_| format!("Invalid timestamp: {}", ts_str))?;
            } else if line.starts_with('#') {
                continue;
            } else if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].to_string();
                let value = line[eq_pos + 1..].to_string();
                tokens.push((key, value));
            }
        }

        if context.is_empty() || namespace.is_empty() {
            return Err("Missing context or namespace in export data".to_string());
        }

        let metadata = ExportMetadata {
            checksum,
            timestamp,
            token_count: tokens.len(),
        };

        Ok(Self {
            context,
            namespace,
            tokens,
            metadata,
            format: ExportFormat::Text,
        })
    }

    pub fn content_types(&self) -> Vec<(String, ContentType)> {
        self.tokens
            .iter()
            .map(|(key, _)| (key.clone(), ContentType::from_key(key)))
            .collect()
    }

    pub fn has_canonical(&self) -> bool {
        self.tokens
            .iter()
            .any(|(key, _)| ContentType::from_key(key).is_canonical())
    }

    pub fn has_content_parts(&self) -> bool {
        self.tokens
            .iter()
            .any(|(key, _)| ContentType::from_key(key).is_content_part())
    }

    pub fn get_canonical(&self) -> Option<&str> {
        self.tokens.iter().find_map(|(key, value)| {
            if ContentType::from_key(key).is_canonical() {
                Some(value.as_str())
            } else {
                None
            }
        })
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        let json = json.trim();

        let context = Self::extract_json_string(json, "context")?;
        let namespace = Self::extract_json_string(json, "namespace")?;

        let tokens_start = json.find("\"tokens\"").ok_or("Missing 'tokens' field")?;
        let tokens_section = &json[tokens_start..];
        let array_start = tokens_section.find('[').ok_or("Invalid tokens array")?;
        let array_end = tokens_section.rfind(']').ok_or("Invalid tokens array")?;
        let tokens_array = &tokens_section[array_start + 1..array_end];

        let mut tokens = Vec::new();
        for token_obj in tokens_array.split("},") {
            let token_obj = token_obj.trim();
            if token_obj.is_empty() {
                continue;
            }

            let key = Self::extract_json_string(token_obj, "key")?;
            let value = Self::extract_json_string(token_obj, "value")?;
            tokens.push((key, value));
        }

        let checksum = Self::extract_json_string(json, "checksum")?;
        let timestamp = Self::extract_json_number(json, "timestamp")?;
        let token_count = Self::extract_json_number(json, "token_count")?;

        let metadata = ExportMetadata {
            checksum,
            timestamp,
            token_count: token_count as usize,
        };

        Ok(Self {
            context,
            namespace,
            tokens,
            metadata,
            format: ExportFormat::Json,
        })
    }

    fn extract_json_string(json: &str, field: &str) -> Result<String, String> {
        let pattern = format!("\"{}\": \"", field);
        let start = json
            .find(&pattern)
            .ok_or_else(|| format!("Missing field: {}", field))?;
        let value_start = start + pattern.len();
        let value_section = &json[value_start..];
        let end = value_section
            .find('"')
            .ok_or_else(|| format!("Malformed field: {}", field))?;

        Ok(Self::unescape_json(&value_section[..end]))
    }

    fn extract_json_number(json: &str, field: &str) -> Result<u64, String> {
        let pattern = format!("\"{}\": ", field);
        let start = json
            .find(&pattern)
            .ok_or_else(|| format!("Missing field: {}", field))?;
        let value_start = start + pattern.len();
        let value_section = &json[value_start..];

        let end = value_section
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(value_section.len());

        value_section[..end]
            .parse()
            .map_err(|_| format!("Invalid number for field: {}", field))
    }

    fn unescape_json(s: &str) -> String {
        s.replace("\\n", "\n")
            .replace("\\r", "\r")
            .replace("\\t", "\t")
            .replace("\\\"", "\"")
            .replace("\\\\", "\\")
    }
}

impl fmt::Display for ExportData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.format {
            ExportFormat::Text => write!(f, "{}", self.to_text()),
            ExportFormat::Json => match self.to_json() {
                Ok(json) => write!(f, "{}", json),
                Err(e) => write!(f, "Error: {}", e),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportDiff {
    Added {
        key: String,
        value: String,
    },
    Updated {
        key: String,
        old_value: String,
        new_value: String,
    },
    Unchanged {
        key: String,
    },
}

impl fmt::Display for ImportDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImportDiff::Added { key, value } => write!(f, "+ {}: {}", key, value),
            ImportDiff::Updated {
                key,
                old_value,
                new_value,
            } => write!(f, "~ {}: {} → {}", key, old_value, new_value),
            ImportDiff::Unchanged { key } => write!(f, "  {}", key),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImportResult {
    pub success: bool,
    pub tokens_added: usize,
    pub tokens_updated: usize,
    pub tokens_unchanged: usize,
    pub diff: Vec<ImportDiff>,
    pub checksum_valid: bool,
}

impl ImportResult {
    pub fn new() -> Self {
        Self {
            success: false,
            tokens_added: 0,
            tokens_updated: 0,
            tokens_unchanged: 0,
            diff: Vec::new(),
            checksum_valid: false,
        }
    }

    pub fn total_changes(&self) -> usize {
        self.tokens_added + self.tokens_updated
    }
}

impl Default for ImportResult {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ImportResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Import Result:")?;
        writeln!(f, "  Success: {}", self.success)?;
        writeln!(f, "  Checksum Valid: {}", self.checksum_valid)?;
        writeln!(f, "  Added: {}", self.tokens_added)?;
        writeln!(f, "  Updated: {}", self.tokens_updated)?;
        writeln!(f, "  Unchanged: {}", self.tokens_unchanged)?;
        writeln!(f, "  Total Changes: {}", self.total_changes())?;

        if !self.diff.is_empty() {
            writeln!(f, "\nDiff:")?;
            for change in &self.diff {
                writeln!(f, "  {}", change)?;
            }
        }

        Ok(())
    }
}
