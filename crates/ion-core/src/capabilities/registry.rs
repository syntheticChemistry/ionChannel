//! Runtime capability registry
//!
//! This module provides dynamic capability registration and discovery,
//! allowing AI agents, plugins, and applications to register custom
//! capabilities at runtime without recompilation.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

/// A capability identifier (namespace.domain.action)
///
/// # Examples
/// ```
/// # use ion_core::capabilities::Capability;
/// let cap = Capability::new("ion", "input", "keyboard");
/// assert_eq!(cap.to_string(), "ion.input.keyboard");
///
/// let parsed = Capability::parse("ai.vision.analyze").unwrap();
/// assert_eq!(parsed.namespace, "ai");
/// ```
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Capability {
    /// Namespace (e.g., "ion", "ai", "plugin", "agent:id")
    pub namespace: String,

    /// Domain (e.g., "input", "capture", "vision", "automation")
    pub domain: String,

    /// Specific action (e.g., "keyboard", "screen", "analyze")
    pub action: String,

    /// Optional version for evolution
    pub version: Option<String>,
}

impl Capability {
    /// Create a new capability
    #[must_use]
    pub fn new(
        namespace: impl Into<String>,
        domain: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        Self {
            namespace: namespace.into(),
            domain: domain.into(),
            action: action.into(),
            version: None,
        }
    }

    /// Parse from string: "namespace.domain.action[@version]"
    ///
    /// # Errors
    /// Returns error if format is invalid
    pub fn parse(s: &str) -> Result<Self, CapabilityError> {
        let parts: Vec<&str> = s.split('@').collect();
        let cap_str = parts[0];
        let version = parts.get(1).map(ToString::to_string);

        let parts: Vec<&str> = cap_str.split('.').collect();
        if parts.len() != 3 {
            return Err(CapabilityError::InvalidFormat(s.to_string()));
        }

        Ok(Self {
            namespace: parts[0].to_string(),
            domain: parts[1].to_string(),
            action: parts[2].to_string(),
            version,
        })
    }

    /// Check if matches pattern (glob-style)
    ///
    /// # Examples
    /// ```
    /// # use ion_core::capabilities::Capability;
    /// let cap = Capability::parse("ion.input.keyboard").unwrap();
    /// assert!(cap.matches("ion.input.*"));
    /// assert!(cap.matches("ion.*.*"));
    /// assert!(!cap.matches("ai.*.*"));
    /// ```
    #[must_use]
    pub fn matches(&self, pattern: &str) -> bool {
        let Ok(pattern_cap) = Self::parse(pattern) else {
            return false;
        };

        let ns_match = pattern_cap.namespace == "*" || pattern_cap.namespace == self.namespace;
        let dom_match = pattern_cap.domain == "*" || pattern_cap.domain == self.domain;
        let act_match = pattern_cap.action == "*" || pattern_cap.action == self.action;

        ns_match && dom_match && act_match
    }
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref v) = self.version {
            write!(
                f,
                "{}.{}.{}@{}",
                self.namespace, self.domain, self.action, v
            )
        } else {
            write!(f, "{}.{}.{}", self.namespace, self.domain, self.action)
        }
    }
}

/// Runtime definition of a capability with metadata
#[derive(Debug, Clone)]
pub struct CapabilityDefinition {
    /// The capability identifier
    pub capability: Capability,

    /// Human-readable name
    pub display_name: String,

    /// Description for users
    pub description: String,

    /// Risk level for UI warnings
    pub risk_level: RiskLevel,

    /// Who defined this capability
    pub provider: CapabilityProvider,

    /// When it was registered
    pub registered_at: SystemTime,

    /// Deprecation info (if any)
    pub deprecated: Option<DeprecationInfo>,

    /// Required capabilities (dependencies)
    pub requires: Vec<Capability>,
}

/// Danger level of a capability
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    /// Read-only, safe operations
    Minimal,
    /// Normal operations
    Low,
    /// Sensitive but controlled
    Medium,
    /// Requires extra confirmation
    High,
    /// Extremely dangerous
    Critical,
}

/// Who provided this capability
#[derive(Debug, Clone)]
pub enum CapabilityProvider {
    /// Built into ionChannel
    Core,
    /// Loaded plugin
    Plugin {
        /// Plugin name
        name: String
    },
    /// AI agent
    Agent {
        /// Agent identifier
        id: String
    },
    /// Application-defined
    Application {
        /// Application identifier
        app_id: String
    },
}

/// Deprecation information
#[derive(Debug, Clone)]
pub struct DeprecationInfo {
    /// Version when deprecated
    pub since: String,
    /// Reason for deprecation
    pub reason: String,
    /// Replacement capability
    pub replacement: Option<Capability>,
    /// Version when it will be removed
    pub removal_in: Option<String>,
}

/// Events emitted by the registry
#[derive(Debug, Clone)]
pub enum CapabilityEvent {
    /// New capability registered
    Registered(CapabilityDefinition),
    /// Capability removed
    Unregistered(CapabilityDefinition),
    /// Capability deprecated
    Deprecated(Capability, DeprecationInfo),
}

