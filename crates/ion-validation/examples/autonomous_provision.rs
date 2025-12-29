//! Autonomous VM Provisioning Example
//!
//! This demonstrates the agentic pattern: AI working on behalf of humans
//! to provision a VM with ZERO human interaction.
//!
//! Run with:
//!   cargo run --example autonomous_provision --features libvirt
//!
//! This will:
//! 1. Generate SSH keys automatically
//! 2. Create cloud-init configuration
//! 3. Provision a VM
//! 4. Wait for boot
//! 5. Connect via SSH
//! 6. Run a test command
//! 7. Clean up
//!
//! NO PASSWORDS. NO CONSOLE. NO MANUAL STEPS.

use anyhow::Result;
use ion_deploy::autonomous::{AutonomousProvisionConfig, AutonomousProvisioner};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("\n╔══════════════════════════════════════════════════════════════════════╗");
    println!("║                                                                      ║");
    println!("║              🤖 AUTONOMOUS VM PROVISIONING EXAMPLE                   ║");
    println!("║                                                                      ║");
    println!("║  Demonstrating: AI working on behalf of humans                      ║");
    println!("║  Zero human interaction required                                    ║");
    println!("║                                                                      ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝\n");

    // Configure autonomous provisioner
    let config = AutonomousProvisionConfig {
        vm_name: "ionChannel-autonomous-demo".to_string(),
        ram_mb: 2048,
        vcpus: 2,
        disk_gb: 10,
        username: "ubuntu".to_string(),
        packages: vec!["git".to_string(), "build-essential".to_string()],
        ..Default::default()
    };

    println!("📋 Configuration:");
    println!("   VM Name: {}", config.vm_name);
    println!("   RAM: {} MB", config.ram_mb);
    println!("   vCPUs: {}", config.vcpus);
    println!("   User: {}", config.username);
    println!("   Base Image: {}", config.base_image.display());
    println!();

    // Create provisioner
    let provisioner = AutonomousProvisioner::new(config);

    println!("🚀 Starting autonomous provisioning...\n");

    // Provision autonomously
    match provisioner.provision().await {
        Ok((mut ssh, ip)) => {
            println!("\n✅ AUTONOMOUS PROVISIONING COMPLETE!");
            println!("════════════════════════════════════════════════════════════════");
            println!("   VM IP: {}", ip);
            println!("   SSH: Automatically configured and connected");
            println!("   Keys: Generated and injected automatically");
            println!();

            // Demonstrate autonomous execution
            println!("🧪 Running test command autonomously...");
            match ssh.execute("uname -a && uptime").await {
                Ok(output) => {
                    println!("\n📤 Command output:");
                    println!("{}", output);
                },
                Err(e) => {
                    eprintln!("⚠️  Command failed: {}", e);
                },
            }

            println!("\n🧹 Cleaning up...");
            provisioner.destroy().await?;
            println!("✅ Cleanup complete");

            println!("\n════════════════════════════════════════════════════════════════");
            println!("🎉 DEMONSTRATION COMPLETE!");
            println!();
            println!("This was fully autonomous:");
            println!("  ✅ No passwords entered");
            println!("  ✅ No console interaction");
            println!("  ✅ No manual SSH configuration");
            println!("  ✅ AI working on behalf of humans");
            println!("════════════════════════════════════════════════════════════════\n");
        },
        Err(e) => {
            eprintln!("\n❌ Autonomous provisioning failed: {}", e);
            eprintln!("\n💡 Common issues:");
            eprintln!("   • Base image not found (download Ubuntu cloud image)");
            eprintln!("   • libvirt not running (sudo systemctl start libvirtd)");
            eprintln!("   • Permissions (add user to libvirt group)");
            eprintln!("\n📖 See DEPLOYMENT.md for setup instructions\n");
            return Err(e);
        },
    }

    Ok(())
}
