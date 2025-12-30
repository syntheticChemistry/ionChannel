// rustdesk-validation.rs - Create VM with RustDesk for Wayland issue validation
//
// Purpose: Spin up a VM with RustDesk to allow user to test remote connection
//          and confirm the "connects but doesn't render" Wayland issue
//
// Workflow:
// 1. Create VM from Ubuntu 24.04 + RustDesk template
// 2. Wait for boot and SSH
// 3. Extract RustDesk ID from running service
// 4. Display connection instructions for user

use anyhow::{Context, Result};
use benchscale::{CloudInit, LibvirtBackend};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug)]
struct RustDeskVm {
    name: String,
    ip: String,
    username: String,
    ssh_key_path: PathBuf,
    rustdesk_id: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n╔══════════════════════════════════════════════════════════════════════════╗");
    println!("║                                                                          ║");
    println!("║                  🖥️  RUSTDESK WAYLAND VALIDATION                         ║");
    println!("║                  Reproduce the Wayland Remote Desktop Issue             ║");
    println!("║                                                                          ║");
    println!("╚══════════════════════════════════════════════════════════════════════════╝\n");

    println!("📋 Purpose: Create a VM with RustDesk to test Wayland remote desktop");
    println!("   Expected: Connection works, but screen doesn't render (black)\n");

    // Initialize benchScale backend
    println!("🔧 Initializing benchScale backend...");
    let backend = LibvirtBackend::new().context("Failed to create LibvirtBackend")?;
    println!("✅ Backend ready\n");

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Create RustDesk VM
    let mut vm = create_rustdesk_vm(&backend).await?;

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Extract RustDesk ID
    println!("🔍 Extracting RustDesk ID...\n");
    vm.rustdesk_id = extract_rustdesk_id(&vm).await?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Display connection instructions
    display_connection_instructions(&vm)?;

    Ok(())
}

async fn create_rustdesk_vm(backend: &LibvirtBackend) -> Result<RustDeskVm> {
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let vm_name = format!("rustdesk-wayland-test-{}", timestamp);

    println!("🏗️  Creating Ubuntu 24.04 + RustDesk VM...");
    println!("   VM Name: {}", vm_name);

    let template_path = PathBuf::from("/var/lib/libvirt/images/ubuntu-24.04-rustdesk-template.qcow2");

    if !template_path.exists() {
        anyhow::bail!(
            "Template not found: {}\nRun: cd agentReagents && sudo ./scripts/build-ubuntu-24-rustdesk.sh",
            template_path.display()
        );
    }

    println!("   ✅ Template: {}", template_path.display());

    // Generate SSH key
    let ssh_key_path = PathBuf::from(format!("/tmp/iontest_key_{}", vm_name));
    let ssh_pub_key = generate_ssh_keypair(&ssh_key_path).await?;

    let username = "ubuntu";

    // Cloud-init configuration (minimal)
    let cloud_init = CloudInit::builder()
        .add_user(username.to_string(), ssh_pub_key)
        .package_update(false)
        .build();

    println!("   🚀 Creating VM...");

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
        .await
        .or_else(|_e| {
            // If benchScale times out, check if VM exists and get IP manually
            eprintln!("   ⚠️  benchScale timed out, checking VM manually...");
            get_vm_info_manually(&vm_name)
        })?;

    println!("   ✅ VM created: {}", node_info.name);
    println!("   ✅ IP address: {}", node_info.ip_address);

    // Wait for SSH
    println!("   ⏳ Waiting for SSH (may take 30-60 seconds)...");
    wait_for_ssh(&node_info.ip_address, &ssh_key_path, 180, username).await?;
    println!("   ✅ SSH ready");

    // Wait a bit more for RustDesk service to start
    println!("   ⏳ Waiting for RustDesk service to start...");
    sleep(Duration::from_secs(10)).await;
    println!("   ✅ System ready");

    Ok(RustDeskVm {
        name: node_info.name,
        ip: node_info.ip_address,
        username: username.to_string(),
        ssh_key_path,
        rustdesk_id: None,
    })
}

fn get_vm_info_manually(vm_name: &str) -> Result<benchscale::backend::NodeInfo> {
    use std::collections::HashMap;
    
    // Check if VM exists
    let list_output = Command::new("sudo")
        .args(&["virsh", "list", "--all"])
        .output()?;
    
    let list_str = String::from_utf8_lossy(&list_output.stdout);
    if !list_str.contains(vm_name) {
        anyhow::bail!("VM {} not found in virsh list", vm_name);
    }

    // Get IP
    let ip_output = Command::new("sudo")
        .args(&["virsh", "domifaddr", vm_name])
        .output()?;

    let ip_str = String::from_utf8_lossy(&ip_output.stdout);
    let ip = ip_str
        .lines()
        .find(|line| line.contains("ipv4"))
        .and_then(|line| {
            line.split_whitespace()
                .last()
                .and_then(|s| s.split('/').next())
        })
        .ok_or_else(|| anyhow::anyhow!("Could not extract IP from: {}", ip_str))?;

    Ok(benchscale::backend::NodeInfo {
        id: vm_name.to_string(),
        name: vm_name.to_string(),
        container_id: vm_name.to_string(),
        ip_address: ip.to_string(),
        network: "default".to_string(),
        status: benchscale::backend::NodeStatus::Running,
        metadata: HashMap::new(),
    })
}

