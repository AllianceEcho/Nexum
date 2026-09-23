//! Nexum plugin runtime — manifest, permissions, capabilities, and lifecycle.

pub use nexum_engine::{
    EngineAdapter, EngineCapabilities, EngineError, EngineSnapshot, EngineTask, EngineTaskState,
    map_engine_snapshot,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ——— Plugin lifecycle trait ———————————————————————————————

/// Trait for plugins that support lifecycle operations.
///
/// Implementations should perform their actual load/start/stop logic.
pub trait PluginLifecycle: Send {
    /// Called when the plugin is loaded (e.g. resolving shared library).
    fn load(&mut self) -> Result<(), PluginError>;

    /// Called when the plugin is started.
    fn start(&mut self) -> Result<(), PluginError>;

    /// Called when the plugin is stopped.
    fn stop(&mut self) -> Result<(), PluginError>;
}

// ——— Engine extension trait ——————————————————————————————

/// A trait for plugins that provide engine adapters.
///
/// Engine extensions are a way to add new download engines to the system
/// without modifying the engine crate. Plugins implement this trait and
/// register themselves through the engine's registry.
pub trait EngineProvider: PluginLifecycle + Send + Sized {
    /// Returns the engine's name (e.g., "http", "bitdown").
    fn engine_name(&self) -> &str;

    /// Creates a new engine adapter instance.
    fn make_engine(&self) -> Box<dyn EngineAdapter>;
}

/// A trait for plugins that provide resolvers.
///
/// Resolver extensions are a way to add new input resolvers (magnet, torrent,
/// CDN, etc.) to the system without modifying the resolver crate.
pub trait ResolverProvider: PluginLifecycle + Send + Sized {
    /// Returns the resolver's name (e.g., "torrent", "cdn").
    fn resolver_name(&self) -> &str;

    /// Creates a new resolver instance.
    fn make_resolver(&self) -> Box<dyn nexum_resolver::Resolver>;
}

// ——— Plugin state ———————————————————————————

/// Lifecycle state of a plugin.
///
/// Transitions:
///
/// Pending → Loading → Loaded → Starting → Started → Stopping → Stopped
///                                              ↘ Error
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PluginState {
    /// Plugin manifest loaded but not yet initialised.
    Pending,
    /// Plugin is being loaded (e.g. resolving its entry library).
    Loading,
    /// Plugin has been loaded but not yet started.
    Loaded,
    /// Plugin is starting up.
    Starting,
    /// Plugin is running.
    Started,
    /// Plugin is shutting down.
    Stopping,
    /// Plugin has been stopped.
    Stopped,
    /// Plugin encountered an error.
    Error(String),
}

impl PluginState {
    /// Returns true if the plugin is in a terminal state (Stopped or Error).
    pub fn is_terminal(&self) -> bool {
        matches!(self, PluginState::Stopped | PluginState::Error(_))
    }

    /// Returns true if the plugin can be started.
    pub fn can_start(&self) -> bool {
        matches!(self, PluginState::Loaded)
    }

    /// Returns true if the plugin is running (Started).
    pub fn is_active(&self) -> bool {
        matches!(self, PluginState::Started)
    }
}

// ——— Plugin error —————————————————————————————

/// Errors that can occur during plugin lifecycle operations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PluginError {
    /// Plugin was not found in the manager.
    NotFound(String),
    /// Invalid state transition.
    InvalidTransition { from: PluginState, to: &'static str },
    /// The plugin is already in the target state.
    AlreadyInState(PluginState),
    /// An error from the plugin itself.
    PluginError(String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "plugin not found: {id}"),
            Self::InvalidTransition { from, to } => {
                write!(f, "invalid transition from {from:?} to {to}")
            }
            Self::AlreadyInState(state) => write!(f, "plugin already in state {state:?}"),
            Self::PluginError(msg) => write!(f, "plugin error: {msg}"),
        }
    }
}

impl std::error::Error for PluginError {}

// ——— Plugin ———————————————————————————————————

/// A plugin instance wrapping its manifest and current lifecycle state.
pub struct Plugin {
    manifest: PluginManifest,
    state: PluginState,
}

impl Plugin {
    /// Creates a new pending plugin from a manifest.
    pub fn new(manifest: PluginManifest) -> Self {
        Self {
            state: PluginState::Pending,
            manifest,
        }
    }

    /// Returns the plugin's manifest.
    pub fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    /// Returns the plugin's current state.
    pub fn state(&self) -> PluginState {
        self.state.clone()
    }

    /// Returns the plugin ID.
    pub fn id(&self) -> &str {
        &self.manifest.id
    }

