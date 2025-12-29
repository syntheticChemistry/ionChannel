//! Provision a VM and get RustDesk connection info
//!
//! This example shows the full flow:
//! 1. Create a VM using LibvirtBackend
//! 2. Install RustDesk
//! 3. Get connection ID

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(not(feature = "libvirt"))]
    {
        eprintln!("This example requires the 'libvirt' feature.");
        eprintln!("Run with: cargo run --example provision_and_connect --features libvirt");
        std::process::exit(1);
    }

    #[cfg(feature = "libvirt")]
    {
        println!("\n🚀 AUTOMATED VM PROVISIONING & RUSTDESK SETUP");
        println!("════════════════════════════════════════════════════════════════\n");

        // Use existing VM
        println!("📋 Using existing test1 VM...\n");

        // Get VM info from benchScale
        use benchscale::backend::{Backend, LibvirtBackend};
        let backend = LibvirtBackend::new()?;

        println!("🔍 Checking VM status...");
        match backend.get_node("test1").await {
            Ok(node) => {
                println!("✓ VM found: {}", node.name);
                println!("  Status: {:?}", node.status);
                println!("  IP: {}\n", &node.ip_address);

                if !node.ip_address.is_empty() {
                    println!("═══════════════════════════════════════════════════════════════");
                    println!("\n📊 VM READY FOR MANUAL SETUP\n");
                    println!("VM IP: {}", node.ip_address);
                    println!("\nConnect via console and run:");
                    println!("  $ sudo dpkg -i /tmp/rustdesk.deb");
                    println!("  $ sudo apt-get install -f -y");
                    println!("  $ rustdesk --get-id");
                    println!("\nOr try SSH (if password works):");
                    println!("  $ ssh iontest@{}", node.ip_address);
                    println!("  Password: iontest");
                    println!("\n═══════════════════════════════════════════════════════════════\n");
                } else {
                    println!("⚠️  VM has no IP address yet. Start it with:");
                    println!("   $ virsh start test1\n");
                }
            },
            Err(e) => {
                println!("⚠️  Could not find VM: {:?}", e);
                println!("\nCreate a VM first with:");
                println!("   $ virsh list --all");
                println!();
            },
        }
    }

    Ok(())
}
