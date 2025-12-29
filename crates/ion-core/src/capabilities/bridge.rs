//! Bridge between capability system and xdg-desktop-portal
//!
//! This module provides translation between ionChannel's capability primitives
//! and the xdg-desktop-portal D-Bus interfaces.

use super::primitives::CoreCapabilities;
use std::collections::HashMap;

/// Maps capability primitives to portal interface methods
///
/// This provides bidirectional mapping between our atomic capabilities
/// and the D-Bus methods they enable.
pub struct CapabilityPortalBridge {
    /// Map from capability to portal methods it enables
    cap_to_methods: HashMap<CoreCapabilities, Vec<PortalMethod>>,

    /// Map from portal method to required capabilities
    method_to_caps: HashMap<PortalMethod, CoreCapabilities>,
}

/// A specific portal interface method
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PortalMethod {
    /// Portal interface name (e.g., "org.freedesktop.portal.RemoteDesktop")
    pub interface: String,

    /// Method name (e.g., "NotifyPointerMotion")
    pub method: String,
}

impl PortalMethod {
    /// Create a new portal method reference
    #[must_use]
    pub fn new(interface: impl Into<String>, method: impl Into<String>) -> Self {
        Self {
            interface: interface.into(),
            method: method.into(),
        }
    }
}

impl CapabilityPortalBridge {
    /// Create a new bridge with standard mappings
    #[must_use]
    pub fn new() -> Self {
        let mut bridge = Self {
            cap_to_methods: HashMap::new(),
            method_to_caps: HashMap::new(),
        };

        bridge.register_standard_mappings();
        bridge
    }

    /// Register all standard portal mappings
    fn register_standard_mappings(&mut self) {
        // RemoteDesktop portal
        self.register_bidirectional(
            CoreCapabilities::CAP_KEYBOARD_WRITE,
            PortalMethod::new(
                "org.freedesktop.portal.RemoteDesktop",
                "NotifyKeyboardKeycode",
            ),
        );

        self.register_bidirectional(
            CoreCapabilities::CAP_KEYBOARD_WRITE,
            PortalMethod::new(
                "org.freedesktop.portal.RemoteDesktop",
                "NotifyKeyboardKeysym",
            ),
        );

        self.register_bidirectional(
            CoreCapabilities::CAP_POINTER_WRITE,
            PortalMethod::new(
                "org.freedesktop.portal.RemoteDesktop",
                "NotifyPointerMotion",
            ),
        );

        self.register_bidirectional(
            CoreCapabilities::CAP_POINTER_WRITE,
            PortalMethod::new(
                "org.freedesktop.portal.RemoteDesktop",
                "NotifyPointerMotionAbsolute",
            ),
        );

        self.register_bidirectional(
            CoreCapabilities::CAP_POINTER_WRITE,
            PortalMethod::new(
                "org.freedesktop.portal.RemoteDesktop",
                "NotifyPointerButton",
            ),
        );

        self.register_bidirectional(
            CoreCapabilities::CAP_POINTER_WRITE,
            PortalMethod::new("org.freedesktop.portal.RemoteDesktop", "NotifyPointerAxis"),
        );

        self.register_bidirectional(
            CoreCapabilities::CAP_TOUCH_WRITE,
            PortalMethod::new("org.freedesktop.portal.RemoteDesktop", "NotifyTouchDown"),
        );

        self.register_bidirectional(
            CoreCapabilities::CAP_TOUCH_WRITE,
            PortalMethod::new("org.freedesktop.portal.RemoteDesktop", "NotifyTouchMotion"),
        );

        self.register_bidirectional(
            CoreCapabilities::CAP_TOUCH_WRITE,
            PortalMethod::new("org.freedesktop.portal.RemoteDesktop", "NotifyTouchUp"),
        );

        // ScreenCast portal
        self.register_bidirectional(
            CoreCapabilities::CAP_SCREEN_READ,
            PortalMethod::new("org.freedesktop.portal.ScreenCast", "SelectSources"),
        );

        self.register_bidirectional(
            CoreCapabilities::CAP_SCREEN_READ,
            PortalMethod::new("org.freedesktop.portal.ScreenCast", "Start"),
        );

        // Clipboard portal (future)
        self.register_bidirectional(
            CoreCapabilities::CAP_CLIPBOARD_READ,
            PortalMethod::new("org.freedesktop.portal.Clipboard", "RequestClipboard"),
        );

        self.register_bidirectional(
            CoreCapabilities::CAP_CLIPBOARD_WRITE,
            PortalMethod::new("org.freedesktop.portal.Clipboard", "SetClipboard"),
        );
    }

