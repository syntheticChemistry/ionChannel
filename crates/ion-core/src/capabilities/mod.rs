//! Capability system - atomic primitives + dynamic registry
//!
//! This module implements a hybrid capability system with:
//! - 64 atomic primitives (bitflags, fast path)
//! - Unlimited dynamic capabilities (string-based, runtime registry)
//! - Standard compositions (RDP, automation, AI agents)
//!
//! # Architecture
//!
//! ```text
//! Core Layer: 64 atomic primitives (bitflags)
//! ├─ Fast checks (~1 CPU cycle)
//! └─ Common operations
//!
//! Dynamic Layer: Unlimited capabilities
//! ├─ Runtime registration
//! ├─ AI agent definitions
//! └─ Plugin capabilities
//! ```
//!
//! # Examples
//!
//! ```rust
//! use ion_core::capabilities::*;
//!
//! // Use atomic primitives
//! let rdp_primitives = CoreCapabilities::CAP_KEYBOARD_WRITE
//!     | CoreCapabilities::CAP_POINTER_WRITE
//!     | CoreCapabilities::CAP_SCREEN_READ;
//!
//! // Check capability
//! assert!(rdp_primitives.contains(CoreCapabilities::CAP_KEYBOARD_WRITE));
//! ```

pub mod bridge;
pub mod composition;
pub mod grant;
pub mod primitives;
pub mod registry;
pub mod storage;

// Re-exports
pub use bridge::{CapabilityPortalBridge, PortalMethod, PortalMethodDescriptions};
pub use composition::{
    get_composition, get_composition_name, list_compositions, COMPOSITION_AI_AGENT,
    COMPOSITION_AI_VISION, COMPOSITION_AUTOMATION, COMPOSITION_RDP, COMPOSITION_RECORDER,
    COMPOSITION_VIEW_ONLY,
};
pub use grant::{CapabilityGrant, CapabilityRestrictions, GrantUsageStats, TimeRange};
pub use primitives::CoreCapabilities;
pub use registry::{
    Capability, CapabilityDefinition, CapabilityError, CapabilityEvent, CapabilityProvider,
    CapabilityRegistry, DeprecationInfo, RiskLevel,
};
pub use storage::{FileSystemGrantStorage, GrantStorage, StorageError};
