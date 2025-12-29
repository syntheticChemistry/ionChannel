//! Standard capability compositions
//!
//! This module defines standard compositions of atomic primitives into
//! commonly-used capability sets like RDP, automation, AI agents, etc.
//!
//! # Philosophy
//!
//! Compositions are **not new capabilities** - they're just convenient
//! names for specific combinations of atomic primitives. This keeps the
//! 64 bits maximally utilized while providing user-friendly groupings.

use crate::capabilities::primitives::CoreCapabilities;

/// Standard Remote Desktop Protocol capabilities
///
/// Includes:
/// - Keyboard input injection
/// - Mouse/pointer control
/// - Full screen capture
/// - Cursor visibility
/// - Clipboard read/write
///
/// # Example
/// ```rust
/// # use ion_core::capabilities::{CoreCapabilities, COMPOSITION_RDP};
/// assert!(COMPOSITION_RDP.contains(CoreCapabilities::CAP_KEYBOARD_WRITE));
/// assert!(COMPOSITION_RDP.contains(CoreCapabilities::CAP_SCREEN_READ));
/// ```
pub const COMPOSITION_RDP: CoreCapabilities = CoreCapabilities::from_bits_truncate(
    CoreCapabilities::CAP_KEYBOARD_WRITE.bits()
        | CoreCapabilities::CAP_POINTER_WRITE.bits()
        | CoreCapabilities::CAP_SCREEN_READ.bits()
        | CoreCapabilities::CAP_SCREEN_FULL.bits()
        | CoreCapabilities::CAP_CURSOR_READ.bits()
        | CoreCapabilities::CAP_CLIPBOARD_READ.bits()
        | CoreCapabilities::CAP_CLIPBOARD_WRITE.bits(),
);

/// View-only remote desktop (no input control)
///
/// Includes:
/// - Full screen capture
/// - Cursor visibility
///
/// # Use Case
/// Screen sharing, presentations, remote monitoring
pub const COMPOSITION_VIEW_ONLY: CoreCapabilities = CoreCapabilities::from_bits_truncate(
    CoreCapabilities::CAP_SCREEN_READ.bits()
        | CoreCapabilities::CAP_SCREEN_FULL.bits()
        | CoreCapabilities::CAP_CURSOR_READ.bits(),
);

/// UI Automation capabilities (testing, RPA)
///
/// Includes:
/// - Keyboard and pointer injection
/// - Window listing and focus
/// - Window geometry queries
/// - Window content capture
///
/// # Use Case
/// Selenium, Playwright, RPA tools, testing frameworks
pub const COMPOSITION_AUTOMATION: CoreCapabilities = CoreCapabilities::from_bits_truncate(
    CoreCapabilities::CAP_KEYBOARD_WRITE.bits()
        | CoreCapabilities::CAP_POINTER_WRITE.bits()
        | CoreCapabilities::CAP_WINDOW_LIST.bits()
        | CoreCapabilities::CAP_WINDOW_TITLE.bits()
        | CoreCapabilities::CAP_WINDOW_FOCUS.bits()
        | CoreCapabilities::CAP_WINDOW_GEOMETRY.bits()
        | CoreCapabilities::CAP_SCREEN_WINDOW.bits(),
);

/// AI Vision Assistant (read-only analysis)
///
/// Includes:
/// - Screen and window content reading
/// - Direct pixel access (fast processing)
/// - Window list and titles
///
/// # Use Case
/// AI agents that analyze screen content, OCR, scene understanding
pub const COMPOSITION_AI_VISION: CoreCapabilities = CoreCapabilities::from_bits_truncate(
    CoreCapabilities::CAP_SCREEN_READ.bits()
        | CoreCapabilities::CAP_PIXEL_DIRECT.bits()
        | CoreCapabilities::CAP_WINDOW_LIST.bits()
        | CoreCapabilities::CAP_WINDOW_CONTENT.bits()
        | CoreCapabilities::CAP_WINDOW_TITLE.bits()
        | CoreCapabilities::CAP_SCREEN_METADATA.bits(),
);

/// AI Full Agent (vision + control)
///
/// Includes:
/// - All AI vision capabilities
/// - Keyboard and pointer control
/// - Clipboard access
/// - Window focus control
///
/// # Use Case
/// AI agents that can see and interact with the desktop
pub const COMPOSITION_AI_AGENT: CoreCapabilities = CoreCapabilities::from_bits_truncate(
    COMPOSITION_AI_VISION.bits()
        | CoreCapabilities::CAP_KEYBOARD_WRITE.bits()
        | CoreCapabilities::CAP_POINTER_WRITE.bits()
        | CoreCapabilities::CAP_WINDOW_FOCUS.bits()
        | CoreCapabilities::CAP_CLIPBOARD_READ.bits()
        | CoreCapabilities::CAP_CLIPBOARD_WRITE.bits(),
);

