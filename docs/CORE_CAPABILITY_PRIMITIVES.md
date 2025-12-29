# Core Capability Primitives - Atomic Building Blocks

**Philosophy:** 64 bits = 64 atomic primitives that compose into everything else

**Date:** December 28, 2025

---

## 🎯 Design Principle

### Atomic vs Composite

#### ❌ Bad (Redundant, Limited)
```rust
const CAP_REMOTE_DESKTOP = 1 << 0;    // Redundant!
const CAP_AUTOMATION = 1 << 1;        // Redundant!
const CAP_SCREEN_SHARING = 1 << 2;    // Redundant!
const CAP_INJECT_KEYBOARD = 1 << 3;   // Primitive ✓
// We've wasted 3 bits on things that are just combinations!
```

#### ✅ Good (Atomic, Composable)
```rust
// Primitives only - everything builds from these
const CAP_KEYBOARD_READ = 1 << 0;
const CAP_KEYBOARD_WRITE = 1 << 1;
const CAP_MOUSE_READ = 1 << 2;
const CAP_MOUSE_WRITE = 1 << 3;
const CAP_SCREEN_READ = 1 << 4;

// High-level capabilities are compositions:
// RDP = KEYBOARD_WRITE | MOUSE_WRITE | SCREEN_READ
// Automation = KEYBOARD_WRITE | MOUSE_WRITE | WINDOW_QUERY
// AI Vision = SCREEN_READ | PIXEL_ACCESS
```

---

## 📊 64 Core Primitives (Atomic Building Blocks)

### Category 1: Input Device Access (Bits 0-15)

#### Keyboard (Bits 0-3)
```rust
const CAP_KEYBOARD_READ         = 1 << 0;   // Read key state, detect pressed keys
const CAP_KEYBOARD_WRITE        = 1 << 1;   // Inject key events
const CAP_KEYBOARD_RAW          = 1 << 2;   // Raw keycodes (bypass layout)
const CAP_KEYBOARD_INTERCEPT    = 1 << 3;   // Intercept before apps see them
```

**Usage:**
- **RDP:** `KEYBOARD_WRITE` (inject remote keystrokes)
- **Keylogger:** `KEYBOARD_READ + KEYBOARD_INTERCEPT` (dangerous!)
- **Input mapper:** `KEYBOARD_READ + KEYBOARD_WRITE` (remap keys)
- **AI assistant:** `KEYBOARD_WRITE` (type for user)

#### Mouse/Pointer (Bits 4-7)
```rust
const CAP_POINTER_READ          = 1 << 4;   // Read mouse position/buttons
const CAP_POINTER_WRITE         = 1 << 5;   // Move mouse, inject clicks
const CAP_POINTER_LOCK          = 1 << 6;   // Lock pointer to window (games)
const CAP_POINTER_WARP          = 1 << 7;   // Warp across screens instantly
```

**Usage:**
- **RDP:** `POINTER_WRITE` (remote mouse control)
- **Screen recorder:** `POINTER_READ` (highlight clicks)
- **Automation:** `POINTER_WRITE` (click buttons)
- **AI agent:** `POINTER_READ + POINTER_WRITE` (understand & interact)

#### Touch (Bits 8-11)
```rust
const CAP_TOUCH_READ            = 1 << 8;   // Read touch events
const CAP_TOUCH_WRITE           = 1 << 9;   // Inject touch events
const CAP_GESTURE_READ          = 1 << 10;  // Read complex gestures
const CAP_GESTURE_WRITE         = 1 << 11;  // Inject gestures (pinch, swipe)
```

#### Other Input (Bits 12-15)
```rust
const CAP_GAMEPAD_READ          = 1 << 12;  // Read gamepad state
const CAP_GAMEPAD_WRITE         = 1 << 13;  // Inject gamepad events
const CAP_STYLUS_READ           = 1 << 14;  // Pen/stylus input
const CAP_STYLUS_WRITE          = 1 << 15;  // Inject stylus events
```

---

### Category 2: Screen/Display Access (Bits 16-27)

#### Screen Reading (Bits 16-19)
```rust
const CAP_SCREEN_READ           = 1 << 16;  // Read screen pixels (any method)
const CAP_SCREEN_METADATA       = 1 << 17;  // Screen info (size, DPI, rotation)
const CAP_PIXEL_DIRECT          = 1 << 18;  // Direct framebuffer access
const CAP_CURSOR_READ           = 1 << 19;  // Read cursor image/position
```

