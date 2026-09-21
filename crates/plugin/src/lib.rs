//! Nexum plugin runtime — manifest, permissions, and capabilities.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Permission granted to a plugin.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum Permission {
    None,
    Read(PathPattern),
    Write(PathPattern),
    Network(String),
    Execute,
}

/// A path pattern that a plugin is allowed to access.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct PathPattern {
    /// The base path to restrict to (e.g., "/home/user/downloads").
    pub path: PathBuf,
    /// If true, allow subdirectories recursively.
    pub recursive: bool,
}

impl std::fmt::Display for PathPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.display())?;
        if self.recursive { write!(f, "/**") } else { Ok(()) }
    }
}

/// A capability a plugin declares it provides.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Capability {
    pub name: String,
    pub version: String,
    pub features: Vec<String>,
}

impl Capability {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self { name: name.into(), version: version.into(), features: Vec::new() }
    }
}

/// Plugin manifest — metadata about a plugin.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub version_min: Option<String>,
    pub version_max: Option<String>,
    pub entry: String,
    pub permissions: Vec<Permission>,
    pub capabilities: Vec<Capability>,
}

impl PluginManifest {
    pub fn new(id: impl Into<String>, name: impl Into<String>, entry: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: "0.0.0".to_owned(),
            description: None,
            author: None,
            license: None,
            version_min: None,
            version_max: None,
            entry: entry.into(),
            permissions: Vec::new(),
            capabilities: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    pub fn with_permissions(mut self, permissions: Vec<Permission>) -> Self {
        self.permissions = permissions;
        self
    }

    pub fn with_capabilities(mut self, capabilities: Vec<Capability>) -> Self {
        self.capabilities = capabilities;
        self
    }
}

impl std::fmt::Display for PluginManifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} v{} ({})", self.name, self.version, self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_minimal_manifest() {
        let manifest = PluginManifest::new("test-plugin", "Test Plugin", "libtest.so");
        assert_eq!(manifest.id, "test-plugin");
        assert_eq!(manifest.name, "Test Plugin");
        assert_eq!(manifest.entry, "libtest.so");
        assert!(manifest.permissions.is_empty());
        assert!(manifest.capabilities.is_empty());
    }

    #[test]
    fn manifest_display_includes_name_version_id() {
        let manifest = PluginManifest::new("id1", "My Plugin", "lib.so")
            .with_version("1.2.3");
        let display = format!("{}", manifest);
        assert!(display.contains("My Plugin"));
        assert!(display.contains("1.2.3"));
        assert!(display.contains("id1"));
    }

    #[test]
    fn path_pattern_display() {
        let pattern = PathPattern {
            path: PathBuf::from("/data"),
            recursive: true,
        };
        assert_eq!(format!("{}", pattern), "/data/**");

        let non_recursive = PathPattern {
            path: PathBuf::from("/data"),
            recursive: false,
        };
        assert_eq!(format!("{}", non_recursive), "/data");
    }

    #[test]
    fn capability_serializes() {
        let cap = Capability::new("download", "1.0");
        let json = serde_json::to_string(&cap).unwrap();
        assert!(json.contains("\"name\":\"download\""));
        assert!(json.contains("\"version\":\"1.0\""));
    }

    #[test]
    fn manifest_serializes() {
        let manifest = PluginManifest::new("p1", "P", "e.so");
        let json = serde_json::to_string(&manifest).unwrap();
        assert!(json.contains("\"id\":\"p1\""));
        assert!(json.contains("\"entry\":\"e.so\""));
    }
}
