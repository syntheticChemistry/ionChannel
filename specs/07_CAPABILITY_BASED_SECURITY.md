# Subsystem 07: Dynamic Capability Architecture (Runtime-Extensible)

```yaml
subsystem: dynamic-capability-architecture
status: design-phase
priority: P0-ARCHITECTURE
philosophy: runtime-extensibility-for-ai-era
evolution: v2.0 - unlimited capabilities
```

## Executive Summary

A **runtime-extensible capability system** that supports unlimited, dynamically-registered capabilities for the AI agent era. Instead of hardcoded bitflags (limited to 64), this architecture allows new capabilities to be registered, discovered, and used at runtime without recompilation.

### The Problem with Bitflags (v1.0)

```rust
// Limited to 64 capabilities - what happens in year 5?
bitflags! {
    RemoteDesktopCapabilities: u64 {
        CAP_INJECT_KEYBOARD = 1 << 0;
        // ... 62 more
        // Out of bits! 😱
    }
}
```

**Limitations:**
- ❌ Fixed at 64 capabilities
- ❌ Requires recompilation to add new ones
- ❌ AI agents can't define custom capabilities
- ❌ No namespace support
- ❌ Not forward compatible

### The Runtime-Extensible Solution (v2.0)

```rust
// Unlimited capabilities via string identifiers
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Capability {
    namespace: String,  // "ion", "ai", "custom"
    domain: String,     // "input", "capture", "ai.vision"
    action: String,     // "keyboard", "screen", "analyze"
}

// Example: "ion.input.keyboard"
// Example: "ai.vision.scene_understanding"
// Example: "custom.automation.window_management"
```

**Benefits:**
- ✅ Unlimited capabilities
- ✅ Runtime registration
- ✅ Namespaced (avoid conflicts)
- ✅ AI agents can define new capabilities
- ✅ Forward compatible
- ✅ Discoverable at runtime

---

## Architecture Overview

### Three-Tier Capability System

```
┌─────────────────────────────────────────────────────────┐
│                    TIER 1: CORE                         │
│         (Built-in, optimized, well-known)               │
│  ion.input.keyboard, ion.capture.screen, etc.          │
│  - Hardcoded for performance                            │
│  - Bitflags for fast checks (backward compat)           │
│  - ~64 most common capabilities                         │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                 TIER 2: EXTENDED                        │
│        (Runtime-registered, namespaced)                 │
│  ai.vision.*, automation.*, custom.*                    │
│  - Registered at runtime by plugins                     │
│  - String-based checks                                  │
│  - Unlimited expansion                                  │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                 TIER 3: DYNAMIC                         │
│     (Agent-defined, per-session, temporary)             │
│  agent:abc123.custom.*, session:xyz.temp.*             │
│  - Created by AI agents on-demand                       │
│  - Scoped to session or agent                           │
│  - Automatic cleanup                                    │
└─────────────────────────────────────────────────────────┘
```

---

## Core Data Structures

### Capability Identity