**Usage:**
- **RDP:** `SCREEN_READ` (send screen to remote)
- **OCR:** `SCREEN_READ + PIXEL_DIRECT` (fast text extraction)
- **AI vision:** `SCREEN_READ + PIXEL_DIRECT` (scene understanding)
- **Screenshot:** `SCREEN_READ` (capture)

#### Screen Targeting (Bits 20-23)
```rust
const CAP_SCREEN_FULL           = 1 << 20;  // Capture all monitors
const CAP_SCREEN_PRIMARY        = 1 << 21;  // Primary monitor only
const CAP_SCREEN_REGION         = 1 << 22;  // Specific rectangle
const CAP_SCREEN_WINDOW         = 1 << 23;  // Specific window only
```

#### Frame Control (Bits 24-27)
```rust
const CAP_FRAME_RATE_NORMAL     = 1 << 24;  // Up to 30 FPS
const CAP_FRAME_RATE_HIGH       = 1 << 25;  // Up to 60 FPS
const CAP_FRAME_RATE_UNLIMITED  = 1 << 26;  // Unlimited (streaming)
const CAP_FRAME_TIMESTAMP       = 1 << 27;  // Precise frame timing
```

---

### Category 3: Window Management (Bits 28-35)

#### Window Query (Bits 28-31)
```rust
const CAP_WINDOW_LIST           = 1 << 28;  // List all windows
const CAP_WINDOW_GEOMETRY       = 1 << 29;  // Read position/size
const CAP_WINDOW_TITLE          = 1 << 30;  // Read window titles
const CAP_WINDOW_CONTENT        = 1 << 31;  // Read window buffer (pixels)
```

#### Window Control (Bits 32-35)
```rust
const CAP_WINDOW_FOCUS          = 1 << 32;  // Change focus
const CAP_WINDOW_MOVE           = 1 << 33;  // Move/resize windows
const CAP_WINDOW_STATE          = 1 << 34;  // Minimize/maximize/fullscreen
const CAP_WINDOW_CLOSE          = 1 << 35;  // Close windows (dangerous!)
```

---

### Category 4: Data Exchange (Bits 36-43)

#### Clipboard (Bits 36-39)
```rust
const CAP_CLIPBOARD_READ        = 1 << 36;  // Read clipboard text
const CAP_CLIPBOARD_WRITE       = 1 << 37;  // Write clipboard text
const CAP_CLIPBOARD_IMAGE       = 1 << 38;  // Read/write images
const CAP_CLIPBOARD_FILES       = 1 << 39;  // Read/write file paths
```

#### File Transfer (Bits 40-43)
```rust
const CAP_FILE_READ             = 1 << 40;  // Read files (with user approval)
const CAP_FILE_WRITE            = 1 << 41;  // Write files (with user approval)
const CAP_FILE_DELETE           = 1 << 42;  // Delete files (dangerous!)
const CAP_FILE_EXECUTE          = 1 << 43;  // Execute files (very dangerous!)
```

---

### Category 5: Audio (Bits 44-47)

```rust
const CAP_AUDIO_OUTPUT_READ     = 1 << 44;  // Capture system audio
const CAP_AUDIO_OUTPUT_WRITE    = 1 << 45;  // Play audio
const CAP_AUDIO_INPUT_READ      = 1 << 46;  // Capture microphone
const CAP_AUDIO_INPUT_WRITE     = 1 << 47;  // Inject virtual mic audio
```

---

### Category 6: System Info (Bits 48-51)

```rust
const CAP_SYSTEM_PROCESS_LIST   = 1 << 48;  // List running processes
const CAP_SYSTEM_MEMORY_INFO    = 1 << 49;  // Memory usage stats
const CAP_SYSTEM_NETWORK_INFO   = 1 << 50;  // Network interfaces/stats
const CAP_SYSTEM_HARDWARE_INFO  = 1 << 51;  // CPU, GPU, etc.
```

---

### Category 7: Notifications (Bits 52-55)

```rust
const CAP_NOTIFICATION_READ     = 1 << 52;  // Read notification content
const CAP_NOTIFICATION_WRITE    = 1 << 53;  // Send notifications
const CAP_NOTIFICATION_DISMISS  = 1 << 54;  // Dismiss notifications
const CAP_NOTIFICATION_INTERCEPT = 1 << 55; // Intercept before display
```

---

### Category 8: Meta/Control (Bits 56-63)

