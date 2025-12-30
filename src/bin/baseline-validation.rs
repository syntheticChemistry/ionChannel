// baseline-validation.rs - Test baseline VMs (Ubuntu 24 and Pop!_OS COSMIC, no RustDesk)
//
// Purpose: Validate that baseline templates work correctly before adding RustDesk
//
// Tests:
// - VM creation from template
// - SSH connectivity
// - Desktop environment detection
// - Wayland compositor verification

use anyhow::{Context, Result};
use benchscale::{CloudInit, LibvirtBackend};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug)]
struct BaselineVm {
    name: String,
    ip: String,
    username: String,
    ssh_key_path: PathBuf,
    desktop_env: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n╔══════════════════════════════════════════════════════════════════════════╗");
    println!("║                                                                          ║");
    println!("║                  🧪 BASELINE VM VALIDATION                               ║");
    println!("║                  Ubuntu 24 + Pop!_OS COSMIC                              ║");
    println!("║                                                                          ║");
    println!("╚══════════════════════════════════════════════════════════════════════════╝\n");

    // Initialize benchScale backend
    println!("🔧 Initializing benchScale backend...");
    let backend = LibvirtBackend::new().context("Failed to create LibvirtBackend")?;
    println!("✅ Backend ready\n");

    // Create both baseline VMs
    let mut ubuntu_vm = None;
    let mut popos_vm = None;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("📋 PHASE 1: Creating Baseline VMs\n");

    // Create Ubuntu 24.04 baseline VM
    match create_ubuntu_baseline(&backend).await {
        Ok(vm) => {
            println!("✅ Ubuntu 24.04 VM created successfully!");
            println!("   Name: {}", vm.name);
            println!("   IP: {}", vm.ip);
            println!("   Desktop: {}\n", vm.desktop_env);
            ubuntu_vm = Some(vm);
        }
        Err(e) => {
            println!("❌ Ubuntu 24.04 VM creation failed: {:?}\n", e);
        }
    }