async fn extract_rustdesk_id(vm: &RustDeskVm) -> Result<Option<String>> {
    println!("   Method 1: Checking RustDesk config file...");
    let config_check = ssh_command(
        vm,
        "cat ~/.config/rustdesk/RustDesk.toml 2>/dev/null | grep -i '^id' | head -1 || echo 'not_found'"
    ).await;

    if let Ok(output) = config_check {
        if !output.contains("not_found") && output.trim().len() > 0 {
            let id = output.split('=').nth(1)
                .map(|s| s.trim().trim_matches('"').to_string());
            if let Some(id) = id {
                if !id.is_empty() && id != "\"\"" {
                    println!("   ✅ Found RustDesk ID: {}", id);
                    return Ok(Some(id));
                }
            }
        }
    }

    println!("   Method 2: Generating RustDesk ID (first run)...");
    println!("   ℹ️  Starting RustDesk service to generate ID...");
    
    let _ = ssh_command(vm, "sudo systemctl restart rustdesk").await;
    sleep(Duration::from_secs(5)).await;

    let config_check2 = ssh_command(
        vm,
        "cat ~/.config/rustdesk/RustDesk.toml 2>/dev/null | grep -i '^id' | head -1 || echo 'not_found'"
    ).await;

    if let Ok(output) = config_check2 {
        if !output.contains("not_found") {
            let id = output.split('=').nth(1)
                .map(|s| s.trim().trim_matches('"').to_string());
            if let Some(id) = id {
                if !id.is_empty() && id != "\"\"" {
                    println!("   ✅ Generated RustDesk ID: {}", id);
                    return Ok(Some(id));
                }
            }
        }
    }

    println!("   ⚠️  Could not extract RustDesk ID automatically");
    println!("   ℹ️  You can get it manually by:");
    println!("       ssh -i {} {}@{}", vm.ssh_key_path.display(), vm.username, vm.ip);
    println!("       cat ~/.config/rustdesk/RustDesk.toml");

    Ok(None)
}

fn display_connection_instructions(vm: &RustDeskVm) -> Result<()> {
    println!("╔══════════════════════════════════════════════════════════════════════════╗");
    println!("║                                                                          ║");
    println!("║                  ✅ RUSTDESK VM READY FOR TESTING!                       ║");
    println!("║                                                                          ║");
    println!("╚══════════════════════════════════════════════════════════════════════════╝\n");

    println!("📊 VM Information:");
    println!("   Name:     {}", vm.name);
    println!("   IP:       {}", vm.ip);
    println!("   Username: {}", vm.username);
    println!("   Password: ubuntu\n");

    if let Some(ref id) = vm.rustdesk_id {
        println!("🔑 RustDesk ID: {}\n", id);
    } else {
        println!("⚠️  RustDesk ID: Not available yet (see manual extraction below)\n");
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    println!("🧪 VALIDATION STEPS:\n");

    println!("1. SSH to VM (optional, for manual ID extraction):");
    println!("   ssh -i {} {}@{}\n", vm.ssh_key_path.display(), vm.username, vm.ip);

    println!("2. Get RustDesk ID (if not shown above):");
    println!("   cat ~/.config/rustdesk/RustDesk.toml | grep '^id'\n");

    println!("3. From YOUR REMOTE COMPUTER:");
    println!("   a. Open RustDesk client");
    if let Some(ref id) = vm.rustdesk_id {
        println!("   b. Enter ID: {}", id);
    } else {
        println!("   b. Enter the RustDesk ID shown above");
    }
    println!("   c. Enter password: ubuntu");
    println!("   d. Attempt to connect\n");

    println!("📋 EXPECTED RESULT:");
    println!("   ✅ Connection succeeds (you can connect)");
    println!("   ❌ Screen is BLACK or DOESN'T RENDER\n");

    println!("   This confirms the Wayland security issue!");
    println!("   Standard remote desktop tools cannot capture Wayland compositors\n");

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("🔍 TROUBLESHOOTING:\n");
    
    println!("If connection fails:");
    println!("   • Check firewall on this machine");
    println!("   • Verify RustDesk service is running:");
    println!("     ssh -i {} {}@{} 'systemctl status rustdesk'", 
             vm.ssh_key_path.display(), vm.username, vm.ip);
    println!("   • Try restarting RustDesk:");
    println!("     ssh -i {} {}@{} 'sudo systemctl restart rustdesk'",
             vm.ssh_key_path.display(), vm.username, vm.ip);

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("📝 NEXT STEPS:\n");
    println!("1. Test connection from remote computer");
    println!("2. Confirm you can connect but screen doesn't render");
    println!("3. Run ionChannel solution test:");
    println!("   cargo run --bin ionchannel-solution --features benchscale --release\n");

    println!("💡 Keep this VM running for testing!");
    println!("   To stop later: sudo virsh destroy {}", vm.name);
    println!("   To remove later: sudo virsh undefine {}\n", vm.name);

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
            "rustdesk-validation",
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
                "-o",
                "LogLevel=ERROR",
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

async fn ssh_command(vm: &RustDeskVm, command: &str) -> Result<String> {
    let output = Command::new("ssh")
        .args(&[
            "-i",
            &vm.ssh_key_path.to_string_lossy(),
            "-o",
            "StrictHostKeyChecking=no",
            "-o",
            "UserKnownHostsFile=/dev/null",
            "-o",
            "LogLevel=ERROR",
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