    /// Transitions the plugin state, validating the transition.
    fn transition_to(&mut self, next: PluginState) -> Result<(), PluginError> {
        let current = self.state.clone();
        match next {
            PluginState::Loading if current != PluginState::Pending => {
                return Err(PluginError::InvalidTransition {
                    from: current,
                    to: "Loading",
                });
            }
            PluginState::Loaded if current != PluginState::Loading => {
                return Err(PluginError::InvalidTransition {
                    from: current,
                    to: "Loaded",
                });
            }
            PluginState::Starting if current != PluginState::Loaded => {
                return Err(PluginError::InvalidTransition {
                    from: current,
                    to: "Starting",
                });
            }
            PluginState::Started if current != PluginState::Starting => {
                return Err(PluginError::InvalidTransition {
                    from: current,
                    to: "Started",
                });
            }
            PluginState::Stopping
                if !matches!(current, PluginState::Started | PluginState::Starting) =>
            {
                return Err(PluginError::InvalidTransition {
                    from: current,
                    to: "Stopping",
                });
            }
            PluginState::Stopped if current != PluginState::Stopping => {
                return Err(PluginError::InvalidTransition {
                    from: current,
                    to: "Stopped",
                });
            }
            _ => {}
        }
        self.state = next;
        Ok(())
    }
}

// ——— Plugin manager ——————————————————————————————————

/// Manages the lifecycle of multiple plugins.
pub struct PluginManager {
    plugins: Vec<Plugin>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    /// Creates a new empty plugin manager.
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Registers a plugin manifest, returning its index.
    pub fn register(&mut self, manifest: PluginManifest) -> usize {
        let idx = self.plugins.len();
        let mut plugin = Plugin::new(manifest);
        // Immediately transition to Loading so the caller can invoke lifecycle methods.
        plugin.transition_to(PluginState::Loading).ok();
        self.plugins.push(plugin);
        idx
    }

    /// Returns the number of registered plugins.
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Returns true if no plugins are registered.
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// Returns a plugin by ID, or None if not found.
    pub fn get(&self, id: &str) -> Option<&Plugin> {
        self.plugins.iter().find(|p| p.id() == id)
    }

    /// Returns a mutable reference to a plugin by ID.
    fn get_mut(&mut self, id: &str) -> Option<&mut Plugin> {
        self.plugins.iter_mut().find(|p| p.id() == id)
    }

    /// Transitions a plugin to the Loaded state.
    pub fn loaded(&mut self, id: &str) -> Result<(), PluginError> {
        self.get_mut(id)
            .ok_or_else(|| PluginError::NotFound(id.to_owned()))?
            .transition_to(PluginState::Loaded)
    }

    /// Transitions a plugin through its full startup chain (Pending → Loading → Loaded → Starting → Started).
    pub fn start_plugin(&mut self, id: &str) -> Result<(), PluginError> {
        let plugin = self
            .get_mut(id)
            .ok_or_else(|| PluginError::NotFound(id.to_owned()))?;

        match plugin.state() {
            PluginState::Started => return Ok(()),
            PluginState::Loading | PluginState::Pending => {
                plugin.transition_to(PluginState::Loaded)?;
            }
            PluginState::Loaded => {}
            _ => {
                return Err(PluginError::InvalidTransition {
                    from: plugin.state(),
                    to: "Started",
                });
            }
        }

        // Loaded → Starting → Started
        plugin.transition_to(PluginState::Starting)?;
        plugin.transition_to(PluginState::Started)?;
        Ok(())
    }

    /// Stops a plugin (Started → Stopping → Stopped).
    pub fn stop_plugin(&mut self, id: &str) -> Result<(), PluginError> {
        let plugin = self
            .get_mut(id)
            .ok_or_else(|| PluginError::NotFound(id.to_owned()))?;

        match plugin.state() {
            PluginState::Started | PluginState::Starting => {}
            _ => {
                return Err(PluginError::InvalidTransition {
                    from: plugin.state(),
                    to: "Stopped",
                });
            }
        }

        // Started → Stopping → Stopped
        plugin.transition_to(PluginState::Stopping)?;
        plugin.transition_to(PluginState::Stopped)?;
        Ok(())
    }

    /// Transitions a plugin to an Error state from any point.
    pub fn error(&mut self, id: &str, message: impl Into<String>) -> Result<(), PluginError> {
        let plugin = self
            .get_mut(id)
            .ok_or_else(|| PluginError::NotFound(id.to_owned()))?;
        plugin.transition_to(PluginState::Error(message.into()))
    }

