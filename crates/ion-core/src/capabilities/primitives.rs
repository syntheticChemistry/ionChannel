//! Core capability primitives (64 atomic operations)
//!
//! This module defines the 64 fundamental capability primitives that form
//! the basis of all higher-level capabilities. Each primitive represents
//! an atomic operation that cannot be subdivided further.
//!
//! # Design Philosophy
//!
//! - **Atomic**: Each capability is indivisible
//! - **Composable**: High-level capabilities are compositions
//! - **Complete**: 64 primitives cover all operations
//! - **Efficient**: Bitflags provide O(1) checks
//!
//! # Categories
//!
//! 1. **Input** (Bits 0-15): Keyboard, mouse, touch, gamepad
//! 2. **Screen** (Bits 16-27): Reading, targeting, frame control
//! 3. **Window** (Bits 28-35): Query, control
//! 4. **Data** (Bits 36-43): Clipboard, files
//! 5. **Audio** (Bits 44-47): Input/output
//! 6. **System** (Bits 48-51): Process, memory, network
//! 7. **Notifications** (Bits 52-55): Read, write, dismiss
//! 8. **Meta** (Bits 56-63): Session control, grants

use bitflags::bitflags;

bitflags! {
    /// Core capability primitives (64 atomic operations)
    ///
    /// Each bit represents a fundamental operation. High-level capabilities
    /// (like RDP, automation, AI workflows) are compositions of these primitives.
    ///
    /// # Performance
    ///
    /// Bitflag operations are extremely fast (~1 CPU cycle):
    /// - Check: `caps.contains(CAP_KEYBOARD_WRITE)`
    /// - Union: `caps | other_caps`
    /// - Intersection: `caps & other_caps`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ion_core::capabilities::CoreCapabilities;
    /// // Create RDP capability set
    /// let rdp = CoreCapabilities::CAP_KEYBOARD_WRITE
    ///     | CoreCapabilities::CAP_POINTER_WRITE
    ///     | CoreCapabilities::CAP_SCREEN_READ;
    ///
    /// // Check if has keyboard capability
    /// assert!(rdp.contains(CoreCapabilities::CAP_KEYBOARD_WRITE));
    ///
    /// // Add clipboard capability
    /// let rdp_with_clipboard = rdp | CoreCapabilities::CAP_CLIPBOARD_READ;
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CoreCapabilities: u64 {
        // ===== INPUT: KEYBOARD (Bits 0-3) =====

        /// Read keyboard state (detect which keys are pressed)
        const CAP_KEYBOARD_READ         = 1 << 0;

        /// Inject keyboard events (type, send key presses)
        const CAP_KEYBOARD_WRITE        = 1 << 1;

        /// Access raw keycodes (bypass keyboard layout)
        const CAP_KEYBOARD_RAW          = 1 << 2;

        /// Intercept keyboard events before applications see them
        /// **WARNING**: This is dangerous - enables keyloggers
        const CAP_KEYBOARD_INTERCEPT    = 1 << 3;

        // ===== INPUT: POINTER (Bits 4-7) =====

        /// Read mouse/pointer position and button state
        const CAP_POINTER_READ          = 1 << 4;

        /// Move mouse cursor and inject click events
        const CAP_POINTER_WRITE         = 1 << 5;

        /// Lock pointer to a window (for games, 3D apps)
        const CAP_POINTER_LOCK          = 1 << 6;

        /// Warp pointer instantly across screens
        const CAP_POINTER_WARP          = 1 << 7;

        // ===== INPUT: TOUCH (Bits 8-11) =====

        /// Read touch events from touchscreens
        const CAP_TOUCH_READ            = 1 << 8;

        /// Inject touch events (simulate taps, swipes)
        const CAP_TOUCH_WRITE           = 1 << 9;

        /// Read complex gestures (pinch, rotate, etc.)
        const CAP_GESTURE_READ          = 1 << 10;

        /// Inject complex gestures
        const CAP_GESTURE_WRITE         = 1 << 11;

        // ===== INPUT: OTHER (Bits 12-15) =====

        /// Read gamepad/controller state
        const CAP_GAMEPAD_READ          = 1 << 12;

        /// Inject gamepad/controller events
        const CAP_GAMEPAD_WRITE         = 1 << 13;

        /// Read stylus/pen input (pressure, tilt, etc.)
        const CAP_STYLUS_READ           = 1 << 14;

        /// Inject stylus/pen events
        const CAP_STYLUS_WRITE          = 1 << 15;

        // ===== SCREEN: READING (Bits 16-19) =====

        /// Read screen pixels (any method: DMA-BUF, SHM, PipeWire)
        const CAP_SCREEN_READ           = 1 << 16;

        /// Read screen metadata (size, DPI, rotation, color depth)
        const CAP_SCREEN_METADATA       = 1 << 17;

        /// Direct framebuffer access (fastest, zero-copy)
        const CAP_PIXEL_DIRECT          = 1 << 18;

        /// Read cursor image and position
        const CAP_CURSOR_READ           = 1 << 19;

        // ===== SCREEN: TARGETING (Bits 20-23) =====

        /// Capture all monitors
        const CAP_SCREEN_FULL           = 1 << 20;

        /// Capture primary monitor only
        const CAP_SCREEN_PRIMARY        = 1 << 21;

        /// Capture specific rectangular region
        const CAP_SCREEN_REGION         = 1 << 22;

        /// Capture specific window only
        const CAP_SCREEN_WINDOW         = 1 << 23;

        // ===== SCREEN: FRAME CONTROL (Bits 24-27) =====

        /// Normal frame rate (up to 30 FPS)
        const CAP_FRAME_RATE_NORMAL     = 1 << 24;

        /// High frame rate (up to 60 FPS)
        const CAP_FRAME_RATE_HIGH       = 1 << 25;

        /// Unlimited frame rate (for streaming, recording)
        const CAP_FRAME_RATE_UNLIMITED  = 1 << 26;

        /// Precise frame timestamps (for synchronization)
        const CAP_FRAME_TIMESTAMP       = 1 << 27;

        // ===== WINDOW: QUERY (Bits 28-31) =====

        /// List all windows
        const CAP_WINDOW_LIST           = 1 << 28;

        /// Read window position and size
        const CAP_WINDOW_GEOMETRY       = 1 << 29;

        /// Read window titles
        const CAP_WINDOW_TITLE          = 1 << 30;

        /// Read window content (pixel buffer)
        const CAP_WINDOW_CONTENT        = 1 << 31;

        // ===== WINDOW: CONTROL (Bits 32-35) =====

        /// Change window focus
        const CAP_WINDOW_FOCUS          = 1 << 32;

        /// Move and resize windows
        const CAP_WINDOW_MOVE           = 1 << 33;

        /// Minimize, maximize, fullscreen
        const CAP_WINDOW_STATE          = 1 << 34;

        /// Close windows
        /// **WARNING**: This is dangerous - can close important windows
        const CAP_WINDOW_CLOSE          = 1 << 35;

        // ===== DATA: CLIPBOARD (Bits 36-39) =====

        /// Read clipboard text
        const CAP_CLIPBOARD_READ        = 1 << 36;

        /// Write clipboard text
        const CAP_CLIPBOARD_WRITE       = 1 << 37;

        /// Read/write clipboard images
        const CAP_CLIPBOARD_IMAGE       = 1 << 38;

        /// Read/write clipboard file paths
        const CAP_CLIPBOARD_FILES       = 1 << 39;

        // ===== DATA: FILES (Bits 40-43) =====

        /// Read files (with user approval per file)
        const CAP_FILE_READ             = 1 << 40;

        /// Write files (with user approval per file)
        const CAP_FILE_WRITE            = 1 << 41;

        /// Delete files
        /// **WARNING**: This is dangerous - potential data loss
        const CAP_FILE_DELETE           = 1 << 42;

        /// Execute files
        /// **WARNING**: This is VERY dangerous - malware vector
        const CAP_FILE_EXECUTE          = 1 << 43;

        // ===== AUDIO (Bits 44-47) =====

        /// Capture system audio output (speakers/headphones)
        const CAP_AUDIO_OUTPUT_READ     = 1 << 44;

        /// Play audio through system
        const CAP_AUDIO_OUTPUT_WRITE    = 1 << 45;

        /// Capture microphone input
        const CAP_AUDIO_INPUT_READ      = 1 << 46;

        /// Inject virtual microphone audio
        const CAP_AUDIO_INPUT_WRITE     = 1 << 47;

        // ===== SYSTEM INFO (Bits 48-51) =====

        /// List running processes
        const CAP_SYSTEM_PROCESS_LIST   = 1 << 48;

        /// Read memory usage statistics
        const CAP_SYSTEM_MEMORY_INFO    = 1 << 49;

        /// Read network interface and statistics
        const CAP_SYSTEM_NETWORK_INFO   = 1 << 50;

        /// Read hardware information (CPU, GPU, etc.)
        const CAP_SYSTEM_HARDWARE_INFO  = 1 << 51;

        // ===== NOTIFICATIONS (Bits 52-55) =====

        /// Read notification content
        const CAP_NOTIFICATION_READ     = 1 << 52;

        /// Send/display notifications
        const CAP_NOTIFICATION_WRITE    = 1 << 53;

        /// Dismiss notifications
        const CAP_NOTIFICATION_DISMISS  = 1 << 54;

        /// Intercept notifications before display
        const CAP_NOTIFICATION_INTERCEPT = 1 << 55;

        // ===== META/CONTROL (Bits 56-63) =====

        /// Work in background (without window focus)
        const CAP_SESSION_BACKGROUND    = 1 << 56;

        /// Persist grant across sessions
        const CAP_SESSION_PERSIST       = 1 << 57;

        /// Bypass rate limiting
        /// **WARNING**: This is dangerous - enables abuse
        const CAP_RATE_LIMIT_BYPASS     = 1 << 58;

        /// Don't log to audit trail
        /// **WARNING**: This is VERY dangerous - eliminates accountability
        const CAP_AUDIT_EXEMPT          = 1 << 59;

        /// Query available capabilities
        const CAP_CAPABILITY_QUERY      = 1 << 60;

        /// Register new capabilities (for plugins)
        const CAP_CAPABILITY_REGISTER   = 1 << 61;

        /// Manage other applications' grants
        /// **WARNING**: This is admin-level - can modify all security
        const CAP_GRANT_MANAGE          = 1 << 62;

        /// Grant all capabilities
        /// **WARNING**: This is EXTREMELY dangerous - full system control
        const CAP_GRANT_ALL             = 1 << 63;
    }
}