/// Global registry of available capabilities
///
/// This is the central registry for all runtime-registered capabilities.
/// It supports concurrent access and provides fast lookup by various keys.
pub struct CapabilityRegistry {
    /// All registered capabilities
    definitions: Arc<RwLock<HashMap<Capability, CapabilityDefinition>>>,

    /// Index by namespace for fast lookup
    by_namespace: Arc<RwLock<HashMap<String, Vec<Capability>>>>,

    /// Index by domain for fast lookup
    by_domain: Arc<RwLock<HashMap<String, Vec<Capability>>>>,

    /// Event channel for notifications
    events: tokio::sync::broadcast::Sender<CapabilityEvent>,
}

impl CapabilityRegistry {
    /// Create a new registry
    #[must_use]
    pub fn new() -> Self {
        let (tx, _rx) = tokio::sync::broadcast::channel(100);

        Self {
            definitions: Arc::new(RwLock::new(HashMap::new())),
            by_namespace: Arc::new(RwLock::new(HashMap::new())),
            by_domain: Arc::new(RwLock::new(HashMap::new())),
            events: tx,
        }
    }

    /// Register a new capability
    ///
    /// # Errors
    /// Returns error if capability already exists
    pub async fn register(&self, definition: CapabilityDefinition) -> Result<(), CapabilityError> {
        let cap = definition.capability.clone();

        // Store definition
        let mut defs = self.definitions.write().await;
        if defs.contains_key(&cap) {
            return Err(CapabilityError::AlreadyExists(cap.to_string()));
        }
        defs.insert(cap.clone(), definition.clone());
        drop(defs);

        // Update indexes
        let mut by_ns = self.by_namespace.write().await;
        by_ns
            .entry(cap.namespace.clone())
            .or_default()
            .push(cap.clone());
        drop(by_ns);

        let mut by_dom = self.by_domain.write().await;
        by_dom
            .entry(cap.domain.clone())
            .or_default()
            .push(cap.clone());
        drop(by_dom);

        // Notify listeners
        let _ = self.events.send(CapabilityEvent::Registered(definition));

        Ok(())
    }

    /// Find capabilities matching pattern
    ///
    /// # Examples
    /// ```
    /// # use ion_core::capabilities::CapabilityRegistry;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let registry = CapabilityRegistry::new();
    /// // ... register some capabilities
    ///
    /// // Find all AI capabilities
    /// let ai_caps = registry.find_matching("ai.*.*").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_matching(&self, pattern: &str) -> Result<Vec<Capability>, CapabilityError> {
        let defs = self.definitions.read().await;

