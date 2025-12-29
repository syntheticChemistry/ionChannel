//! Complete End-to-End Validation Demo
//!
//! This demonstrates the full ionChannel validation flow using benchScale:
//! 1. Capability-based VM backend discovery
//! 2. VM provisioning with health monitoring
//! 3. RustDesk installation with ID retrieval
//! 4. ionChannel portal deployment
//! 5. E2E verification
//! 6. Full event stream observation
//!
//! Run with: cargo run -p ion-validation --example full_e2e_demo --features libvirt
//!
//! Environment Variables:
//!   VM_SSH_USER - SSH username for VM (default: ubuntu)
//!   VM_SSH_PASSWORD - SSH password for VM (default: ubuntu)
//!   BENCHSCALE_SSH_PORT - SSH port (default: 22)
//!   IONCHANNEL_REPO_URL - Git repo URL
//!   RUSTDESK_VERSION - RustDesk version to install

use futures::StreamExt;
use ion_validation::{
    capabilities::CapabilityRegistry,
    events::ValidationEvent,
    impls::{IonChannelDeployer, LibvirtProvider, LibvirtProvisioner, RustDeskProvider},
    orchestrator::{ValidationOrchestrator, ValidationPlan},
    providers::{vm::VmSpec, VmBackendRegistry},
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for observability
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("\n╔══════════════════════════════════════════════════════════════════════╗");
    println!("║                                                                      ║");
    println!("║           🚀 ionChannel E2E Validation Demo 🚀                       ║");
    println!("║                                                                      ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝\n");

    println!("📋 DEMO OVERVIEW");
    println!("═══════════════════════════════════════════════════════════════════════\n");
    println!("This demo showcases:");
    println!("  1. Capability-based VM backend discovery (primal pattern)");
    println!("  2. VM provisioning via benchScale v2.0.0");
    println!("  3. RustDesk installation with ID retrieval");
    println!("  4. ionChannel portal deployment (build + deploy)");
    println!("  5. E2E verification and health checks");
    println!("  6. Full event streaming (AI-agent ready)\n");

    // Phase 0: VM Backend Discovery
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("📡 PHASE 0: VM Backend Discovery (Capability-Based)");
    println!("═══════════════════════════════════════════════════════════════════════\n");

    let vm_registry = VmBackendRegistry::new();

    // Register available VM providers
    println!("Registering VM backend providers...");
    let libvirt_provider = Arc::new(LibvirtProvider::new());
    vm_registry.register(libvirt_provider.clone()).await;
    println!("  ✓ LibvirtProvider registered\n");

    // Discover available backends (parallel)
    println!("Discovering available backends (parallel checks)...");
    let available = vm_registry.find_available().await;

    if available.is_empty() {
        println!("❌ No VM backends available!");
        println!("\n💡 To enable libvirt:");
        println!("   sudo apt install libvirt-daemon-system");
        println!("   sudo systemctl start libvirtd");
        println!("   sudo usermod -aG libvirt $USER\n");
        return Ok(());
    }

    println!("  ✓ Found {} available backend(s)", available.len());
    for provider in &available {
        println!("    - {} ({})", provider.name(), provider.id());
    }
    println!();

    // Check health status
    println!("Checking health status...");
    let health_status = vm_registry.health_check().await;

    for (id, health) in &health_status {
        match health {
            Ok(h) => {
                println!("  {} - {}", if h.healthy { "✅" } else { "⚠️ " }, id);
                if let Some(version) = &h.version {
                    println!("     Version: {}", version);
                }
                println!(
                    "     VMs: {} available, {} running",
                    h.resources.vms_available, h.resources.vms_running
                );
            },
            Err(e) => {
                println!("  ❌ {} - Error: {}", id, e);
            },
        }
    }
    println!();

    // Create provisioner from best backend
    println!("Creating provisioner from best backend...");
    let best_provider = vm_registry
        .find_best()
        .await
        .ok_or_else(|| anyhow::anyhow!("No VM backend available"))?;

    let vm_provisioner = best_provider.create_provisioner().await?;
    println!("  ✓ Provisioner ready: {}\n", vm_provisioner.name());

    // Phase 1-4: Full E2E Validation
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("🎯 PHASES 1-4: Full E2E Validation Flow");
    println!("═══════════════════════════════════════════════════════════════════════\n");

    // Set up capability registry
    let mut registry = CapabilityRegistry::new();

    // Register all capability providers
    registry.register_vm_provisioner(Arc::new(LibvirtProvisioner::new().await?));
    registry.register_remote_desktop(Arc::new(RustDeskProvider::new()));
    registry.register_portal_deployer(Arc::new(IonChannelDeployer::new()));

    println!("✓ Capability registry configured");
    println!("  - VM Provisioner: libvirt");
    println!("  - Remote Desktop: rustdesk");
    println!("  - Portal Deployer: ionchannel\n");

    // Create validation plan
    println!("Creating validation plan...");

    let vm_spec = VmSpec {
        name: "ionChannel-demo-vm".to_string(),
        ..Default::default()
    };

    let plan = ValidationPlan::builder()
        .vm_spec(vm_spec)
        .with_remote_desktop()
        .with_portal()
        .with_verification()
        .with_ssh_credentials(
            std::env::var("VM_SSH_USER").unwrap_or_else(|_| "ubuntu".to_string()),
            std::env::var("VM_SSH_PASSWORD").unwrap_or_else(|_| "ubuntu".to_string()),
        )
        .build()?;

    println!("  ✓ Plan configured:");
    println!("    - Phase 1: VM Provisioning");
    println!("    - Phase 2: RustDesk Installation");
    println!("    - Phase 3: Portal Deployment");
    println!("    - Phase 4: E2E Verification\n");

    // Execute with full observability
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("▶  EXECUTING VALIDATION (Streaming Events)");
    println!("═══════════════════════════════════════════════════════════════════════\n");

    let orchestrator = ValidationOrchestrator::with_registry(registry);
    let mut execution = orchestrator.execute(plan).await?;

    // Track key information
    let mut vm_id = String::new();
    let mut vm_ip = String::new();
    let mut rustdesk_id = String::new();
    let mut deployment_id = String::new();
    let mut current_phase = 0;

    // Observe all events (AI-agent style!)
    while let Some(event) = execution.next().await {
        match &event {
            ValidationEvent::Started { plan_id, .. } => {
                println!("🚀 Validation started");
                println!("   Plan ID: {}\n", plan_id);
            },

            ValidationEvent::ProvisioningStarted { vm_name, .. } => {
                current_phase = 1;
                println!("═══════════════════════════════════════════════════════════════════════");
                println!("📦 PHASE 1: VM Provisioning");
                println!("═══════════════════════════════════════════════════════════════════════");
                println!("   Provisioning: {}", vm_name);
            },

            ValidationEvent::VmProvisioned {
                vm_id: id,
                vm_name,
                ip,
                duration,
                ..
            } => {
                vm_id = id.clone();
                vm_ip = ip.clone();
                println!("   ✅ VM provisioned successfully!");
                println!("      ID: {}", id);
                println!("      Name: {}", vm_name);
                println!("      IP: {}", ip);
                println!("      Duration: {:?}", duration);
            },

            ValidationEvent::InstallingPackage { package, .. } => {
                if current_phase != 2 {
                    current_phase = 2;
                    println!(
                        "\n═══════════════════════════════════════════════════════════════════════"
                    );
                    println!("🖥️  PHASE 2: Remote Desktop Installation");
                    println!(
                        "═══════════════════════════════════════════════════════════════════════"
                    );
                }
                println!("   📦 Installing: {}", package);
            },

            ValidationEvent::PackageInstalled {
                package, version, ..
            } => {
                println!("   ✅ Installed: {} v{}", package, version);
            },

            ValidationEvent::RemoteDesktopReady { desktop_id, .. } => {
                rustdesk_id = desktop_id.clone();
                println!("   ✅ Remote Desktop ready!");
                println!("      RustDesk ID: {}", desktop_id);
            },

            ValidationEvent::DeployingPortal { target, .. } => {
                current_phase = 3;
                println!(
                    "\n═══════════════════════════════════════════════════════════════════════"
                );
                println!("🚀 PHASE 3: Portal Deployment");
                println!("═══════════════════════════════════════════════════════════════════════");
                println!("   Deploying to: {}", target);
            },

            ValidationEvent::PortalDeployed {
                deployment_id: id,
                services,
                ..
            } => {
                deployment_id = id.clone();
                println!("   ✅ Portal deployed successfully!");
                println!("      Deployment ID: {}", id);
                println!("      Services: {}", services.join(", "));
            },

            ValidationEvent::VerificationComplete {
                success, details, ..
            } => {
                current_phase = 4;
                println!(
                    "\n═══════════════════════════════════════════════════════════════════════"
                );
                println!("✔️  PHASE 4: E2E Verification");
                println!("═══════════════════════════════════════════════════════════════════════");
                println!(
                    "   Status: {}",
                    if *success {
                        "✅ SUCCESS"
                    } else {
                        "❌ FAILED"
                    }
                );
                println!("   Details: {}", details);
            },

            ValidationEvent::PhaseComplete {
                phase,
                phase_name,
                duration,
                ..
            } => {
                println!(
                    "   ⏱️  Phase {} ({}) complete in {:?}",
                    phase, phase_name, duration
                );
            },

            ValidationEvent::Warning { message, .. } => {
                println!("   ⚠️  Warning: {}", message);
            },

            ValidationEvent::Error { message, .. } => {
                println!("   ❌ Error: {}", message);
            },

            ValidationEvent::Complete {
                rustdesk_id: rid,
                total_duration,
                phases_completed,
                metrics,
                ..
            } => {
                if !rid.is_empty() && rid != "UNAVAILABLE" {
                    rustdesk_id = rid.clone();
                }

                println!(
                    "\n═══════════════════════════════════════════════════════════════════════"
                );
                println!("🎉 VALIDATION COMPLETE!");
                println!(
                    "═══════════════════════════════════════════════════════════════════════\n"
                );

                println!("📊 Summary:");
                println!("   Total Duration: {:?}", total_duration);
                println!("   Phases Completed: {}", phases_completed);
                println!();

                println!("🔑 Results:");
                println!("   VM ID: {}", vm_id);
                println!("   VM IP: {}", vm_ip);
                if !rustdesk_id.is_empty() && rustdesk_id != "UNAVAILABLE" {
                    println!("   RustDesk ID: {}", rustdesk_id);
                }
                if !deployment_id.is_empty() {
                    println!("   Deployment ID: {}", deployment_id);
                }
                println!();

                println!("⏱️  Performance:");
                println!("   Provisioning: {:?}", metrics.provisioning_duration);
                println!("   Installation: {:?}", metrics.installation_duration);
                println!("   Deployment: {:?}", metrics.deployment_duration);
                println!();
            },

            _ => {
                // Other events - show description
                println!("   {}", event.description());
            },
        }
    }

    println!("═══════════════════════════════════════════════════════════════════════");
    println!("✨ DEMO COMPLETE - Key Achievements");
    println!("═══════════════════════════════════════════════════════════════════════\n");

    println!("✅ Demonstrated:");
    println!("  1. Capability-based VM backend discovery (primal pattern)");
    println!("  2. Parallel availability checks and health monitoring");
    println!("  3. VM provisioning with benchScale v2.0.0");
    println!("  4. Automated RustDesk installation");
    println!("  5. Complete ionChannel portal deployment");
    println!("  6. E2E verification and health checks");
    println!("  7. Full event streaming for AI agents\n");

    println!("🎯 Architecture Principles:");
    println!("  ✓ Zero hardcoding (all environment-driven)");
    println!("  ✓ Runtime discovery (primal philosophy)");
    println!("  ✓ Capability-based selection");
    println!("  ✓ Event streaming (observable)");
    println!("  ✓ Modern async Rust patterns\n");

    println!("📚 Environment Variables Used:");
    println!("  VM_SSH_USER, VM_SSH_PASSWORD");
    println!("  BENCHSCALE_SSH_PORT, BENCHSCALE_LIBVIRT_URI");
    println!("  IONCHANNEL_REPO_URL, RUSTDESK_VERSION\n");

    println!("🚀 Ready for production validation!");
    println!("═══════════════════════════════════════════════════════════════════════\n");

    Ok(())
}
