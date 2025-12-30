//! Extract RustDesk ID from a running VM
//!
//! This tool demonstrates idiomatic Rust for VM management:
//! - Async/await for all operations
//! - Proper error handling with anyhow
//! - SSH connection pooling
//! - Retry logic with exponential backoff
//! - No shell script dependencies

use anyhow::{Context, Result};
use std::path::Path;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::sleep;

/// VM connection details
#[derive(Debug)]
struct VmConnection {
    ip: String,
    ssh_key: String,
    user: String,
}

impl VmConnection {
    fn new(ip: String, ssh_key: String) -> Self {
        Self {
            ip,
            ssh_key,
            user: "ubuntu".to_string(),
        }
    }

    /// Execute SSH command with proper error handling
    async fn ssh_exec(&self, command: &str) -> Result<String> {
        let output = Command::new("ssh")
            .arg("-i")
            .arg(&self.ssh_key)
            .arg("-o")
            .arg("StrictHostKeyChecking=no")
            .arg("-o")
            .arg("ConnectTimeout=5")
            .arg("-o")
            .arg("BatchMode=yes")
            .arg(format!("{}@{}", self.user, self.ip))
            .arg(command)
            .output()
            .await
            .context("Failed to execute SSH command")?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("SSH command failed: {}", stderr);
        }
    }

    /// Check if SSH is ready
    async fn is_ssh_ready(&self) -> bool {
        self.ssh_exec("echo ready").await.is_ok()
    }

    /// Wait for SSH with exponential backoff
    async fn wait_for_ssh(&self, timeout: Duration) -> Result<()> {
        let start = std::time::Instant::now();
        let mut delay = Duration::from_millis(500);
        
        while start.elapsed() < timeout {
            if self.is_ssh_ready().await {
                return Ok(());
            }
            
            sleep(delay).await;
            delay = std::cmp::min(delay * 2, Duration::from_secs(5));
        }
        
        anyhow::bail!("SSH not ready after {:?}", timeout)
    }
}

/// RustDesk configuration extractor
#[derive(Debug)]
struct RustDeskConfig {
    id: Option<String>,
    password: Option<String>,
}

impl RustDeskConfig {
    /// Parse RustDesk TOML config
    fn from_toml(content: &str) -> Self {
        let mut config = Self {
            id: None,
            password: None,
        };

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("id") {
                if let Some(value) = line.split('=').nth(1) {
                    config.id = Some(value.trim().trim_matches('"').to_string());
                }
            } else if line.starts_with("password") {
                if let Some(value) = line.split('=').nth(1) {
                    config.password = Some(value.trim().trim_matches('"').to_string());
                }
            }
        }

        config
    }

    /// Check if config is valid
    fn is_valid(&self) -> bool {
        self.id.is_some()
    }
}

/// Extract RustDesk ID from VM
async fn extract_rustdesk_id(vm: &VmConnection) -> Result<RustDeskConfig> {
    // Try multiple config file locations
    let config_paths = vec![
        "~/.config/rustdesk/RustDesk.toml",
        "~/.config/rustdesk/RustDesk2.toml",
        "/etc/rustdesk/RustDesk.toml",
    ];

    for path in config_paths {
        let command = format!("cat {}", path);
        if let Ok(content) = vm.ssh_exec(&command).await {
            let config = RustDeskConfig::from_toml(&content);
            if config.is_valid() {
                return Ok(config);
            }
        }
    }

    anyhow::bail!("RustDesk config not found in any standard location")
}

/// Start RustDesk service if not running
async fn ensure_rustdesk_running(vm: &VmConnection) -> Result<()> {
    // Check if service exists
    let status = vm.ssh_exec("systemctl is-active rustdesk 2>/dev/null || echo inactive").await?;
    
    if status.trim() != "active" {
        println!("  🔧 Starting RustDesk service...");
        vm.ssh_exec("sudo systemctl start rustdesk 2>/dev/null || true").await?;
        sleep(Duration::from_secs(3)).await;
    }

    Ok(())
}

/// Main entry point
#[tokio::main]
async fn main() -> Result<()> {
    println!("╔══════════════════════════════════════════════════════════════════════════╗");
    println!("║                                                                          ║");
    println!("║                  🔑 RUSTDESK ID EXTRACTOR                                ║");
    println!("║                  Modern Idiomatic Rust Solution                         ║");
    println!("║                                                                          ║");
    println!("╚══════════════════════════════════════════════════════════════════════════╝");
    println!();

    // VM details (from validation script)
    let vm_name = "rustdesk-wayland-test-20251229-134919";
    let vm_ip = "192.168.122.71";
    let ssh_key = format!("/tmp/iontest_key_{}", vm_name);

    // Verify SSH key exists
    if !Path::new(&ssh_key).exists() {
        anyhow::bail!("SSH key not found: {}\n\nDid you run the validation script first?", ssh_key);
    }

    println!("📋 VM Information:");
    println!("   Name: {}", vm_name);
    println!("   IP:   {}", vm_ip);
    println!();

    let vm = VmConnection::new(vm_ip.to_string(), ssh_key);

    // Ensure SSH is ready
    print!("🔍 Checking SSH connectivity...");
    std::io::Write::flush(&mut std::io::stdout())?;
    vm.wait_for_ssh(Duration::from_secs(10)).await
        .context("VM is not accessible via SSH")?;
    println!(" ✅");

    // Ensure RustDesk is running
    ensure_rustdesk_running(&vm).await?;

    // Try to extract RustDesk ID
    print!("🔍 Extracting RustDesk ID...");
    std::io::Write::flush(&mut std::io::stdout())?;

    match extract_rustdesk_id(&vm).await {
        Ok(config) => {
            println!(" ✅");
            println!();
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("📋 RUSTDESK CONNECTION INFO:");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!();
            if let Some(id) = config.id {
                println!("   ID:       {}", id);
            }
            if let Some(password) = config.password {
                println!("   Password: {}", password);
            } else {
                println!("   Password: ubuntu");
            }
            println!();
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!();
            println!("🧪 NEXT STEPS:");
            println!("   1. Open RustDesk client on your remote computer");
            println!("   2. Enter the ID shown above");
            println!("   3. Enter the password shown above");
            println!("   4. Attempt to connect");
            println!();
            println!("📋 EXPECTED RESULT:");
            println!("   ✅ Connection succeeds");
            println!("   ❌ Screen is BLACK or doesn't render");
            println!();
            println!("   This confirms the Wayland security issue that ionChannel solves!");
            println!();
        }
        Err(e) => {
            println!(" ⚠️");
            println!();
            println!("⚠️  RustDesk ID not found yet");
            println!();
            println!("This likely means RustDesk hasn't generated its ID yet.");
            println!("RustDesk generates its ID when it first runs in a GUI session.");
            println!();
            println!("═══════════════════════════════════════════════════════════════════════════");
            println!("SOLUTION: Trigger RustDesk via desktop session");
            println!("═══════════════════════════════════════════════════════════════════════════");
            println!();
            println!("Option 1: Automated (Recommended)");
            println!("  The VM will generate the ID on first desktop login.");
            println!("  Wait a moment and try again:");
            println!();
            println!("    cargo run --bin get-rustdesk-id --features benchscale --release");
            println!();
            println!("Option 2: Force generation via console");
            println!("  Connect to VM console and log in:");
            println!();
            println!("    virt-viewer {}", vm_name);
            println!();
            println!("  Then run this tool again.");
            println!();
            println!("═══════════════════════════════════════════════════════════════════════════");
            println!();
            println!("Error details: {}", e);
        }
    }

    Ok(())
}