        Ok(defs
            .keys()
            .filter(|cap| cap.matches(pattern))
            .cloned()
            .collect())
    }

    /// Get definition for capability
    ///
    /// # Errors
    /// Returns error if capability not found
    pub async fn get(&self, cap: &Capability) -> Result<CapabilityDefinition, CapabilityError> {
        let defs = self.definitions.read().await;
        defs.get(cap)
            .cloned()
            .ok_or_else(|| CapabilityError::NotFound(cap.to_string()))
    }

    /// List all capabilities in namespace
    #[must_use]
    pub async fn list_namespace(&self, namespace: &str) -> Vec<Capability> {
        let by_ns = self.by_namespace.read().await;
        by_ns.get(namespace).cloned().unwrap_or_default()
    }

    /// List all capabilities in domain
    #[must_use]
    pub async fn list_domain(&self, domain: &str) -> Vec<Capability> {
        let by_dom = self.by_domain.read().await;
        by_dom.get(domain).cloned().unwrap_or_default()
    }

    /// Unregister capability (for cleanup)
    ///
    /// # Errors
    /// Returns error if capability not found
    pub async fn unregister(&self, cap: &Capability) -> Result<(), CapabilityError> {
        let mut defs = self.definitions.write().await;
        let def = defs
            .remove(cap)
            .ok_or_else(|| CapabilityError::NotFound(cap.to_string()))?;
        drop(defs);

        // Update indexes
        let mut by_ns = self.by_namespace.write().await;
        if let Some(caps) = by_ns.get_mut(&cap.namespace) {
            caps.retain(|c| c != cap);
        }
        drop(by_ns);

        let mut by_dom = self.by_domain.write().await;
        if let Some(caps) = by_dom.get_mut(&cap.domain) {
            caps.retain(|c| c != cap);
        }
        drop(by_dom);

        // Notify listeners
        let _ = self.events.send(CapabilityEvent::Unregistered(def));

        Ok(())
    }

    /// Subscribe to capability events
    #[must_use]
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<CapabilityEvent> {
        self.events.subscribe()
    }

    /// Count total registered capabilities
    #[must_use]
    pub async fn count(&self) -> usize {
        let defs = self.definitions.read().await;
        defs.len()
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Capability errors
#[derive(Debug, thiserror::Error)]
pub enum CapabilityError {
    /// Invalid capability format
    #[error("Invalid capability format: {0}")]
    InvalidFormat(String),

    /// Capability already exists
    #[error("Capability already exists: {0}")]
    AlreadyExists(String),

    /// Capability not found
    #[error("Capability not found: {0}")]
    NotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_parse() {
        let cap = Capability::parse("ion.input.keyboard").unwrap();
        assert_eq!(cap.namespace, "ion");
        assert_eq!(cap.domain, "input");
        assert_eq!(cap.action, "keyboard");
        assert!(cap.version.is_none());
    }

    #[test]
    fn test_capability_parse_with_version() {
        let cap = Capability::parse("ion.input.keyboard@v2").unwrap();
        assert_eq!(cap.version, Some("v2".to_string()));
    }

    #[test]
    fn test_capability_parse_invalid() {
        assert!(Capability::parse("invalid").is_err());
        assert!(Capability::parse("too.many.parts.here").is_err());
    }

    #[test]
    fn test_capability_to_string() {
        let cap = Capability::new("ion", "input", "keyboard");
        assert_eq!(cap.to_string(), "ion.input.keyboard");
    }

    #[test]
    fn test_capability_matches() {
        let cap = Capability::parse("ion.input.keyboard").unwrap();

        assert!(cap.matches("ion.input.keyboard"));
        assert!(cap.matches("ion.input.*"));
        assert!(cap.matches("ion.*.*"));
        assert!(cap.matches("*.*.*"));

        assert!(!cap.matches("ai.input.keyboard"));
        assert!(!cap.matches("ion.capture.*"));
    }

    #[tokio::test]
    async fn test_registry_register() {
        let registry = CapabilityRegistry::new();

        let def = CapabilityDefinition {
            capability: Capability::parse("test.example.action").unwrap(),
            display_name: "Test Action".to_string(),
            description: "A test capability".to_string(),
            risk_level: RiskLevel::Low,
            provider: CapabilityProvider::Core,
            registered_at: SystemTime::now(),
            deprecated: None,
            requires: vec![],
        };

        registry.register(def).await.unwrap();
        assert_eq!(registry.count().await, 1);
    }

    #[tokio::test]
    async fn test_registry_duplicate() {
        let registry = CapabilityRegistry::new();

        let def = CapabilityDefinition {
            capability: Capability::parse("test.example.action").unwrap(),
            display_name: "Test".to_string(),
            description: "Test".to_string(),
            risk_level: RiskLevel::Low,
            provider: CapabilityProvider::Core,
            registered_at: SystemTime::now(),
            deprecated: None,
            requires: vec![],
        };

        registry.register(def.clone()).await.unwrap();

        // Registering again should fail
        let result = registry.register(def).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_registry_find_matching() {
        let registry = CapabilityRegistry::new();

        // Register multiple capabilities
        for (ns, dom, act) in [
            ("ion", "input", "keyboard"),
            ("ion", "input", "mouse"),
            ("ion", "capture", "screen"),
            ("ai", "vision", "analyze"),
        ] {
            let def = CapabilityDefinition {
                capability: Capability::new(ns, dom, act),
                display_name: format!("{} {}", dom, act),
                description: "Test".to_string(),
                risk_level: RiskLevel::Low,
                provider: CapabilityProvider::Core,
                registered_at: SystemTime::now(),
                deprecated: None,
                requires: vec![],
            };
            registry.register(def).await.unwrap();
        }

        // Find all ion.input capabilities
        let ion_input = registry.find_matching("ion.input.*").await.unwrap();
        assert_eq!(ion_input.len(), 2);

        // Find all ai capabilities
        let ai_all = registry.find_matching("ai.*.*").await.unwrap();
        assert_eq!(ai_all.len(), 1);
    }

    #[tokio::test]
    async fn test_registry_unregister() {
        let registry = CapabilityRegistry::new();

        let cap = Capability::parse("test.example.action").unwrap();
        let def = CapabilityDefinition {
            capability: cap.clone(),
            display_name: "Test".to_string(),
            description: "Test".to_string(),
            risk_level: RiskLevel::Low,
            provider: CapabilityProvider::Core,
            registered_at: SystemTime::now(),
            deprecated: None,
            requires: vec![],
        };

        registry.register(def).await.unwrap();
        assert_eq!(registry.count().await, 1);

        registry.unregister(&cap).await.unwrap();
        assert_eq!(registry.count().await, 0);
    }

    #[tokio::test]
    async fn test_registry_events() {
        let registry = CapabilityRegistry::new();
        let mut rx = registry.subscribe();

        let def = CapabilityDefinition {
            capability: Capability::parse("test.example.action").unwrap(),
            display_name: "Test".to_string(),
            description: "Test".to_string(),
            risk_level: RiskLevel::Low,
            provider: CapabilityProvider::Core,
            registered_at: SystemTime::now(),
            deprecated: None,
            requires: vec![],
        };

        registry.register(def).await.unwrap();

        // Should receive registered event
        let event = rx.recv().await.unwrap();
        assert!(matches!(event, CapabilityEvent::Registered(_)));
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Minimal < RiskLevel::Low);
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
        assert!(RiskLevel::High < RiskLevel::Critical);
    }
}