```rust
/// A capability identifier with namespace support
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Capability {
    /// Namespace (e.g., "ion", "ai", "custom", "agent:id")
    pub namespace: String,
    
    /// Domain (e.g., "input", "capture", "vision", "automation")
    pub domain: String,
    
    /// Specific action (e.g., "keyboard", "screen", "analyze")
    pub action: String,
    
    /// Optional version (for evolution)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl Capability {
    /// Parse from string: "namespace.domain.action[@version]"
    pub fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('@').collect();
        let cap_str = parts[0];
        let version = parts.get(1).map(|v| v.to_string());
        
        let parts: Vec<&str> = cap_str.split('.').collect();
        if parts.len() != 3 {
            return Err(Error::InvalidCapability(s.to_string()));
        }
        
        Ok(Capability {
            namespace: parts[0].to_string(),
            domain: parts[1].to_string(),
            action: parts[2].to_string(),
            version,
        })
    }
    
    /// Format as string
    pub fn to_string(&self) -> String {
        match &self.version {
            Some(v) => format!("{}.{}.{}@{}", self.namespace, self.domain, self.action, v),
            None => format!("{}.{}.{}", self.namespace, self.domain, self.action),
        }
    }
    
    /// Check if matches pattern (glob-style)
    /// Example: "ion.input.*" matches "ion.input.keyboard"
    pub fn matches(&self, pattern: &str) -> bool {
        let pattern_cap = match Capability::from_str(pattern) {
            Ok(c) => c,
            Err(_) => return false,
        };
        
        let ns_match = pattern_cap.namespace == "*" || pattern_cap.namespace == self.namespace;
        let dom_match = pattern_cap.domain == "*" || pattern_cap.domain == self.domain;
        let act_match = pattern_cap.action == "*" || pattern_cap.action == self.action;
        
        ns_match && dom_match && act_match
    }
}

/// Convenience type for multiple capabilities
pub type CapabilitySet = HashSet<Capability>;

/// Helper for building capability sets
pub struct CapabilitySetBuilder {
    caps: CapabilitySet,
}

impl CapabilitySetBuilder {
    pub fn new() -> Self {
        Self { caps: HashSet::new() }
    }
    
    pub fn add(mut self, cap: impl Into<Capability>) -> Self {
        self.caps.insert(cap.into());
        self
    }
    
    pub fn add_pattern(mut self, pattern: &str) -> Result<Self> {
        // Add all registered capabilities matching pattern
        let registry = CapabilityRegistry::global();
        for cap in registry.find_matching(pattern)? {
            self.caps.insert(cap);
        }
        Ok(self)
    }
    
    pub fn build(self) -> CapabilitySet {
        self.caps
    }
}
```

### Capability Definition (Metadata)

```rust
/// Runtime definition of a capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDefinition {
    /// The capability identifier
    pub capability: Capability,
    
    /// Human-readable name
    pub display_name: String,
    
    /// Description for users
    pub description: String,
    
    /// Danger level (for UI warning)
    pub risk_level: RiskLevel,
    
    /// Who defined this capability
    pub provider: CapabilityProvider,
    
    /// When it was registered
    pub registered_at: SystemTime,
    
    /// Deprecation info (if any)
    pub deprecated: Option<DeprecationInfo>,
    
    /// Related capabilities (for grouping)
    pub related: Vec<Capability>,
    
    /// Required capabilities (dependencies)
    pub requires: Vec<Capability>,
    
    /// Custom metadata (extensible)
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RiskLevel {
    Minimal,      // Read-only, safe
    Low,          // Normal operations
    Medium,       // Sensitive but controlled
    High,         // Requires extra confirmation
    Critical,     // Extremely dangerous
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityProvider {
    Core,                          // Built into ionChannel
    Plugin { name: String },       // Loaded plugin
    Agent { id: String },          // AI agent
    Application { app_id: String }, // Application-defined
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecationInfo {
    pub since: String,  // Version
    pub reason: String,
    pub replacement: Option<Capability>,
    pub removal_in: Option<String>,  // Version
}
```

### Capability Registry (Runtime)