```rust
const CAP_SESSION_BACKGROUND    = 1 << 56;  // Work without focus
const CAP_SESSION_PERSIST       = 1 << 57;  // Remember grant across sessions
const CAP_RATE_LIMIT_BYPASS     = 1 << 58;  // Bypass rate limiting
const CAP_AUDIT_EXEMPT          = 1 << 59;  // Don't log to audit (dangerous!)

const CAP_CAPABILITY_QUERY      = 1 << 60;  // Query available capabilities
const CAP_CAPABILITY_REGISTER   = 1 << 61;  // Register new capabilities (plugins)
const CAP_GRANT_MANAGE          = 1 << 62;  // Manage other grants (admin)
const CAP_GRANT_ALL             = 1 << 63;  // Grant all capabilities (super dangerous!)
```

---

## 🔧 Composing High-Level Capabilities

### Standard Compositions

```rust
/// Remote Desktop Protocol (standard use case)
pub const COMPOSITION_RDP: CoreCapabilities = 
    CoreCapabilities::CAP_KEYBOARD_WRITE
    .union(CoreCapabilities::CAP_POINTER_WRITE)
    .union(CoreCapabilities::CAP_SCREEN_READ)
    .union(CoreCapabilities::CAP_SCREEN_FULL)
    .union(CoreCapabilities::CAP_CURSOR_READ)
    .union(CoreCapabilities::CAP_CLIPBOARD_READ)
    .union(CoreCapabilities::CAP_CLIPBOARD_WRITE);
// String: "ion.rdp.standard"

/// View-Only Remote Desktop
pub const COMPOSITION_VIEW_ONLY: CoreCapabilities =
    CoreCapabilities::CAP_SCREEN_READ
    .union(CoreCapabilities::CAP_SCREEN_FULL)
    .union(CoreCapabilities::CAP_CURSOR_READ);
// String: "ion.rdp.view_only"

/// UI Automation (testing/RPA)
pub const COMPOSITION_AUTOMATION: CoreCapabilities =
    CoreCapabilities::CAP_KEYBOARD_WRITE
    .union(CoreCapabilities::CAP_POINTER_WRITE)
    .union(CoreCapabilities::CAP_WINDOW_LIST)
    .union(CoreCapabilities::CAP_WINDOW_TITLE)
    .union(CoreCapabilities::CAP_WINDOW_FOCUS)
    .union(CoreCapabilities::CAP_SCREEN_WINDOW);
// String: "ion.automation.ui"

/// AI Vision Assistant
pub const COMPOSITION_AI_VISION: CoreCapabilities =
    CoreCapabilities::CAP_SCREEN_READ
    .union(CoreCapabilities::CAP_PIXEL_DIRECT)
    .union(CoreCapabilities::CAP_WINDOW_LIST)
    .union(CoreCapabilities::CAP_WINDOW_CONTENT)
    .union(CoreCapabilities::CAP_WINDOW_TITLE);
// String: "ai.vision.assistant"

/// AI Full Agent (vision + control)
pub const COMPOSITION_AI_AGENT: CoreCapabilities =
    COMPOSITION_AI_VISION
    .union(CoreCapabilities::CAP_KEYBOARD_WRITE)
    .union(CoreCapabilities::CAP_POINTER_WRITE)
    .union(CoreCapabilities::CAP_WINDOW_FOCUS)
    .union(CoreCapabilities::CAP_CLIPBOARD_READ)
    .union(CoreCapabilities::CAP_CLIPBOARD_WRITE);
// String: "ai.agent.full"

/// Screen Recorder
pub const COMPOSITION_RECORDER: CoreCapabilities =
    CoreCapabilities::CAP_SCREEN_READ
    .union(CoreCapabilities::CAP_SCREEN_FULL)
    .union(CoreCapabilities::CAP_CURSOR_READ)
    .union(CoreCapabilities::CAP_AUDIO_OUTPUT_READ)
    .union(CoreCapabilities::CAP_FRAME_RATE_HIGH)
    .union(CoreCapabilities::CAP_FRAME_TIMESTAMP);
// String: "ion.recorder.screen"
```

---

## 🎯 Benefits Summary

### 1. **Complete Coverage**
- 64 bits = 64 fundamental primitives
- Covers ALL possible operations
- Nothing redundant

### 2. **Composability**
- RDP = primitives
- Automation = primitives
- AI workflows = primitives
- Infinite combinations

### 3. **Granular Control**
- Users choose exact primitives
- Remove unwanted capabilities
- Principle of least privilege

### 4. **Efficiency**
- Fast bitflag checks for common cases
- String capabilities for compositions
- Best of both worlds

### 5. **AI-Ready**
- Agents compose primitives dynamically
- Register custom workflows
- Unlimited extensibility

---

**This is the foundation for unlimited extensibility!** 🚀

Every possible operation is a combination of these 64 atomic primitives. RDP, automation, AI workflows - they're all just compositions.