    /// Register a bidirectional mapping
    fn register_bidirectional(&mut self, cap: CoreCapabilities, method: PortalMethod) {
        // Capability -> Method
        self.cap_to_methods
            .entry(cap)
            .or_default()
            .push(method.clone());

        // Method -> Capability
        self.method_to_caps
            .entry(method)
            .and_modify(|caps| *caps |= cap)
            .or_insert(cap);
    }

    /// Check if capabilities allow a specific portal method
    #[must_use]
    pub fn check_method(&self, granted_caps: CoreCapabilities, method: &PortalMethod) -> bool {
        if let Some(required_caps) = self.method_to_caps.get(method) {
            granted_caps.contains(*required_caps)
        } else {
            // Unknown method - deny by default
            false
        }
    }

    /// Get all portal methods enabled by capabilities
    #[must_use]
    pub fn get_enabled_methods(&self, caps: CoreCapabilities) -> Vec<PortalMethod> {
        let mut methods = Vec::new();

        for (&cap, cap_methods) in &self.cap_to_methods {
            if caps.contains(cap) {
                methods.extend(cap_methods.clone());
            }
        }

        methods
    }

    /// Get required capabilities for a portal method
    #[must_use]
    pub fn get_required_caps(&self, method: &PortalMethod) -> Option<CoreCapabilities> {
        self.method_to_caps.get(method).copied()
    }

    /// Convert portal device types to capabilities
    ///
    /// Maps D-Bus device type flags (keyboard=1, pointer=2, touchscreen=4)
    /// to our capability primitives.
    #[must_use]
    pub fn device_types_to_caps(device_types: u32) -> CoreCapabilities {
        let mut caps = CoreCapabilities::empty();

        // Keyboard = 1
        if device_types & 1 != 0 {
            caps |= CoreCapabilities::CAP_KEYBOARD_WRITE;
        }

        // Pointer = 2
        if device_types & 2 != 0 {
            caps |= CoreCapabilities::CAP_POINTER_WRITE;
        }

        // Touchscreen = 4
        if device_types & 4 != 0 {
            caps |= CoreCapabilities::CAP_TOUCH_WRITE;
        }

        caps
    }

    /// Convert capabilities to portal device types
    ///
    /// Inverse of `device_types_to_caps`.
    #[must_use]
    pub fn caps_to_device_types(caps: CoreCapabilities) -> u32 {
        let mut device_types = 0u32;

        if caps.contains(CoreCapabilities::CAP_KEYBOARD_WRITE) {
            device_types |= 1; // Keyboard
        }

        if caps.contains(CoreCapabilities::CAP_POINTER_WRITE) {
            device_types |= 2; // Pointer
        }

        if caps.contains(CoreCapabilities::CAP_TOUCH_WRITE) {
            device_types |= 4; // Touchscreen
        }

        device_types
    }

    /// Convert portal source types to capabilities
    ///
    /// Maps D-Bus source type flags (monitor=1, window=2, virtual=4)
    /// to our screen targeting capabilities.
    #[must_use]
    pub fn source_types_to_caps(source_types: u32) -> CoreCapabilities {
        let mut caps = CoreCapabilities::CAP_SCREEN_READ;

        // Monitor = 1
        if source_types & 1 != 0 {
            caps |= CoreCapabilities::CAP_SCREEN_PRIMARY;
        }

        // Window = 2
        if source_types & 2 != 0 {
            caps |= CoreCapabilities::CAP_SCREEN_WINDOW;
        }

        // Virtual = 4 (entire desktop)
        if source_types & 4 != 0 {
            caps |= CoreCapabilities::CAP_SCREEN_FULL;
        }

        caps
    }