```rust
/// Global registry of all available capabilities
pub struct CapabilityRegistry {
    /// All registered capabilities
    definitions: Arc<RwLock<HashMap<Capability, CapabilityDefinition>>>,
    
    /// Indexes for fast lookup
    by_namespace: Arc<RwLock<HashMap<String, Vec<Capability>>>>,
    by_domain: Arc<RwLock<HashMap<String, Vec<Capability>>>>,
    
    /// Core capabilities (bitflags for fast path)
    core_capabilities: CoreCapabilities,
    
    /// Event bus for capability registration/changes
    events: broadcast::Sender<CapabilityEvent>,
}

impl CapabilityRegistry {
    /// Get global singleton
    pub fn global() -> &'static Self {
        static REGISTRY: OnceCell<CapabilityRegistry> = OnceCell::new();
        REGISTRY.get_or_init(|| Self::new())
    }
    
    /// Register a new capability
    pub async fn register(&self, definition: CapabilityDefinition) -> Result<()> {
        let cap = definition.capability.clone();
        
        // Store definition
        let mut defs = self.definitions.write().await;
        if defs.contains_key(&cap) {
            return Err(Error::CapabilityAlreadyExists(cap.to_string()));
        }
        defs.insert(cap.clone(), definition.clone());
        
        // Update indexes
        let mut by_ns = self.by_namespace.write().await;
        by_ns.entry(cap.namespace.clone())
            .or_default()
            .push(cap.clone());
        
        let mut by_dom = self.by_domain.write().await;
        by_dom.entry(cap.domain.clone())
            .or_default()
            .push(cap.clone());
        
        // Notify listeners
        let _ = self.events.send(CapabilityEvent::Registered(definition));
        
        info!("Registered capability: {}", cap.to_string());
        Ok(())
    }
    
    /// Find capabilities matching pattern
    pub async fn find_matching(&self, pattern: &str) -> Result<Vec<Capability>> {
        let defs = self.definitions.read().await;
        let pattern_cap = Capability::from_str(pattern)?;
        
        Ok(defs.keys()
            .filter(|cap| cap.matches(pattern))
            .cloned()
            .collect())
    }
    
    /// Get definition for capability
    pub async fn get(&self, cap: &Capability) -> Result<CapabilityDefinition> {
        let defs = self.definitions.read().await;
        defs.get(cap)
            .cloned()
            .ok_or_else(|| Error::UnknownCapability(cap.to_string()))
    }
    
    /// List all capabilities in namespace
    pub async fn list_namespace(&self, namespace: &str) -> Vec<Capability> {
        let by_ns = self.by_namespace.read().await;
        by_ns.get(namespace)
            .cloned()
            .unwrap_or_default()
    }
    
    /// List all capabilities in domain
    pub async fn list_domain(&self, domain: &str) -> Vec<Capability> {
        let by_dom = self.by_domain.read().await;
        by_dom.get(domain)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Unregister capability (for cleanup)
    pub async fn unregister(&self, cap: &Capability) -> Result<()> {
        let mut defs = self.definitions.write().await;
        let def = defs.remove(cap)
            .ok_or_else(|| Error::UnknownCapability(cap.to_string()))?;
        
        // Update indexes
        let mut by_ns = self.by_namespace.write().await;
        if let Some(caps) = by_ns.get_mut(&cap.namespace) {
            caps.retain(|c| c != cap);
        }
        
        let mut by_dom = self.by_domain.write().await;
        if let Some(caps) = by_dom.get_mut(&cap.domain) {
            caps.retain(|c| c != cap);
        }
        
        // Notify listeners
        let _ = self.events.send(CapabilityEvent::Unregistered(def));
        
        Ok(())
    }
    
    /// Subscribe to capability events
    pub fn subscribe(&self) -> broadcast::Receiver<CapabilityEvent> {
        self.events.subscribe()
    }
}

#[derive(Debug, Clone)]
pub enum CapabilityEvent {
    Registered(CapabilityDefinition),
    Unregistered(CapabilityDefinition),
    Deprecated(Capability, DeprecationInfo),
}
```

---

## Core Capabilities (Fast Path)

For performance, maintain a bitflags version of the most common capabilities:

