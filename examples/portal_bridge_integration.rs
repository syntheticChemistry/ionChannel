//! Portal Bridge Integration Example
//!
//! Demonstrates:
//! - Using the bridge to check portal method permissions
//! - Converting between D-Bus types and capabilities
//! - Getting human-readable method descriptions
//! - Finding all methods enabled by capabilities
//!
//! Run with:
//! ```bash
//! cargo run --example portal_bridge_integration
//! ```

use ion_core::capabilities::{
    CapabilityGrant, CapabilityPortalBridge, CoreCapabilities, PortalMethod,
    PortalMethodDescriptions, COMPOSITION_RDP,
};

fn main() {
    println!("=== Portal Bridge Integration Example ===\n");

    // 1. Create a bridge
    println!("1. Creating portal bridge...");
    let bridge = CapabilityPortalBridge::new();
    println!("   ✓ Bridge initialized with standard mappings");

    // 2. Create a grant with RDP capabilities
    println!("\n2. Creating RDP capability grant...");
    let grant = CapabilityGrant::new("org.rustdesk.RustDesk".to_string(), COMPOSITION_RDP);
    println!("   ✓ Grant created for RustDesk");
    println!(
        "   ✓ Capabilities: {}",
        grant.primitives.bits().count_ones()
    );

    // 3. Check specific portal method permissions
    println!("\n3. Checking portal method permissions...");

    let keyboard_method = PortalMethod::new(
        "org.freedesktop.portal.RemoteDesktop",
        "NotifyKeyboardKeycode",
    );
    let allowed = bridge.check_method(grant.primitives, &keyboard_method);
    println!(
        "   ✓ NotifyKeyboardKeycode: {}",
        if allowed { "ALLOWED" } else { "DENIED" }
    );

    let pointer_method = PortalMethod::new(
        "org.freedesktop.portal.RemoteDesktop",
        "NotifyPointerMotion",
    );
    let allowed = bridge.check_method(grant.primitives, &pointer_method);
    println!(
        "   ✓ NotifyPointerMotion: {}",
        if allowed { "ALLOWED" } else { "DENIED" }
    );

    let unknown_method = PortalMethod::new("org.example.UnknownPortal", "UnknownMethod");
    let allowed = bridge.check_method(grant.primitives, &unknown_method);
    println!(
        "   ✓ UnknownMethod: {} (unknown methods denied by default)",
        if allowed { "ALLOWED" } else { "DENIED" }
    );

    // 4. Get all enabled methods
    println!("\n4. Finding all enabled portal methods...");
    let enabled_methods = bridge.get_enabled_methods(grant.primitives);
    println!(
        "   ✓ Grant enables {} portal methods:",
        enabled_methods.len()
    );
    for (i, method) in enabled_methods.iter().take(5).enumerate() {
        let desc = PortalMethodDescriptions::describe(method);
        println!("      {}. {} - {}", i + 1, method.method, desc);
    }
    if enabled_methods.len() > 5 {
        println!("      ... and {} more", enabled_methods.len() - 5);
    }

    // 5. Convert D-Bus device types to capabilities
    println!("\n5. Converting D-Bus device types to capabilities...");

    // Device types: Keyboard (1) | Pointer (2) | Touchscreen (4)
    let device_types_bits = 3u32; // Keyboard | Pointer
    let caps = CapabilityPortalBridge::device_types_to_caps(device_types_bits);
    println!("   ✓ D-Bus device_types=3 (Keyboard|Pointer) →");
    if caps.contains(CoreCapabilities::CAP_KEYBOARD_WRITE) {
        println!("      - CAP_KEYBOARD_WRITE");
    }
    if caps.contains(CoreCapabilities::CAP_POINTER_WRITE) {
        println!("      - CAP_POINTER_WRITE");
    }

    // 6. Convert capabilities to D-Bus device types
    println!("\n6. Converting capabilities to D-Bus device types...");
    let caps = CoreCapabilities::CAP_KEYBOARD_WRITE
        | CoreCapabilities::CAP_POINTER_WRITE
        | CoreCapabilities::CAP_TOUCH_WRITE;
    let device_types = CapabilityPortalBridge::caps_to_device_types(caps);
    println!("   ✓ Capabilities → D-Bus device_types={}", device_types);
    println!("      Binary: {:03b}", device_types);
    println!(
        "      - Bit 0 (Keyboard): {}",
        if device_types & 1 != 0 {
            "SET"
        } else {
            "UNSET"
        }
    );
    println!(
        "      - Bit 1 (Pointer): {}",
        if device_types & 2 != 0 {
            "SET"
        } else {
            "UNSET"
        }
    );
    println!(
        "      - Bit 2 (Touch): {}",
        if device_types & 4 != 0 {
            "SET"
        } else {
            "UNSET"
        }
    );

    // 7. Convert D-Bus source types to capabilities
    println!("\n7. Converting D-Bus source types to capabilities...");

    // Source types: Monitor (1) | Window (2) | Virtual (4)
    let source_types_bits = 6u32; // Window | Virtual
    let caps = CapabilityPortalBridge::source_types_to_caps(source_types_bits);
    println!("   ✓ D-Bus source_types=6 (Window|Virtual) →");
    if caps.contains(CoreCapabilities::CAP_SCREEN_WINDOW) {
        println!("      - CAP_SCREEN_WINDOW");
    }
    if caps.contains(CoreCapabilities::CAP_SCREEN_FULL) {
        println!("      - CAP_SCREEN_FULL");
    }

    // 8. Get required capabilities for a method
    println!("\n8. Getting required capabilities for portal methods...");
    let method = PortalMethod::new(
        "org.freedesktop.portal.RemoteDesktop",
        "NotifyPointerButton",
    );
    if let Some(required) = bridge.get_required_caps(&method) {
        println!("   ✓ NotifyPointerButton requires:");
        if required.contains(CoreCapabilities::CAP_POINTER_WRITE) {
            println!("      - CAP_POINTER_WRITE");
        }
    }

    // 9. Round-trip conversion test
    println!("\n9. Testing round-trip conversions...");
    let original_device = 5u32; // Keyboard (1) | Touch (4)
    let caps = CapabilityPortalBridge::device_types_to_caps(original_device);
    let converted = CapabilityPortalBridge::caps_to_device_types(caps);
    println!(
        "   ✓ Device types: {} → capabilities → {}",
        original_device, converted
    );
    assert_eq!(
        original_device, converted,
        "Round-trip conversion should be lossless"
    );
    println!("   ✓ Round-trip successful!");

    // 10. Practical usage example
    println!("\n10. Practical usage in portal handler...");
    println!("   Code example:");
    println!("   ```rust");
    println!("   // In RemoteDesktop portal implementation");
    println!("   async fn notify_keyboard_keycode(&self, keycode: i32) {{");
    println!("       let method = PortalMethod::new(");
    println!("           \"org.freedesktop.portal.RemoteDesktop\",");
    println!("           \"NotifyKeyboardKeycode\"");
    println!("       );");
    println!();
    println!("       // Check if grant allows this method");
    println!("       if !bridge.check_method(grant.primitives, &method) {{");
    println!("           return Err(AccessDenied);");
    println!("       }}");
    println!();
    println!("       // Forward to backend");
    println!("       backend.send_keycode(keycode).await");
    println!("   }}");
    println!("   ```");

    println!("\n=== Example Complete ===");
    println!("\nKey Takeaways:");
    println!("  • Bridge enables clean integration with D-Bus portals");
    println!("  • Bidirectional mapping: Capabilities ↔ Portal methods");
    println!("  • Device/source type conversion is automatic");
    println!("  • Unknown methods are denied by default (secure)");
    println!("  • Round-trip conversions are lossless");
}
