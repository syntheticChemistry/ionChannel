//! Automatic RustDesk ID Generator - Zero GUI Dependency
//!
//! Eliminates deep debt by automating RustDesk ID generation via SSH.
//! No X11/Wayland display required, no manual login, fully automated.

use anyhow::{Context, Result};
use std::time::Duration;
use tracing::{info, warn};

#[cfg(feature = "benchscale")]
use benchscale::backend::ssh::SshClient;

struct Args {
    vm_name: String,
    username: String,
    password: String,
    timeout: u64,
}

impl Args {
    fn parse() -> Self {
        let args: Vec<String> = std::env::args().collect();
        
        // Simple argument parsing
        let mut vm_name = String::new();
        let mut username = "ubuntu".to_string();
        let mut password = "ubuntu".to_string();
        let mut timeout = 180u64;

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--vm-name" if i + 1 < args.len() => {
                    vm_name = args[i + 1].clone();
                    i += 2;
                }
                "--username" if i + 1 < args.len() => {
                    username = args[i + 1].clone();
                    i += 2;
                }
                "--password" if i + 1 < args.len() => {
                    password = args[i + 1].clone();
                    i += 2;
                }
                "--timeout" if i + 1 < args.len() => {
                    timeout = args[i + 1].parse().unwrap_or(180);
                    i += 2;
                }
                _ => i += 1,
            }
        }

        Self {
            vm_name,
            username,
            password,
            timeout,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("auto_generate_rustdesk_id=info".parse()?),
        )
        .init();

    let args = Args::parse();

    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════╗");
    println!("║                                                                          ║");
    println!("║           🤖 AUTOMATIC RUSTDESK ID GENERATOR                             ║");
    println!("║           Zero GUI Dependency - Pure Automation                          ║");
    println!("║                                                                          ║");
    println!("╚══════════════════════════════════════════════════════════════════════════╝");
    println!();

    // Get VM IP from virsh
    info!("Getting VM IP address...");
    let output = tokio::process::Command::new("sudo")
        .args(&["virsh", "domifaddr", &args.vm_name])
        .output()
        .await
        .context("Failed to run virsh command")?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let ip_address = output_str
        .lines()
        .find(|line| line.contains("ipv4"))
        .and_then(|line| {
            line.split_whitespace()
                .find(|part| part.contains('/'))
                .and_then(|ip_cidr| ip_cidr.split('/').next())
        })
        .context("Failed to extract IP address from virsh output")?
        .to_string();

    println!("📋 VM Information:");
    println!("   Name: {}", args.vm_name);
    println!("   IP:   {}", ip_address);
    println!();

    // Connect via SSH
    info!("Connecting via SSH to {}...", ip_address);
    println!("🔌 Establishing SSH connection...");

    let mut ssh = SshClient::connect(
        &ip_address,
        22,
        &args.username,
        &args.password,
    )
    .await
    .context("Failed to connect via SSH")?;

    println!("   ✅ SSH connected");
    println!();

    // Start desktop session and RustDesk
    println!("🚀 Starting desktop automation...");
    
    // 1. Start Xvfb (virtual X server) in background
    info!("Starting virtual X server...");
    let (exit_code, stdout, stderr) = ssh
        .execute(&[
            "pkill Xvfb || true; \
             Xvfb :99 -screen 0 1024x768x24 &> /tmp/xvfb.log & \
             sleep 2 && echo 'Xvfb started'".to_string(),
        ])
        .await?;
    
    if exit_code == 0 && stdout.contains("Xvfb started") {
        println!("   ✅ Virtual X server started");
    } else {
        warn!("Xvfb may not have started cleanly: {}", stderr);
    }

    // 2. Start RustDesk with DISPLAY=:99
    info!("Starting RustDesk...");
    println!("🖥️  Starting RustDesk service...");
    
    let (exit_code, stdout, stderr) = ssh
        .execute(&[
            "export DISPLAY=:99 && \
             pkill rustdesk || true && \
             rustdesk --server &> /tmp/rustdesk.log & \
             sleep 5 && echo 'RustDesk started'".to_string(),
        ])
        .await?;

    if exit_code == 0 {
        println!("   ✅ RustDesk started");
    } else {
        warn!("RustDesk output: {} / {}", stdout, stderr);
    }

    // 3. Wait for config file to be generated
    info!("Waiting for RustDesk to generate ID...");
    println!("⏳ Waiting for ID generation (up to {} seconds)...", args.timeout);
    
    let timeout = Duration::from_secs(args.timeout);
    let start = std::time::Instant::now();
    let mut rustdesk_id: Option<String> = None;

    while start.elapsed() < timeout {
        // Try multiple config locations
        let (exit_code, stdout, _) = ssh
            .execute(&[
                "cat ~/.config/rustdesk/RustDesk2.toml 2>/dev/null || \
                 cat ~/.config/rustdesk/RustDesk.toml 2>/dev/null || \
                 echo 'not found'".to_string(),
            ])
            .await?;

        if exit_code == 0 && stdout.contains("id =") {
            // Parse the ID
            for line in stdout.lines() {
                if line.trim().starts_with("id =") {
                    if let Some(id) = line.split('"').nth(1) {
                        rustdesk_id = Some(id.to_string());
                        break;
                    }
                }
            }

            if rustdesk_id.is_some() {
                break;
            }
        }

        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    // Disconnect SSH
    ssh.disconnect().await?;

    println!();

    // Report results
    if let Some(id) = rustdesk_id {
        println!("╔══════════════════════════════════════════════════════════════════════════╗");
        println!("║                                                                          ║");
        println!("║                  ✅ RUSTDESK ID GENERATED!                               ║");
        println!("║                                                                          ║");
        println!("╚══════════════════════════════════════════════════════════════════════════╝");
        println!();
        println!("🔑 RustDesk ID: {}", id);
        println!();
        println!("═══════════════════════════════════════════════════════════════════════════");
        println!("                    📋 NEXT STEPS");
        println!("═══════════════════════════════════════════════════════════════════════════");
        println!();
        println!("1. Open RustDesk client on remote computer");
        println!("2. Enter ID: {}", id);
        println!("3. Password: {}", args.password);
        println!("4. Expected: BLACK SCREEN (this is the bug we're solving!)");
        println!();
        println!("Once you confirm the black screen issue, we'll implement the");
        println!("ionChannel solution to enable proper Wayland screen capture.");
        println!();
        println!("═══════════════════════════════════════════════════════════════════════════");
        println!();

        Ok(())
    } else {
        println!("⚠️  RustDesk ID not generated within {} seconds", args.timeout);
        println!();
        println!("This can happen if:");
        println!("  • RustDesk needs more time (try increasing --timeout)");
        println!("  • Network issues prevent RustDesk from connecting to its server");
        println!("  • RustDesk package has issues");
        println!();
        println!("Debug info:");
        println!("  VM: {}", args.vm_name);
        println!("  IP: {}", ip_address);
        println!();
        println!("You can SSH in to check logs:");
        println!("  ssh {}@{}", args.username, ip_address);
        println!("  cat /tmp/rustdesk.log");
        println!("  cat /tmp/xvfb.log");
        println!();

        anyhow::bail!("Timeout waiting for RustDesk ID");
    }
}