```rust
bitflags! {
    /// Core capabilities (fast path via bitflags)
    /// These are the 64 most common capabilities
    pub struct CoreCapabilities: u64 {
        // Input (bits 0-15)
        const INPUT_KEYBOARD    = 1 << 0;
        const INPUT_POINTER     = 1 << 1;
        const INPUT_TOUCH       = 1 << 2;
        
        // Capture (bits 16-31)
        const CAPTURE_SCREEN    = 1 << 16;
        const CAPTURE_WINDOW    = 1 << 17;
        const CAPTURE_REGION    = 1 << 18;
        
        // ... 58 more common capabilities
    }
}

impl CoreCapabilities {
    /// Map to/from string capability
    pub fn from_capability(cap: &Capability) -> Option<Self> {
        match cap.to_string().as_str() {
            "ion.input.keyboard" => Some(Self::INPUT_KEYBOARD),
            "ion.input.pointer" => Some(Self::INPUT_POINTER),
            "ion.capture.screen" => Some(Self::CAPTURE_SCREEN),
            // ... mapping for all 64 core capabilities
            _ => None,
        }
    }
    
    pub fn to_capability(self) -> Vec<Capability> {
        let mut caps = Vec::new();
        
        if self.contains(Self::INPUT_KEYBOARD) {
            caps.push(Capability::from_str("ion.input.keyboard").unwrap());
        }
        if self.contains(Self::INPUT_POINTER) {
            caps.push(Capability::from_str("ion.input.pointer").unwrap());
        }
        // ... for all 64
        
        caps
    }
}
```

---

## Capability Grant (Updated)

```rust
/// Grant of capabilities to an application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityGrant {
    pub id: Uuid,
    pub app_id: String,
    
    /// Granted capabilities (unlimited)
    pub capabilities: CapabilitySet,
    
    /// Core capabilities (fast path, if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub core_caps: Option<CoreCapabilities>,
    
    pub granted_at: SystemTime,
    pub expires_at: Option<SystemTime>,
    pub reason: Option<String>,
    pub restrictions: CapabilityRestrictions,
    pub usage: GrantUsageStats,
}

impl CapabilityGrant {
    /// Check if grant includes capability (optimized)
    pub fn has_capability(&self, cap: &Capability) -> bool {
        // Fast path: check bitflags if core capability
        if let Some(core) = &self.core_caps {
            if let Some(core_cap) = CoreCapabilities::from_capability(cap) {
                return core.contains(core_cap);
            }
        }
        
        // Slow path: check set
        self.capabilities.contains(cap)
    }
    
    /// Check if matches pattern
    pub fn matches_pattern(&self, pattern: &str) -> bool {
        self.capabilities.iter()
            .any(|cap| cap.matches(pattern))
    }
}
```

---

## AI Agent Integration

### Agent-Defined Capabilities

```rust
/// AI agents can define custom capabilities at runtime
pub trait AgentCapabilityProvider {
    /// Register capabilities this agent provides
    async fn register_capabilities(&self) -> Result<Vec<CapabilityDefinition>>;
    
    /// Handle capability request
    async fn handle_capability_request(
        &self,
        cap: &Capability,
        context: &RequestContext,
    ) -> Result<CapabilityResponse>;
}

/// Example: AI vision agent
pub struct VisionAgent {
    agent_id: String,
}

impl AgentCapabilityProvider for VisionAgent {
    async fn register_capabilities(&self) -> Result<Vec<CapabilityDefinition>> {
        Ok(vec![
            CapabilityDefinition {
                capability: Capability::from_str(&format!(
                    "agent:{}.vision.scene_understanding",
                    self.agent_id
                ))?,
                display_name: "Scene Understanding".to_string(),
                description: "Analyze screen content for semantic understanding".to_string(),
                risk_level: RiskLevel::Medium,
                provider: CapabilityProvider::Agent {
                    id: self.agent_id.clone(),
                },
                registered_at: SystemTime::now(),
                deprecated: None,
                related: vec![],
                requires: vec![
                    Capability::from_str("ion.capture.screen")?,
                ],
                metadata: HashMap::new(),
            },
            // More AI capabilities...
        ])
    }
    
    async fn handle_capability_request(
        &self,
        cap: &Capability,
        context: &RequestContext,
    ) -> Result<CapabilityResponse> {
        match cap.action.as_str() {
            "scene_understanding" => {
                // Perform AI analysis
                let analysis = self.analyze_scene(context.screen_data).await?;
                Ok(CapabilityResponse::SceneAnalysis(analysis))
            }
            _ => Err(Error::UnsupportedCapability(cap.to_string())),
        }
    }
}
```