impl CoreCapabilities {
    /// Convert to string capability name
    pub fn to_capability_name(self) -> Vec<&'static str> {
        let mut names = Vec::new();

        if self.contains(Self::CAP_KEYBOARD_READ) {
            names.push("ion.primitive.keyboard.read");
        }
        if self.contains(Self::CAP_KEYBOARD_WRITE) {
            names.push("ion.primitive.keyboard.write");
        }
        if self.contains(Self::CAP_POINTER_READ) {
            names.push("ion.primitive.pointer.read");
        }
        if self.contains(Self::CAP_POINTER_WRITE) {
            names.push("ion.primitive.pointer.write");
        }
        if self.contains(Self::CAP_SCREEN_READ) {
            names.push("ion.primitive.screen.read");
        }
        // ... more mappings

        names
    }

    /// Parse from string capability name
    pub fn from_capability_name(name: &str) -> Option<Self> {
        match name {
            "ion.primitive.keyboard.read" => Some(Self::CAP_KEYBOARD_READ),
            "ion.primitive.keyboard.write" => Some(Self::CAP_KEYBOARD_WRITE),
            "ion.primitive.pointer.read" => Some(Self::CAP_POINTER_READ),
            "ion.primitive.pointer.write" => Some(Self::CAP_POINTER_WRITE),
            "ion.primitive.screen.read" => Some(Self::CAP_SCREEN_READ),
            // ... more mappings
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_operations() {
        let caps = CoreCapabilities::CAP_KEYBOARD_WRITE | CoreCapabilities::CAP_POINTER_WRITE;

        assert!(caps.contains(CoreCapabilities::CAP_KEYBOARD_WRITE));
        assert!(caps.contains(CoreCapabilities::CAP_POINTER_WRITE));
        assert!(!caps.contains(CoreCapabilities::CAP_SCREEN_READ));
    }

    #[test]
    fn test_bitflag_union() {
        let input_caps = CoreCapabilities::CAP_KEYBOARD_WRITE | CoreCapabilities::CAP_POINTER_WRITE;
        let screen_caps = CoreCapabilities::CAP_SCREEN_READ | CoreCapabilities::CAP_CURSOR_READ;

        let rdp_caps = input_caps | screen_caps;

        assert!(rdp_caps.contains(CoreCapabilities::CAP_KEYBOARD_WRITE));
        assert!(rdp_caps.contains(CoreCapabilities::CAP_SCREEN_READ));
    }

    #[test]
    fn test_bitflag_intersection() {
        let caps1 = CoreCapabilities::CAP_KEYBOARD_WRITE
            | CoreCapabilities::CAP_POINTER_WRITE
            | CoreCapabilities::CAP_SCREEN_READ;

        let caps2 = CoreCapabilities::CAP_KEYBOARD_WRITE | CoreCapabilities::CAP_SCREEN_READ;

        let common = caps1 & caps2;

        assert!(common.contains(CoreCapabilities::CAP_KEYBOARD_WRITE));
        assert!(common.contains(CoreCapabilities::CAP_SCREEN_READ));
        assert!(!common.contains(CoreCapabilities::CAP_POINTER_WRITE));
    }

    #[test]
    fn test_bitflag_difference() {
        let caps1 = CoreCapabilities::CAP_KEYBOARD_WRITE
            | CoreCapabilities::CAP_POINTER_WRITE
            | CoreCapabilities::CAP_SCREEN_READ;

        let caps2 = CoreCapabilities::CAP_KEYBOARD_WRITE;

        let diff = caps1 & !caps2;

        assert!(!diff.contains(CoreCapabilities::CAP_KEYBOARD_WRITE));
        assert!(diff.contains(CoreCapabilities::CAP_POINTER_WRITE));
        assert!(diff.contains(CoreCapabilities::CAP_SCREEN_READ));
    }

    #[test]
    fn test_empty_capabilities() {
        let empty = CoreCapabilities::empty();
        assert!(!empty.contains(CoreCapabilities::CAP_KEYBOARD_WRITE));
        assert!(empty.is_empty());
    }

    #[test]
    fn test_all_capabilities() {
        let all = CoreCapabilities::all();
        assert!(all.contains(CoreCapabilities::CAP_KEYBOARD_WRITE));
        assert!(all.contains(CoreCapabilities::CAP_SCREEN_READ));
        assert!(all.contains(CoreCapabilities::CAP_GRANT_ALL));
    }
}
