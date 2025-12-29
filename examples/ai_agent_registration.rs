//! AI Agent Capability Registration Example
//!
//! Demonstrates:
//! - Registering custom AI agent capabilities at runtime
//! - Using pattern matching to discover capabilities
//! - Subscribing to capability events
//! - Creating grants with dynamic capabilities
//!
//! Run with:
//! ```bash
//! cargo run --example ai_agent_registration
//! ```

use ion_core::capabilities::{
    Capability, CapabilityDefinition, CapabilityEvent, CapabilityGrant, CapabilityProvider,
    CapabilityRegistry, CoreCapabilities, RiskLevel,
};
use std::time::SystemTime;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AI Agent Capability Registration Example ===\n");

    // 1. Create a capability registry
    println!("1. Creating capability registry...");
    let registry = CapabilityRegistry::new();
    println!("   ✓ Registry initialized");

    // 2. Subscribe to capability events
    println!("\n2. Subscribing to capability events...");
    let mut event_rx = registry.subscribe();

    // Spawn a task to listen for events
    tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            match event {
                CapabilityEvent::Registered(def) => {
                    println!("   📢 Event: Capability registered - {}", def.capability);
                },
                CapabilityEvent::Unregistered(def) => {
                    println!("   📢 Event: Capability unregistered - {}", def.capability);
                },
                CapabilityEvent::Deprecated(cap, info) => {
                    println!(
                        "   📢 Event: Capability deprecated - {} (reason: {})",
                        cap, info.reason
                    );
                },
            }
        }
    });

    // Give the task time to start
    sleep(Duration::from_millis(10)).await;

    // 3. Register Claude vision capability
    println!("\n3. Registering Claude vision capability...");
    let claude_vision = CapabilityDefinition {
        capability: Capability::parse("ai.claude.vision")?,
        display_name: "Claude Vision Analysis".to_string(),
        description: "Analyze screen content using Claude AI's vision model".to_string(),
        risk_level: RiskLevel::Medium,
        provider: CapabilityProvider::Agent {
            id: "claude-desktop".to_string(),
        },
        registered_at: SystemTime::now(),
        deprecated: None,
        requires: vec![Capability::parse("ion.screen.read")?],
    };
    registry.register(claude_vision).await?;
    println!("   ✓ Registered: ai.claude.vision");

    // 4. Register GPT automation capability
    println!("\n4. Registering GPT automation capability...");
    let gpt_automation = CapabilityDefinition {
        capability: Capability::parse("ai.gpt.automation")?,
        display_name: "GPT Desktop Automation".to_string(),
        description: "Automate desktop tasks using GPT".to_string(),
        risk_level: RiskLevel::High,
        provider: CapabilityProvider::Agent {
            id: "openai-desktop".to_string(),
        },
        registered_at: SystemTime::now(),
        deprecated: None,
        requires: vec![
            Capability::parse("ion.input.keyboard")?,
            Capability::parse("ion.input.pointer")?,
            Capability::parse("ion.window.control")?,
        ],
    };
    registry.register(gpt_automation).await?;
    println!("   ✓ Registered: ai.gpt.automation");

    // 5. Register custom plugin capability
    println!("\n5. Registering custom plugin capability...");
    let plugin_cap = CapabilityDefinition {
        capability: Capability::parse("plugin.screenshot.annotate")?,
        display_name: "Screenshot Annotation".to_string(),
        description: "Annotate screenshots with text and shapes".to_string(),
        risk_level: RiskLevel::Low,
        provider: CapabilityProvider::Plugin {
            name: "screenshot-tools".to_string(),
        },
        registered_at: SystemTime::now(),
        deprecated: None,
        requires: vec![],
    };
    registry.register(plugin_cap).await?;
    println!("   ✓ Registered: plugin.screenshot.annotate");

    // Give events time to propagate
    sleep(Duration::from_millis(50)).await;

    // 6. Find all AI capabilities
    println!("\n6. Finding all AI capabilities...");
    let ai_caps = registry.find_matching("ai.*.*").await?;
    println!("   ✓ Found {} AI capabilities:", ai_caps.len());
    for cap in &ai_caps {
        println!("      - {}", cap);
    }

    // 7. Find capabilities by namespace
    println!("\n7. Listing capabilities by namespace...");
    let plugin_caps = registry.list_namespace("plugin").await;
    println!("   ✓ Plugin namespace: {} capabilities", plugin_caps.len());
    for cap in &plugin_caps {
        println!("      - {}", cap);
    }

    // 8. Get capability details
    println!("\n8. Getting capability details...");
    let claude_cap = Capability::parse("ai.claude.vision")?;
    let claude_def = registry.get(&claude_cap).await?;
    println!("   ✓ Name: {}", claude_def.display_name);
    println!("   ✓ Description: {}", claude_def.description);
    println!("   ✓ Risk Level: {:?}", claude_def.risk_level);
    println!("   ✓ Provider: {:?}", claude_def.provider);
    println!("   ✓ Requires {} dependencies", claude_def.requires.len());

    // 9. Create a grant with dynamic capabilities
    println!("\n9. Creating grant with dynamic capabilities...");
    let mut grant = CapabilityGrant::new(
        "claude-desktop".to_string(),
        CoreCapabilities::CAP_SCREEN_READ, // Basic screen reading
    );

    // Add the dynamic AI capability
    grant.add_dynamic("ai.claude.vision".to_string());
    println!("   ✓ Grant created with:");
    println!("      - Primitive capabilities: CAP_SCREEN_READ");
    println!("      - Dynamic capabilities: ai.claude.vision");

    // 10. Check dynamic capability
    println!("\n10. Checking dynamic capability...");
    if grant.has_dynamic("ai.claude.vision") {
        println!("   ✓ Grant has ai.claude.vision capability");
    }

    // 11. Demonstrate versioning
    println!("\n11. Demonstrating capability versioning...");
    let versioned_cap = Capability::parse("ai.claude.vision@v2")?;
    println!("   ✓ Parsed: {}", versioned_cap);
    if let Some(version) = &versioned_cap.version {
        println!("   ✓ Version: {}", version);
    }

    // 12. Show registry statistics
    println!("\n12. Registry statistics...");
    let total_caps = registry.count().await;
    println!("   ✓ Total registered capabilities: {}", total_caps);

    println!("\n=== Example Complete ===");
    println!("\nKey Takeaways:");
    println!("  • AI agents can register unlimited custom capabilities");
    println!("  • Capabilities use namespace.domain.action format");
    println!("  • Pattern matching enables capability discovery");
    println!("  • Events notify when capabilities are registered/unregistered");
    println!("  • Grants can combine primitive + dynamic capabilities");

    Ok(())
}