### Dynamic Capability Composition

```rust
/// Compose capabilities dynamically
pub struct CapabilityComposer {
    registry: &'static CapabilityRegistry,
}

impl CapabilityComposer {
    /// Create composite capability from multiple capabilities
    pub async fn compose(
        &self,
        name: &str,
        components: Vec<Capability>,
    ) -> Result<Capability> {
        // Example: "ai.automation.smart_window_management"
        // Composed from:
        //   - ion.capture.screen (to see windows)
        //   - ion.input.keyboard (to send commands)
        //   - ai.vision.scene_understanding (to understand context)
        
        let composite = CapabilityDefinition {
            capability: Capability::from_str(name)?,
            display_name: format!("Composite: {}", name),
            description: format!("Composed from {} capabilities", components.len()),
            risk_level: components.iter()
                .map(|c| self.registry.get(c).await.ok())
                .filter_map(|d| d.map(|d| d.risk_level))
                .max()
                .unwrap_or(RiskLevel::Low),
            provider: CapabilityProvider::Core,
            registered_at: SystemTime::now(),
            deprecated: None,
            related: components.clone(),
            requires: components,
            metadata: HashMap::new(),
        };
        
        self.registry.register(composite.clone()).await?;
        Ok(composite.capability)
    }
}
```

---

## Configuration Format (Updated)

### TOML Configuration

```toml
# ~/.config/ionChannel/capabilities.toml

# Grant using string capabilities (unlimited)
[[grant]]
app_id = "rustdesk"
capabilities = [
    "ion.input.keyboard",
    "ion.input.pointer",
    "ion.capture.screen",
]
expires = "2025-12-31"
reason = "Remote desktop"

# Grant using patterns (matches multiple)
[[grant]]
app_id = "ai-assistant"
capability_patterns = [
    "ion.capture.*",      # All capture capabilities
    "ai.vision.*",        # All AI vision capabilities
    "custom.automation.*" # All custom automation
]
expires = "session"

# Agent-specific grant
[[grant]]
app_id = "ai-agent:abc123"
capabilities = [
    "agent:abc123.vision.scene_understanding",
    "agent:abc123.automation.smart_window",
]
expires = "session"

# Register custom capability
[[capability_definition]]
capability = "custom.automation.window_management"
display_name = "Window Management Automation"
description = "Automated window positioning and focus"
risk_level = "Medium"
requires = [
    "ion.input.keyboard",
    "ion.capture.screen",
]
```

---

## Plugin System

### Capability Plugins