    /// Returns plugin IDs sorted by registration order.
    pub fn ids(&self) -> Vec<&str> {
        self.plugins.iter().map(|p| p.id()).collect()
    }
}

// ——— Default / No-op lifecycle ——————————————————————————————

/// A no-op lifecycle implementation that does nothing in load/start/stop.
pub struct NoOpLifecycle;

impl PluginLifecycle for NoOpLifecycle {
    fn load(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn start(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn stop(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

// ——— Permission (existing) ——————————————————————————————

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
        if self.recursive {
            write!(f, "/**")
        } else {
            Ok(())
        }
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
        Self {
            name: name.into(),
            version: version.into(),
            features: Vec::new(),
        }
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

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
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

// ——— Tests —————————————————————————————————————————————————————

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
        let manifest = PluginManifest::new("id1", "My Plugin", "lib.so").with_version("1.2.3");
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

    // —— PluginState tests ——

    #[test]
    fn new_plugin_is_pending() {
        let manifest = PluginManifest::new("p1", "P", "e.so");
        let plugin = Plugin::new(manifest);
        assert_eq!(plugin.state(), PluginState::Pending);
    }

    #[test]
    fn state_can_start_only_when_loaded() {
        assert!(!PluginState::Pending.can_start());
        assert!(!PluginState::Loading.can_start());
        assert!(PluginState::Loaded.can_start());
        assert!(!PluginState::Starting.can_start());
        assert!(!PluginState::Started.can_start());
        assert!(!PluginState::Stopped.can_start());
        assert!(!PluginState::Error("x".into()).can_start());
    }

    #[test]
    fn is_terminal_stopped_and_error() {
        assert!(!PluginState::Pending.is_terminal());
        assert!(!PluginState::Started.is_terminal());
        assert!(!PluginState::Loading.is_terminal());
        assert!(!PluginState::Loaded.is_terminal());
        assert!(!PluginState::Starting.is_terminal());
        assert!(!PluginState::Stopping.is_terminal());
        assert!(PluginState::Stopped.is_terminal());
        assert!(PluginState::Error("msg".into()).is_terminal());
    }

    #[test]
    fn is_active_only_when_started() {
        assert!(!PluginState::Pending.is_active());
        assert!(!PluginState::Loaded.is_active());
        assert!(PluginState::Started.is_active());
        assert!(!PluginState::Stopped.is_active());
    }

    // —— Plugin transition tests ——

    #[test]
    fn valid_transition_chain_pending_to_stopped() {
        let manifest = PluginManifest::new("p1", "P", "e.so");
        let mut plugin = Plugin::new(manifest);
        assert_eq!(plugin.state(), PluginState::Pending);

        plugin.transition_to(PluginState::Loading).unwrap();
        assert_eq!(plugin.state(), PluginState::Loading);

        plugin.transition_to(PluginState::Loaded).unwrap();
        assert_eq!(plugin.state(), PluginState::Loaded);

        plugin.transition_to(PluginState::Starting).unwrap();
        assert_eq!(plugin.state(), PluginState::Starting);

        plugin.transition_to(PluginState::Started).unwrap();
        assert_eq!(plugin.state(), PluginState::Started);

        plugin.transition_to(PluginState::Stopping).unwrap();
        assert_eq!(plugin.state(), PluginState::Stopping);

        plugin.transition_to(PluginState::Stopped).unwrap();
        assert_eq!(plugin.state(), PluginState::Stopped);
    }

    #[test]
    fn error_from_any_state() {
        for state in [
            PluginState::Pending,
            PluginState::Loading,
            PluginState::Loaded,
            PluginState::Starting,
            PluginState::Started,
            PluginState::Stopping,
        ] {
            let manifest = PluginManifest::new("p1", "P", "e.so");
            let mut plugin = Plugin::new(manifest);
            // Advance to the given state
            if state != PluginState::Pending {
                plugin.transition_to(state).ok();
            }
            plugin
                .transition_to(PluginState::Error("boom".into()))
                .unwrap();
            assert!(matches!(plugin.state(), PluginState::Error(_)));
        }
    }

    #[test]
    fn invalid_transition_rejected() {
        let manifest = PluginManifest::new("p1", "P", "e.so");
        let mut plugin = Plugin::new(manifest);
        // Cannot go directly from Pending to Started
        let result = plugin.transition_to(PluginState::Started);
        assert!(matches!(result, Err(PluginError::InvalidTransition { .. })));
    }

    #[test]
    fn full_chain_from_pending() {
        let manifest = PluginManifest::new("p1", "P", "e.so");
        let mut plugin = Plugin::new(manifest);
        // Go through the full chain from Pending
        plugin.transition_to(PluginState::Loading).unwrap();
        plugin.transition_to(PluginState::Loaded).unwrap();
        plugin.transition_to(PluginState::Starting).unwrap();
        plugin.transition_to(PluginState::Started).unwrap();
        assert_eq!(plugin.state(), PluginState::Started);
    }

    // —— PluginManager tests ——

    #[test]
    fn manager_starts_empty() {
        let manager = PluginManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
        assert!(manager.ids().is_empty());
    }

    #[test]
    fn register_returns_index() {
        let mut manager = PluginManager::new();
        let m1 = PluginManifest::new("a", "A", "a.so");
        let m2 = PluginManifest::new("b", "B", "b.so");
        assert_eq!(manager.register(m1), 0);
        assert_eq!(manager.register(m2), 1);
        assert_eq!(manager.len(), 2);
    }

    #[test]
    fn register_goes_to_loading_state() {
        let mut manager = PluginManager::new();
        manager.register(PluginManifest::new("p1", "P", "p.so"));
        assert_eq!(manager.get("p1").unwrap().state(), PluginState::Loading);
    }

    #[test]
    fn loaded_transitions_from_loading() {
        let mut manager = PluginManager::new();
        manager.register(PluginManifest::new("p1", "P", "p.so"));
        manager.loaded("p1").unwrap();
        assert_eq!(manager.get("p1").unwrap().state(), PluginState::Loaded);
    }

    #[test]
    fn start_plugin_full_chain() {
        let mut manager = PluginManager::new();
        manager.register(PluginManifest::new("p1", "P", "p.so"));
        // Register puts it in Loading; loaded() puts it in Loaded;
        // start_plugin should handle Loading → Loaded → Starting → Started
        manager.start_plugin("p1").unwrap();
        assert_eq!(manager.get("p1").unwrap().state(), PluginState::Started);
    }

    #[test]
    fn start_plugin_skip_loaded() {
        let mut manager = PluginManager::new();
        let m = PluginManifest::new("p1", "P", "p.so");
        let _idx = manager.register(m);
        // Transition to Loaded manually, then start
        manager.loaded("p1").unwrap();
        manager.start_plugin("p1").unwrap();
        assert_eq!(manager.get("p1").unwrap().state(), PluginState::Started);
    }

    #[test]
    fn stop_plugin_chain() {
        let mut manager = PluginManager::new();
        manager.register(PluginManifest::new("p1", "P", "p.so"));
        manager.start_plugin("p1").unwrap();
        assert_eq!(manager.get("p1").unwrap().state(), PluginState::Started);

        manager.stop_plugin("p1").unwrap();
        assert_eq!(manager.get("p1").unwrap().state(), PluginState::Stopped);
    }

    #[test]
    fn stop_from_unstarted_state_fails() {
        let mut manager = PluginManager::new();
        manager.register(PluginManifest::new("p1", "P", "p.so"));
        // Plugin is still in Loading, cannot stop
        let result = manager.stop_plugin("p1");
        assert!(matches!(result, Err(PluginError::InvalidTransition { .. })));
    }

    #[test]
    fn error_on_missing_plugin() {
        let mut manager = PluginManager::new();
        let result = manager.error("missing", "boom");
        assert!(matches!(result, Err(PluginError::NotFound(_))));
    }

    #[test]
    fn error_from_any_state_to_error() {
        let mut manager = PluginManager::new();
        manager.register(PluginManifest::new("p1", "P", "p.so"));
        manager.error("p1", "something went wrong").unwrap();
        assert!(matches!(
            manager.get("p1").unwrap().state(),
            PluginState::Error(_)
        ));
    }

    #[test]
    fn ids_returns_registered_order() {
        let mut manager = PluginManager::new();
        manager.register(PluginManifest::new("c", "C", "c.so"));
        manager.register(PluginManifest::new("a", "A", "a.so"));
        manager.register(PluginManifest::new("b", "B", "b.so"));
        assert_eq!(manager.ids(), vec!["c", "a", "b"]);
    }

    #[test]
    fn no_op_lifecycle_does_nothing() {
        let mut lifecycle = NoOpLifecycle;
        assert!(lifecycle.load().is_ok());
        assert!(lifecycle.start().is_ok());
        assert!(lifecycle.stop().is_ok());
    }

    #[test]
    fn plugin_get_by_id() {
        let mut manager = PluginManager::new();
        manager.register(PluginManifest::new("p1", "P", "p.so"));
        assert_eq!(manager.get("p1").unwrap().id(), "p1");
        assert!(manager.get("nope").is_none());
    }
}