    /// Convert capabilities to portal source types
    #[must_use]
    pub fn caps_to_source_types(caps: CoreCapabilities) -> u32 {
        let mut source_types = 0u32;

        if caps.contains(CoreCapabilities::CAP_SCREEN_PRIMARY) {
            source_types |= 1; // Monitor
        }

        if caps.contains(CoreCapabilities::CAP_SCREEN_WINDOW) {
            source_types |= 2; // Window
        }

        if caps.contains(CoreCapabilities::CAP_SCREEN_FULL) {
            source_types |= 4; // Virtual
        }

        source_types
    }
}

impl Default for CapabilityPortalBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to generate user-facing descriptions for portal methods
pub struct PortalMethodDescriptions;

impl PortalMethodDescriptions {
    /// Get human-readable description of what a portal method does
    #[must_use]
    pub fn describe(method: &PortalMethod) -> String {
        match (method.interface.as_str(), method.method.as_str()) {
            ("org.freedesktop.portal.RemoteDesktop", "NotifyKeyboardKeycode") => {
                "Send keyboard key presses (by keycode)".to_string()
            },
            ("org.freedesktop.portal.RemoteDesktop", "NotifyKeyboardKeysym") => {
                "Send keyboard key presses (by keysym)".to_string()
            },
            ("org.freedesktop.portal.RemoteDesktop", "NotifyPointerMotion") => {
                "Move the mouse cursor (relative)".to_string()
            },
            ("org.freedesktop.portal.RemoteDesktop", "NotifyPointerMotionAbsolute") => {
                "Move the mouse cursor (absolute)".to_string()
            },
            ("org.freedesktop.portal.RemoteDesktop", "NotifyPointerButton") => {
                "Send mouse button clicks".to_string()
            },
            ("org.freedesktop.portal.RemoteDesktop", "NotifyPointerAxis") => {
                "Send mouse scroll events".to_string()
            },
            ("org.freedesktop.portal.RemoteDesktop", "NotifyTouchDown") => {
                "Send touch press events".to_string()
            },
            ("org.freedesktop.portal.RemoteDesktop", "NotifyTouchMotion") => {
                "Send touch move events".to_string()
            },
            ("org.freedesktop.portal.RemoteDesktop", "NotifyTouchUp") => {
                "Send touch release events".to_string()
            },
            ("org.freedesktop.portal.ScreenCast", "SelectSources") => {
                "Select which screen/window to capture".to_string()
            },
            ("org.freedesktop.portal.ScreenCast", "Start") => {
                "Start screen capture stream".to_string()
            },
            ("org.freedesktop.portal.Clipboard", "RequestClipboard") => {
                "Read clipboard contents".to_string()
            },
            ("org.freedesktop.portal.Clipboard", "SetClipboard") => {
                "Write clipboard contents".to_string()
            },
            _ => format!("Call {}.{}", method.interface, method.method),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let bridge = CapabilityPortalBridge::new();
        assert!(!bridge.cap_to_methods.is_empty());
        assert!(!bridge.method_to_caps.is_empty());
    }

    #[test]
    fn test_check_method() {
        let bridge = CapabilityPortalBridge::new();

        let caps = CoreCapabilities::CAP_KEYBOARD_WRITE;
        let method = PortalMethod::new(
            "org.freedesktop.portal.RemoteDesktop",
            "NotifyKeyboardKeycode",
        );

        assert!(bridge.check_method(caps, &method));
    }

    #[test]
    fn test_check_method_denied() {
        let bridge = CapabilityPortalBridge::new();

        // Grant keyboard, but try to use pointer
        let caps = CoreCapabilities::CAP_KEYBOARD_WRITE;
        let method = PortalMethod::new(
            "org.freedesktop.portal.RemoteDesktop",
            "NotifyPointerMotion",
        );

        assert!(!bridge.check_method(caps, &method));
    }

    #[test]
    fn test_get_enabled_methods() {
        let bridge = CapabilityPortalBridge::new();

        let caps = CoreCapabilities::CAP_KEYBOARD_WRITE | CoreCapabilities::CAP_POINTER_WRITE;
        let methods = bridge.get_enabled_methods(caps);

        // Should have keyboard + pointer methods
        assert!(!methods.is_empty());
        assert!(methods.len() >= 6); // At least 2 keyboard + 4 pointer methods
    }

    #[test]
    fn test_device_types_conversion() {
        // Keyboard only
        let device_types = 1u32;
        let caps = CapabilityPortalBridge::device_types_to_caps(device_types);
        assert!(caps.contains(CoreCapabilities::CAP_KEYBOARD_WRITE));
        assert!(!caps.contains(CoreCapabilities::CAP_POINTER_WRITE));

        // Keyboard + Pointer
        let device_types = 3u32; // 1 | 2
        let caps = CapabilityPortalBridge::device_types_to_caps(device_types);
        assert!(caps.contains(CoreCapabilities::CAP_KEYBOARD_WRITE));
        assert!(caps.contains(CoreCapabilities::CAP_POINTER_WRITE));

        // All devices
        let device_types = 7u32; // 1 | 2 | 4
        let caps = CapabilityPortalBridge::device_types_to_caps(device_types);
        assert!(caps.contains(CoreCapabilities::CAP_KEYBOARD_WRITE));
        assert!(caps.contains(CoreCapabilities::CAP_POINTER_WRITE));
        assert!(caps.contains(CoreCapabilities::CAP_TOUCH_WRITE));
    }

    #[test]
    fn test_caps_to_device_types() {
        let caps = CoreCapabilities::CAP_KEYBOARD_WRITE | CoreCapabilities::CAP_POINTER_WRITE;
        let device_types = CapabilityPortalBridge::caps_to_device_types(caps);
        assert_eq!(device_types, 3); // Keyboard (1) | Pointer (2)
    }

    #[test]
    fn test_device_types_roundtrip() {
        let original_types = 5u32; // Keyboard + Touch
        let caps = CapabilityPortalBridge::device_types_to_caps(original_types);
        let converted_types = CapabilityPortalBridge::caps_to_device_types(caps);
        assert_eq!(original_types, converted_types);
    }

    #[test]
    fn test_source_types_conversion() {
        // Monitor only
        let source_types = 1u32;
        let caps = CapabilityPortalBridge::source_types_to_caps(source_types);
        assert!(caps.contains(CoreCapabilities::CAP_SCREEN_READ));
        assert!(caps.contains(CoreCapabilities::CAP_SCREEN_PRIMARY));

        // Window only
        let source_types = 2u32;
        let caps = CapabilityPortalBridge::source_types_to_caps(source_types);
        assert!(caps.contains(CoreCapabilities::CAP_SCREEN_WINDOW));

        // Virtual (full desktop)
        let source_types = 4u32;
        let caps = CapabilityPortalBridge::source_types_to_caps(source_types);
        assert!(caps.contains(CoreCapabilities::CAP_SCREEN_FULL));
    }

    #[test]
    fn test_source_types_roundtrip() {
        let original_types = 3u32; // Monitor + Window
        let caps = CapabilityPortalBridge::source_types_to_caps(original_types);
        let converted_types = CapabilityPortalBridge::caps_to_source_types(caps);
        assert_eq!(original_types, converted_types);
    }

    #[test]
    fn test_method_description() {
        let method = PortalMethod::new(
            "org.freedesktop.portal.RemoteDesktop",
            "NotifyKeyboardKeycode",
        );
        let desc = PortalMethodDescriptions::describe(&method);
        assert!(!desc.is_empty());
        assert!(desc.contains("keyboard"));
    }

    #[test]
    fn test_unknown_method() {
        let bridge = CapabilityPortalBridge::new();

        let caps = CoreCapabilities::all();
        let method = PortalMethod::new("org.example.UnknownPortal", "UnknownMethod");

        // Should deny unknown methods by default
        assert!(!bridge.check_method(caps, &method));
    }

    #[test]
    fn test_get_required_caps() {
        let bridge = CapabilityPortalBridge::new();

        let method = PortalMethod::new(
            "org.freedesktop.portal.RemoteDesktop",
            "NotifyPointerMotion",
        );

        let required = bridge.get_required_caps(&method).unwrap();
        assert!(required.contains(CoreCapabilities::CAP_POINTER_WRITE));
    }
}
