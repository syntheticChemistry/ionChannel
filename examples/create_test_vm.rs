#!/usr/bin/env rust-script
//! Create test VM from template for validation
//! 
//! Usage: cargo run --example create_test_vm --features benchscale -- <vm-name>

use benchscale::{CloudInit, LibvirtBackend};
use std::env;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <vm-name>", args[0]);
        std::process::exit(1);
    }

    let vm_name = &args[1];
    let template = PathBuf::from("/var/lib/libvirt/images/ubuntu-desktop-rustdesk-template.qcow2");

    if !template.exists() {
        eprintln!("❌ Template not found: {}", template.display());
        eprintln!("   Run the build pipeline first");
        std::process::exit(1);
    }

    println!("Creating test VM: {}", vm_name);
    println!("Template: {}", template.display());

    // Get SSH key if available
    let ssh_key = std::env::var("SSH_PUBLIC_KEY")
        .ok()
        .or_else(|| {
            std::fs::read_to_string(format!("{}/.ssh/id_rsa.pub", std::env::var("HOME").unwrap()))
                .ok()
        });

    let cloud_init = CloudInit::builder()
        .add_user("desktop", ssh_key.as_deref().unwrap_or(""))
        .build();

    #[cfg(feature = "libvirt")]
    {
        use benchscale::backend::Backend;
        
        let backend = LibvirtBackend::new()?;
        
        let vm = backend.create_from_template(
            vm_name,
            &template,
            Some(&cloud_init),
            4096,
            2,
            false,
        ).await?;

        println!("✅ VM created successfully");
        println!("   Name: {}", vm.name);
        println!("   IP: {}", vm.ip_address);
        
        // Get VNC display
        let output = std::process::Command::new("virsh")
            .args(&["vncdisplay", vm_name])
            .output()?;
        
        if output.status.success() {
            let vnc = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("   VNC: localhost{}", vnc);
        }
    }

    #[cfg(not(feature = "libvirt"))]
    {
        eprintln!("❌ This example requires the 'libvirt' feature");
        std::process::exit(1);
    }

    Ok(())
}

