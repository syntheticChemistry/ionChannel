//! Basic Capability Grant Example
//!
//! Demonstrates:
//! - Creating a capability grant for an application
//! - Setting restrictions (rate limits, expiration)
//! - Checking permissions
//! - Recording usage
//! - Storing grants persistently
//!
//! Run with:
//! ```bash
//! cargo run --example basic_capability_grant
//! ```

use ion_core::capabilities::{
    CapabilityGrant, CoreCapabilities, FileSystemGrantStorage, GrantStorage, COMPOSITION_RDP,
};
use std::env;
use std::time::{Duration, SystemTime};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic Capability Grant Example ===\n");

    // 1. Create a basic grant for RustDesk with RDP capabilities
    println!("1. Creating capability grant for RustDesk...");
    let mut grant = CapabilityGrant::new(
        "org.rustdesk.RustDesk".to_string(),
        COMPOSITION_RDP, // Full remote desktop capabilities
    );
    println!("   ✓ Grant ID: {}", grant.id);
    println!("   ✓ App: {}", grant.app_id);
    println!(
        "   ✓ Capabilities: {} primitives",
        grant.primitives.bits().count_ones()
    );

    // 2. Add restrictions
    println!("\n2. Adding restrictions...");
    grant.restrictions.max_events_per_second = Some(100);
    grant.restrictions.expires_at = Some(SystemTime::now() + Duration::from_secs(3600));
    println!("   ✓ Rate limit: 100 events/sec");
    println!("   ✓ Expires in: 1 hour");

    // 3. Check specific permissions
    println!("\n3. Checking permissions...");
    let can_keyboard = grant.has_primitive(CoreCapabilities::CAP_KEYBOARD_WRITE);
    let can_pointer = grant.has_primitive(CoreCapabilities::CAP_POINTER_WRITE);
    let can_screen = grant.has_primitive(CoreCapabilities::CAP_SCREEN_READ);
    println!("   ✓ Can send keyboard input: {}", can_keyboard);
    println!("   ✓ Can send pointer input: {}", can_pointer);
    println!("   ✓ Can read screen: {}", can_screen);

    // 4. Simulate usage
    println!("\n4. Simulating usage...");
    for _ in 0..5 {
        if grant.has_primitive(CoreCapabilities::CAP_KEYBOARD_WRITE) {
            grant.record_use();
            grant.usage.total_events += 10;
        }
    }
    println!("   ✓ Total uses: {}", grant.usage.use_count);
    println!("   ✓ Total events: {}", grant.usage.total_events);
    if let Some(last_used) = grant.usage.last_used {
        println!("   ✓ Last used: {:?}", last_used);
    }

    // 5. Store grant persistently
    println!("\n5. Storing grant persistently...");
    let storage_path = env::temp_dir().join("ionChannel_example_grants");
    let storage = FileSystemGrantStorage::new(&storage_path).await?;
    storage.save(&grant).await?;
    println!("   ✓ Saved to: {}", storage_path.display());

    // 6. Load grant back
    println!("\n6. Loading grant from storage...");
    let loaded_grant = storage.load(&grant.id).await?;
    println!("   ✓ Loaded grant ID: {}", loaded_grant.id);
    println!("   ✓ App: {}", loaded_grant.app_id);
    println!("   ✓ Use count preserved: {}", loaded_grant.usage.use_count);

    // 7. List all grants for the app
    println!("\n7. Listing all grants for app...");
    let app_grants = storage.load_for_app("org.rustdesk.RustDesk").await?;
    println!("   ✓ Found {} grant(s)", app_grants.len());

    // 8. Check expiration
    println!("\n8. Checking expiration status...");
    if grant.is_expired() {
        println!("   ⚠ Grant has expired");
    } else {
        println!("   ✓ Grant is still valid");
    }

    println!("\n=== Example Complete ===");
    Ok(())
}