```rust
/// Plugin that provides new capabilities
#[async_trait]
pub trait CapabilityPlugin: Send + Sync {
    /// Plugin metadata
    fn metadata(&self) -> PluginMetadata;
    
    /// Register capabilities
    async fn register(&self, registry: &CapabilityRegistry) -> Result<()>;
    
    /// Handle capability requests
    async fn handle_request(
        &self,
        cap: &Capability,
        context: &RequestContext,
    ) -> Result<CapabilityResponse>;
    
    /// Cleanup on unload
    async fn unload(&self) -> Result<()>;
}

/// Example: OCR plugin
pub struct OcrPlugin;

#[async_trait]
impl CapabilityPlugin for OcrPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "OCR Plugin".to_string(),
            version: "1.0.0".to_string(),
            author: "ionChannel Team".to_string(),
            capabilities_provided: vec![
                "plugin:ocr.text_recognition",
                "plugin:ocr.layout_detection",
            ],
        }
    }
    
    async fn register(&self, registry: &CapabilityRegistry) -> Result<()> {
        registry.register(CapabilityDefinition {
            capability: Capability::from_str("plugin:ocr.text_recognition")?,
            display_name: "Text Recognition (OCR)".to_string(),
            description: "Extract text from screen captures".to_string(),
            risk_level: RiskLevel::Low,
            provider: CapabilityProvider::Plugin {
                name: "OCR Plugin".to_string(),
            },
            registered_at: SystemTime::now(),
            deprecated: None,
            related: vec![],
            requires: vec![
                Capability::from_str("ion.capture.screen")?,
            ],
            metadata: HashMap::new(),
        }).await?;
        
        Ok(())
    }
    
    async fn handle_request(
        &self,
        cap: &Capability,
        context: &RequestContext,
    ) -> Result<CapabilityResponse> {
        match cap.action.as_str() {
            "text_recognition" => {
                let text = self.recognize_text(context.image_data).await?;
                Ok(CapabilityResponse::TextRecognition(text))
            }
            _ => Err(Error::UnsupportedCapability(cap.to_string())),
        }
    }
    
    async fn unload(&self) -> Result<()> {
        let registry = CapabilityRegistry::global();
        registry.unregister(
            &Capability::from_str("plugin:ocr.text_recognition")?
        ).await?;
        Ok(())
    }
}
```

---

## Migration from Bitflags

### Backward Compatibility

```rust
/// Bridge between old bitflags and new dynamic system
pub struct CapabilityBridge;

impl CapabilityBridge {
    /// Convert old bitflags to new capability set
    pub fn from_bitflags(core: CoreCapabilities) -> CapabilitySet {
        core.to_capability().into_iter().collect()
    }
    
    /// Convert new capability set to bitflags (best effort)
    pub fn to_bitflags(caps: &CapabilitySet) -> Option<CoreCapabilities> {
        let mut core = CoreCapabilities::empty();
        
        for cap in caps {
            if let Some(core_cap) = CoreCapabilities::from_capability(cap) {
                core |= core_cap;
            } else {
                // Contains non-core capability, can't represent fully
                return None;
            }
        }
        
        Some(core)
    }
}
```

---

## Performance Considerations

### Optimization Strategy

```rust
pub struct CapabilityChecker {
    /// Cache recent checks (hot path)
    cache: Arc<RwLock<LruCache<(String, Capability), bool>>>,
    
    /// Bloom filter for quick negative checks
    bloom: Arc<RwLock<BloomFilter>>,
}

impl CapabilityChecker {
    pub async fn check(
        &self,
        app_id: &str,
        cap: &Capability,
        grant: &CapabilityGrant,
    ) -> bool {
        // 1. Check cache (hot path, ~10ns)
        let key = (app_id.to_string(), cap.clone());
        if let Some(&result) = self.cache.read().await.get(&key) {
            return result;
        }
        
        // 2. Check bloom filter (fast negative, ~100ns)
        if !self.bloom.read().await.might_contain(&cap.to_string()) {
            return false;
        }
        
        // 3. Actual check (slower, ~1µs)
        let result = grant.has_capability(cap);
        
        // Cache result
        self.cache.write().await.put(key, result);
        
        result
    }
}
```

### Benchmarks

```rust
// Expected performance:
// - Core capability check (bitflags): ~1 CPU cycle
// - Cached string capability check: ~10ns
// - Uncached string capability check: ~1µs
// - Pattern matching: ~10µs per 1000 capabilities
// - Registry lookup: ~100ns

#[bench]
fn bench_core_capability_check(b: &mut Bencher) {
    let grant = /* ... */;
    let cap = Capability::from_str("ion.input.keyboard").unwrap();
    b.iter(|| grant.has_capability(&cap));
}

#[bench]
fn bench_string_capability_check(b: &mut Bencher) {
    let grant = /* ... */;
    let cap = Capability::from_str("custom.automation.complex").unwrap();
    b.iter(|| grant.has_capability(&cap));
}

#[bench]
fn bench_pattern_matching(b: &mut Bencher) {
    let registry = CapabilityRegistry::global();
    b.iter(|| {
        registry.find_matching("ai.vision.*").await
    });
}
```