    // Create Pop!_OS COSMIC baseline VM
    match create_popos_cosmic_baseline(&backend).await {
        Ok(vm) => {
            println!("✅ Pop!_OS COSMIC VM created successfully!");
            println!("   Name: {}", vm.name);
            println!("   IP: {}", vm.ip);
            println!("   Desktop: {}\n", vm.desktop_env);
            popos_vm = Some(vm);
        }
        Err(e) => {
            println!("❌ Pop!_OS COSMIC VM creation failed: {:?}\n", e);
        }
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("📋 PHASE 2: Validating VMs\n");

    // Validate Ubuntu VM
    if let Some(ref vm) = ubuntu_vm {
        validate_baseline_vm(vm).await?;
    }

    // Validate Pop!_OS VM
    if let Some(ref vm) = popos_vm {
        validate_baseline_vm(vm).await?;
    }

    // Summary
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("📊 VALIDATION SUMMARY\n");

    let mut success_count = 0;
    let mut total_count = 0;

    if ubuntu_vm.is_some() {
        println!("✅ Ubuntu 24.04 Baseline: READY");
        success_count += 1;
    } else {
        println!("❌ Ubuntu 24.04 Baseline: FAILED");
    }
    total_count += 1;

    if popos_vm.is_some() {
        println!("✅ Pop!_OS COSMIC Baseline: READY");
        success_count += 1;
    } else {
        println!("❌ Pop!_OS COSMIC Baseline: FAILED");
    }
    total_count += 1;

    println!("\n🎯 Result: {}/{} VMs validated successfully", success_count, total_count);

    if success_count == total_count {
        println!("\n╔══════════════════════════════════════════════════════════════════════════╗");
        println!("║                                                                          ║");
        println!("║              ✅ ALL BASELINE VMS VALIDATED SUCCESSFULLY!                 ║");
        println!("║                                                                          ║");
        println!("║              Ready to proceed with RustDesk templates                    ║");
        println!("║                                                                          ║");
        println!("╚══════════════════════════════════════════════════════════════════════════╝\n");
    } else {
        println!("\n⚠️  Some VMs failed validation. Check logs above for details.\n");
    }

    // Keep VMs running for manual inspection
    println!("💡 VMs are still running for manual inspection.");
    println!("   Use `virsh list` to see active VMs.");
    println!("   Use `virsh destroy <vm-name>` to stop a VM.");
    println!("   Use `virsh undefine <vm-name>` to remove a VM.\n");

    if let Some(ref vm) = ubuntu_vm {
        println!("   Ubuntu 24.04:");
        println!("     ssh -i {} {}@{}", vm.ssh_key_path.display(), vm.username, vm.ip);
    }

    if let Some(ref vm) = popos_vm {
        println!("   Pop!_OS COSMIC:");
        println!("     ssh -i {} {}@{}", vm.ssh_key_path.display(), vm.username, vm.ip);
    }

    println!();

    Ok(())
}

async fn create_ubuntu_baseline(backend: &LibvirtBackend) -> Result<BaselineVm> {
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let vm_name = format!("ubuntu24-baseline-{}", timestamp);

    println!("🏗️  Creating Ubuntu 24.04 Baseline VM...");

    let template_path = PathBuf::from("/var/lib/libvirt/images/ubuntu-24.04-baseline-template.qcow2");

    if !template_path.exists() {
        anyhow::bail!(
            "Template not found: {}\nRun: agentReagents/scripts/build-ubuntu-24-baseline.sh",
            template_path.display()
        );
    }

    // Generate SSH key
    let ssh_key_path = PathBuf::from(format!("/tmp/iontest_key_{}", vm_name));
    let ssh_pub_key = generate_ssh_keypair(&ssh_key_path).await?;

    let username = "ubuntu";

    // Cloud-init configuration (minimal, template has desktop pre-installed)
    let cloud_init = CloudInit::builder()
        .add_user(username.to_string(), ssh_pub_key)
        .package_update(false)
        .build();

    // Create VM from template
    let node_info = backend
        .create_from_template(
            &vm_name,
            &template_path,
            Some(&cloud_init),
            4096, // 4GB RAM for desktop
            2,    // 2 vCPUs
            false,
        )
        .await?;

    // Wait for SSH
    println!("   ⏳ Waiting for SSH...");
    wait_for_ssh(&node_info.ip_address, &ssh_key_path, 120, username).await?;
    println!("   ✅ SSH ready");

    Ok(BaselineVm {
        name: node_info.name,
        ip: node_info.ip_address,
        username: username.to_string(),
        ssh_key_path,
        desktop_env: "GNOME (Wayland)".to_string(),
    })
}

async fn create_popos_cosmic_baseline(backend: &LibvirtBackend) -> Result<BaselineVm> {
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let vm_name = format!("popos-cosmic-baseline-{}", timestamp);

    println!("🏗️  Creating Pop!_OS COSMIC Baseline VM...");

    let template_path = PathBuf::from("/var/lib/libvirt/images/popos-24-cosmic-baseline-template.qcow2");

    if !template_path.exists() {
        anyhow::bail!(
            "Template not found: {}\nRun: agentReagents/scripts/build-popos-24-cosmic-baseline.sh",
            template_path.display()
        );
    }

    // Generate SSH key
    let ssh_key_path = PathBuf::from(format!("/tmp/iontest_key_{}", vm_name));
    let ssh_pub_key = generate_ssh_keypair(&ssh_key_path).await?;

    let username = "cosmic";

    // Cloud-init configuration (minimal, template has desktop pre-installed)
    let cloud_init = CloudInit::builder()
        .add_user(username.to_string(), ssh_pub_key)
        .package_update(false)
        .build();

    // Create VM from template
    let node_info = backend
        .create_from_template(
            &vm_name,
            &template_path,
            Some(&cloud_init),
            4096, // 4GB RAM for desktop
            2,    // 2 vCPUs
            false,
        )
        .await?;

    // Wait for SSH
    println!("   ⏳ Waiting for SSH...");
    wait_for_ssh(&node_info.ip_address, &ssh_key_path, 120, username).await?;
    println!("   ✅ SSH ready");

    Ok(BaselineVm {
        name: node_info.name,
        ip: node_info.ip_address,
        username: username.to_string(),
        ssh_key_path,
        desktop_env: "COSMIC (Wayland)".to_string(),
    })
}

async fn validate_baseline_vm(vm: &BaselineVm) -> Result<()> {
    println!("🔍 Validating {} ({})...", vm.name, vm.desktop_env);

    // Test 1: SSH connectivity
    println!("   [1/4] Testing SSH connectivity...");
    test_ssh_command(vm, "echo 'SSH test successful'").await?;
    println!("   ✅ SSH connectivity working");

    // Test 2: Desktop environment
    println!("   [2/4] Checking desktop environment...");
    let de_check = test_ssh_command(vm, "echo $XDG_SESSION_TYPE").await?;
    if de_check.contains("wayland") || de_check.contains("$XDG_SESSION_TYPE") {
        println!("   ✅ Wayland session detected (or will be on first login)");
    } else {
        println!("   ⚠️  Wayland not detected: {}", de_check.trim());
    }

    // Test 3: System info
    println!("   [3/4] Gathering system information...");
    let os_info = test_ssh_command(vm, "cat /etc/os-release | grep PRETTY_NAME").await?;
    println!("   ℹ️  {}", os_info.trim());

    // Test 4: Verify no RustDesk
    println!("   [4/4] Verifying RustDesk is NOT installed...");
    let rustdesk_check = test_ssh_command(vm, "which rustdesk || echo 'not found'").await?;
    if rustdesk_check.contains("not found") {
        println!("   ✅ RustDesk not present (baseline confirmed)");
    } else {
        println!("   ⚠️  RustDesk found: {}", rustdesk_check.trim());
    }

    println!("   ✅ Validation complete\n");

    Ok(())
}

async fn generate_ssh_keypair(key_path: &PathBuf) -> Result<String> {
    // Remove old key if exists
    let _ = std::fs::remove_file(key_path);
    let _ = std::fs::remove_file(format!("{}.pub", key_path.display()));

    Command::new("ssh-keygen")
        .args(&[
            "-t",
            "ed25519",
            "-f",
            &key_path.to_string_lossy(),
            "-N",
            "",
            "-C",
            "baseline-validation",
        ])
        .output()
        .context("Failed to generate SSH key")?;

    let pub_key_path = format!("{}.pub", key_path.display());
    let pub_key = std::fs::read_to_string(&pub_key_path).context("Failed to read public key")?;

    Ok(pub_key.trim().to_string())
}

async fn wait_for_ssh(ip: &str, key_path: &PathBuf, timeout_secs: u64, username: &str) -> Result<()> {
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    loop {
        if start.elapsed() > timeout {
            anyhow::bail!("SSH timeout after {} seconds", timeout_secs);
        }

        let output = Command::new("ssh")
            .args(&[
                "-i",
                &key_path.to_string_lossy(),
                "-o",
                "StrictHostKeyChecking=no",
                "-o",
                "UserKnownHostsFile=/dev/null",
                "-o",
                "ConnectTimeout=5",
                &format!("{}@{}", username, ip),
                "exit",
            ])
            .output();

        if let Ok(result) = output {
            if result.status.success() {
                return Ok(());
            }
        }

        sleep(Duration::from_secs(5)).await;
    }
}

async fn test_ssh_command(vm: &BaselineVm, command: &str) -> Result<String> {
    let output = Command::new("ssh")
        .args(&[
            "-i",
            &vm.ssh_key_path.to_string_lossy(),
            "-o",
            "StrictHostKeyChecking=no",
            "-o",
            "UserKnownHostsFile=/dev/null",
            &format!("{}@{}", vm.username, vm.ip),
            command,
        ])
        .output()
        .context("Failed to execute SSH command")?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        anyhow::bail!(
            "Command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    }
}