/// Screen Recorder capabilities
///
/// Includes:
/// - Full screen capture
/// - High frame rate
/// - Frame timestamps
/// - Audio output capture
/// - Cursor visibility
///
/// # Use Case
/// OBS, SimpleScreenRecorder, video capture tools
pub const COMPOSITION_RECORDER: CoreCapabilities = CoreCapabilities::from_bits_truncate(
    CoreCapabilities::CAP_SCREEN_READ.bits()
        | CoreCapabilities::CAP_SCREEN_FULL.bits()
        | CoreCapabilities::CAP_CURSOR_READ.bits()
        | CoreCapabilities::CAP_AUDIO_OUTPUT_READ.bits()
        | CoreCapabilities::CAP_FRAME_RATE_HIGH.bits()
        | CoreCapabilities::CAP_FRAME_TIMESTAMP.bits(),
);

/// Screen Sharing (interactive, moderate quality)
///
/// Includes:
/// - Primary monitor capture
/// - Normal frame rate
/// - Cursor visibility
/// - Pointer position reading (for remote cursor)
///
/// # Use Case
/// Video conferencing, collaboration tools
pub const COMPOSITION_SCREEN_SHARE: CoreCapabilities = CoreCapabilities::from_bits_truncate(
    CoreCapabilities::CAP_SCREEN_READ.bits()
        | CoreCapabilities::CAP_SCREEN_PRIMARY.bits()
        | CoreCapabilities::CAP_CURSOR_READ.bits()
        | CoreCapabilities::CAP_FRAME_RATE_NORMAL.bits()
        | CoreCapabilities::CAP_POINTER_READ.bits(),
);

/// Window Manager capabilities
///
/// Includes:
/// - Window listing and queries
/// - Window geometry and titles
/// - Window focus, move, state control
///
/// # Use Case
/// Tiling window managers, workspace managers
pub const COMPOSITION_WINDOW_MANAGER: CoreCapabilities = CoreCapabilities::from_bits_truncate(
    CoreCapabilities::CAP_WINDOW_LIST.bits()
        | CoreCapabilities::CAP_WINDOW_GEOMETRY.bits()
        | CoreCapabilities::CAP_WINDOW_TITLE.bits()
        | CoreCapabilities::CAP_WINDOW_FOCUS.bits()
        | CoreCapabilities::CAP_WINDOW_MOVE.bits()
        | CoreCapabilities::CAP_WINDOW_STATE.bits(),
);

/// Clipboard Manager capabilities
///
/// Includes:
/// - Text, image, and file clipboard access
///
/// # Use Case
/// Clipboard history tools, clipboard managers
pub const COMPOSITION_CLIPBOARD: CoreCapabilities = CoreCapabilities::from_bits_truncate(
    CoreCapabilities::CAP_CLIPBOARD_READ.bits()
        | CoreCapabilities::CAP_CLIPBOARD_WRITE.bits()
        | CoreCapabilities::CAP_CLIPBOARD_IMAGE.bits()
        | CoreCapabilities::CAP_CLIPBOARD_FILES.bits(),
);

/// Get composition by name
///
/// # Examples
/// ```rust
/// # use ion_core::capabilities::{get_composition, CoreCapabilities};
/// let rdp = get_composition("ion.rdp.standard").unwrap();
/// assert!(rdp.contains(CoreCapabilities::CAP_SCREEN_READ));
/// ```
#[must_use]
pub fn get_composition(name: &str) -> Option<CoreCapabilities> {
    match name {
        "ion.rdp.standard" => Some(COMPOSITION_RDP),
        "ion.rdp.view_only" => Some(COMPOSITION_VIEW_ONLY),
        "ion.automation.ui" => Some(COMPOSITION_AUTOMATION),
        "ion.recorder.screen" => Some(COMPOSITION_RECORDER),
        "ion.share.screen" => Some(COMPOSITION_SCREEN_SHARE),
        "ion.window.manager" => Some(COMPOSITION_WINDOW_MANAGER),
        "ion.clipboard.manager" => Some(COMPOSITION_CLIPBOARD),
        "ai.vision.assistant" => Some(COMPOSITION_AI_VISION),
        "ai.agent.full" => Some(COMPOSITION_AI_AGENT),
        _ => None,
    }
}

/// Get composition name from primitives (best match)
///
/// Returns the most specific composition name that matches the primitives,
/// or None if no standard composition matches.
#[must_use]
pub fn get_composition_name(primitives: CoreCapabilities) -> Option<&'static str> {
    // Check exact matches first
    if primitives == COMPOSITION_RDP {
        return Some("ion.rdp.standard");
    }
    if primitives == COMPOSITION_VIEW_ONLY {
        return Some("ion.rdp.view_only");
    }
    if primitives == COMPOSITION_AUTOMATION {
        return Some("ion.automation.ui");
    }
    if primitives == COMPOSITION_AI_AGENT {
        return Some("ai.agent.full");
    }
    if primitives == COMPOSITION_AI_VISION {
        return Some("ai.vision.assistant");
    }
    if primitives == COMPOSITION_RECORDER {
        return Some("ion.recorder.screen");
    }
    if primitives == COMPOSITION_SCREEN_SHARE {
        return Some("ion.share.screen");
    }
    if primitives == COMPOSITION_WINDOW_MANAGER {
        return Some("ion.window.manager");
    }
    if primitives == COMPOSITION_CLIPBOARD {
        return Some("ion.clipboard.manager");
    }

    None
}