---

## Standard Capability Namespaces

### Namespace Registry

```yaml
# Standard namespaces (community-defined)

ion:  # Core ionChannel capabilities
  input:       # Input injection
    - keyboard
    - pointer
    - touch
  capture:     # Screen/audio capture
    - screen
    - window
    - region
    - audio
  window:      # Window management
    - query
    - focus
    - position
    - close

ai:  # AI agent capabilities
  vision:      # Computer vision
    - scene_understanding
    - object_detection
    - ocr
  audio:       # Audio analysis
    - transcription
    - speaker_id
  reasoning:   # Cognitive
    - plan
    - decide
    - learn

automation:  # Automation frameworks
  ui:          # UI automation
    - click
    - type
    - wait
  workflow:    # Workflow automation
    - sequence
    - branch
    - loop

custom:  # User-defined
  *:           # Any domain/action

plugin:  # Plugin-provided
  *:           # Any plugin namespace

agent:   # Agent-specific (scoped)
  <agent-id>:  # Per-agent namespace
    *:         # Any capability

session: # Session-scoped (temporary)
  <session-id>:
    *:         # Cleanup on session end
```

---

## Future-Proofing

### Version Evolution

```rust
// Capabilities can evolve with versions
"ion.input.keyboard"         // v1 (implicit)
"ion.input.keyboard@v2"      // v2 with enhanced features
"ion.input.keyboard@legacy"  // Old version for compatibility

// Registry handles version negotiation
impl CapabilityRegistry {
    pub async fn resolve_version(
        &self,
        cap: &Capability,
        available_versions: &[String],
    ) -> Result<Capability> {
        // Find best matching version
        // Prefer exact match, then newer, then older
    }
}
```

### Forward Compatibility

```rust
// Handle unknown capabilities gracefully
impl CapabilityStore {
    pub async fn request_with_fallback(
        &self,
        caps: CapabilitySet,
    ) -> Result<CapabilityGrant> {
        let known = caps.iter()
            .filter(|c| self.registry.get(c).await.is_ok())
            .cloned()
            .collect::<CapabilitySet>();
        
        let unknown = caps.difference(&known)
            .cloned()
            .collect::<Vec<_>>();
        
        if !unknown.is_empty() {
            warn!("Unknown capabilities requested: {:?}", unknown);
            // Still allow grant, but log for investigation
        }
        
        self.create_grant(known).await
    }
}
```

---

## Conclusion

This runtime-extensible capability architecture:

### Advantages Over Bitflags
- ✅ **Unlimited capabilities** (not limited to 64)
- ✅ **Runtime registration** (no recompilation)
- ✅ **Namespace support** (avoid conflicts)
- ✅ **AI agent friendly** (dynamic discovery)
- ✅ **Plugin ecosystem** (extensible)
- ✅ **Forward compatible** (handles unknowns)

### Performance
- ✅ **Fast path for common cases** (bitflags for core)
- ✅ **Caching for hot paths** (LRU + bloom filter)
- ✅ **Acceptable overhead** (< 1µs for string checks)

### Philosophy
- ✅ **Runtime-extensible** (future-proof)
- ✅ **Agent-empowering** (AI can define capabilities)
- ✅ **Community-driven** (standard namespaces)
- ✅ **Backward compatible** (bridge to bitflags)

**This is the architecture for the AI agent era - unlimited, dynamic, and future-proof!** 🚀

---

**Status:** Design v2.0 Complete  
**Priority:** P0-ARCHITECTURE  
**Timeline:** Same 16 weeks (more robust architecture)  
**Risk:** Low (backward compatible via bridge)