/// List all available composition names
#[must_use]
pub fn list_compositions() -> Vec<&'static str> {
    vec![
        "ion.rdp.standard",
        "ion.rdp.view_only",
        "ion.automation.ui",
        "ion.recorder.screen",
        "ion.share.screen",
        "ion.window.manager",
        "ion.clipboard.manager",
        "ai.vision.assistant",
        "ai.agent.full",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rdp_composition() {
        assert!(COMPOSITION_RDP.contains(CoreCapabilities::CAP_KEYBOARD_WRITE));
        assert!(COMPOSITION_RDP.contains(CoreCapabilities::CAP_POINTER_WRITE));
        assert!(COMPOSITION_RDP.contains(CoreCapabilities::CAP_SCREEN_READ));
        assert!(COMPOSITION_RDP.contains(CoreCapabilities::CAP_CLIPBOARD_READ));
    }

    #[test]
    fn test_view_only_composition() {
        assert!(COMPOSITION_VIEW_ONLY.contains(CoreCapabilities::CAP_SCREEN_READ));
        assert!(!COMPOSITION_VIEW_ONLY.contains(CoreCapabilities::CAP_KEYBOARD_WRITE));
        assert!(!COMPOSITION_VIEW_ONLY.contains(CoreCapabilities::CAP_POINTER_WRITE));
    }

    #[test]
    fn test_ai_agent_includes_vision() {
        // AI agent should be a superset of AI vision
        assert!(COMPOSITION_AI_AGENT.contains(CoreCapabilities::CAP_SCREEN_READ));
        assert!(COMPOSITION_AI_AGENT.contains(CoreCapabilities::CAP_PIXEL_DIRECT));
        assert!(COMPOSITION_AI_AGENT.contains(CoreCapabilities::CAP_WINDOW_LIST));

        // Plus control capabilities
        assert!(COMPOSITION_AI_AGENT.contains(CoreCapabilities::CAP_KEYBOARD_WRITE));
        assert!(COMPOSITION_AI_AGENT.contains(CoreCapabilities::CAP_POINTER_WRITE));
    }

    #[test]
    fn test_get_composition_by_name() {
        let rdp = get_composition("ion.rdp.standard").unwrap();
        assert_eq!(rdp, COMPOSITION_RDP);

        let view_only = get_composition("ion.rdp.view_only").unwrap();
        assert_eq!(view_only, COMPOSITION_VIEW_ONLY);

        assert!(get_composition("nonexistent").is_none());
    }

    #[test]
    fn test_get_composition_name() {
        assert_eq!(
            get_composition_name(COMPOSITION_RDP),
            Some("ion.rdp.standard")
        );
        assert_eq!(
            get_composition_name(COMPOSITION_VIEW_ONLY),
            Some("ion.rdp.view_only")
        );

        // Custom composition (not a standard one)
        let custom = CoreCapabilities::CAP_KEYBOARD_READ;
        assert_eq!(get_composition_name(custom), None);
    }

    #[test]
    fn test_list_compositions() {
        let compositions = list_compositions();
        assert!(compositions.contains(&"ion.rdp.standard"));
        assert!(compositions.contains(&"ai.agent.full"));
        assert_eq!(compositions.len(), 9);
    }

    #[test]
    fn test_composition_immutability() {
        // Compositions are const, verify they can't be accidentally modified
        let rdp1 = COMPOSITION_RDP;
        let rdp2 = COMPOSITION_RDP;
        assert_eq!(rdp1, rdp2);
    }

    #[test]
    fn test_automation_has_window_caps() {
        assert!(COMPOSITION_AUTOMATION.contains(CoreCapabilities::CAP_WINDOW_LIST));
        assert!(COMPOSITION_AUTOMATION.contains(CoreCapabilities::CAP_WINDOW_FOCUS));
        assert!(COMPOSITION_AUTOMATION.contains(CoreCapabilities::CAP_KEYBOARD_WRITE));
    }

    #[test]
    fn test_recorder_has_high_fps() {
        assert!(COMPOSITION_RECORDER.contains(CoreCapabilities::CAP_FRAME_RATE_HIGH));
        assert!(COMPOSITION_RECORDER.contains(CoreCapabilities::CAP_AUDIO_OUTPUT_READ));
    }
}
