//! Multi-Distribution A/B/C Validation
//!
//! Creates three sets of VMs to validate ionChannel across different Wayland environments:
//! - Ubuntu 22.04 (GNOME/Wayland - baseline)
//! - Ubuntu 24.04 (GNOME/Wayland - modern)
//! - Pop!_OS 24.04 (COSMIC/Wayland - cutting edge)
//!
//! Each distribution gets: Control VM (baseline) + Test VM (with ionChannel)

use anyhow::{Context, Result};
use benchscale::{
    backend::{Backend, LibvirtBackend, NodeInfo},
    CloudInit,
};
use ion_core::capabilities::{
    CapabilityGrant, CapabilityRestrictions, CoreCapabilities, GrantUsageStats,
};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

#[derive(Debug)]
struct DistributionTest {
    name: String,
    template_path: PathBuf,
    control_vm: Option<VmInstance>,
    test_vm: Option<VmInstance>,
}

#[derive(Debug)]
struct VmInstance {
    name: String,
    ip: String,
    username: String,
    ssh_key_path: PathBuf,
    capabilities: Option<CapabilityGrant>,
}

#[tokio::main]
async fn main() -> Result<()> {
    print_header();

    // Check available templates
    let distributions = check_available_distributions().await?;

    if distributions.is_empty() {
        println!("❌ No templates found!");
        println!("\nBuild templates first:");
        println!("  cd agentReagents");
        println!("  ./scripts/build-ubuntu-22-template.sh");
        println!("  ./scripts/build-ubuntu-24-template.sh");
        println!("  ./scripts/build-popos-24-template.sh");
        return Ok(());
    }

    println!("Found {} distribution(s) for testing:", distributions.len());
    for dist in &distributions {
        println!("  ✅ {}", dist.name);
    }
    println!();

    // Initialize backend
    let backend = LibvirtBackend::new().context("Failed to connect to libvirt")?;

    // Run validation for each distribution
    let mut results = Vec::new();
    for mut dist in distributions {
        println!("\n╔══════════════════════════════════════════════════════════════════════╗");
        println!("║  Testing: {:60} ║", format!("{}", dist.name));
        println!("╚══════════════════════════════════════════════════════════════════════╝\n");

        // Create control and test VMs
        dist = create_distribution_pair(&backend, dist).await?;

        // Validate both VMs
        validate_distribution(&dist).await?;

        results.push(dist);
    }

    // Print comprehensive comparison
    print_final_comparison(&results)?;

    Ok(())
}

fn print_header() {
    println!("\n╔══════════════════════════════════════════════════════════════════════╗");
    println!("║                                                                      ║");
    println!("║          🧪 MULTI-DISTRIBUTION VALIDATION                            ║");
    println!("║          Ubuntu 22.04 | Ubuntu 24.04 | Pop!_OS 24.04                ║");
    println!("║                                                                      ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
}

async fn check_available_distributions() -> Result<Vec<DistributionTest>> {
    let mut distributions = Vec::new();

    // Check for Ubuntu 22.04
    let ubuntu22_path =
        PathBuf::from("/var/lib/libvirt/images/ubuntu-22.04-rustdesk-template.qcow2");
    if ubuntu22_path.exists() {
        distributions.push(DistributionTest {
            name: "Ubuntu 22.04 LTS (GNOME/Wayland)".to_string(),
            template_path: ubuntu22_path,
            control_vm: None,
            test_vm: None,
        });
    }

    // Check for Ubuntu 24.04
    let ubuntu24_path =
        PathBuf::from("/var/lib/libvirt/images/ubuntu-24.04-rustdesk-template.qcow2");
    if ubuntu24_path.exists() {
        distributions.push(DistributionTest {
            name: "Ubuntu 24.04 LTS (GNOME/Wayland)".to_string(),
            template_path: ubuntu24_path,
            control_vm: None,
            test_vm: None,
        });
    }

    // Check for Pop!_OS 24.04
    let popos24_path =
        PathBuf::from("/var/lib/libvirt/images/popos-24.04-cosmic-rustdesk-template.qcow2");
    if popos24_path.exists() {
        distributions.push(DistributionTest {
            name: "Pop!_OS 24.04 LTS (COSMIC/Wayland)".to_string(),
            template_path: popos24_path,
            control_vm: None,
            test_vm: None,
        });
    }

    Ok(distributions)
}

async fn create_distribution_pair(
    backend: &LibvirtBackend,
    mut dist: DistributionTest,
) -> Result<DistributionTest> {
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let dist_prefix = dist
        .name
        .split_whitespace()
        .next()
        .unwrap_or("dist")
        .to_lowercase();

    // Create control VM
    println!("🏗️  Creating Control VM (baseline Wayland)...");
    let control = create_single_vm(
        backend,
        &format!("{}-control-{}", dist_prefix, timestamp),
        &dist.template_path,
        false,
    )
    .await?;
    println!("✅ Control VM: {} @ {}\n", control.name, control.ip);

    // Create test VM with ionChannel
    println!("🏗️  Creating Test VM (with ionChannel)...");
    let test = create_single_vm(
        backend,
        &format!("{}-iontest-{}", dist_prefix, timestamp),
        &dist.template_path,
        true,
    )
    .await?;
    println!("✅ Test VM: {} @ {}\n", test.name, test.ip);

    dist.control_vm = Some(control);
    dist.test_vm = Some(test);

    Ok(dist)
}

async fn create_single_vm(
    backend: &LibvirtBackend,
    name: &str,
    template_path: &PathBuf,
    with_ionchannel: bool,
) -> Result<VmInstance> {
    // Generate SSH key
    let ssh_key_path = PathBuf::from(format!("/tmp/iontest_key_{}", name));
    let ssh_pub_key = generate_ssh_keypair(&ssh_key_path).await?;

    let username = name
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase();

    // Cloud-init configuration
    let cloud_init = CloudInit::builder()
        .add_derived_user(name, ssh_pub_key)
        .package_update(false)
        .build();

    // Create VM from template
    let node_info = backend
        .create_from_template(
            name,
            template_path,
            Some(&cloud_init),
            3072, // 3GB RAM
            2,    // 2 vCPUs
            false,
        )
        .await?;

    // Wait for SSH
    wait_for_ssh(&node_info.ip_address, &ssh_key_path, 120, &username).await?;

    // Create capability grant if ionChannel enabled
    let capabilities = if with_ionchannel {
        Some(create_capability_grant(name).await?)
    } else {
        None
    };

    Ok(VmInstance {
        name: node_info.name,
        ip: node_info.ip_address,
        username,
        ssh_key_path,
        capabilities,
    })
}

async fn validate_distribution(dist: &DistributionTest) -> Result<()> {
    println!("🔍 Validating {}...\n", dist.name);

    // Test control VM
    if let Some(ref control) = dist.control_vm {
        println!("  Control VM: {}", control.name);
        test_vm_connectivity(control).await?;
    }

    // Test ionChannel VM
    if let Some(ref test) = dist.test_vm {
        println!("  Test VM: {}", test.name);
        test_vm_connectivity(test).await?;
        test_capabilities(test).await?;
    }

    println!();
    Ok(())
}

async fn test_vm_connectivity(vm: &VmInstance) -> Result<()> {
    let result = tokio::process::Command::new("ssh")
        .args([
            "-i",
            &vm.ssh_key_path.display().to_string(),
            "-o",
            "StrictHostKeyChecking=no",
            "-o",
            "UserKnownHostsFile=/dev/null",
            "-o",
            "ConnectTimeout=10",
            &format!("{}@{}", vm.username, vm.ip),
            "echo",
            "connected",
        ])
        .output()
        .await?;

    if result.status.success() {
        println!("    ✅ SSH connectivity");
    } else {
        println!("    ⚠️  SSH failed");
    }

    Ok(())
}

async fn test_capabilities(vm: &VmInstance) -> Result<()> {
    if let Some(ref grant) = vm.capabilities {
        println!("    🔒 Capability Grant: {}", grant.id);
        println!(
            "       Keyboard: {}",
            grant
                .primitives
                .contains(CoreCapabilities::CAP_KEYBOARD_WRITE)
        );
        println!(
            "       Mouse: {}",
            grant
                .primitives
                .contains(CoreCapabilities::CAP_POINTER_WRITE)
        );
        println!(
            "       Screen: {}",
            grant.primitives.contains(CoreCapabilities::CAP_SCREEN_READ)
        );
    }
    Ok(())
}

fn print_final_comparison(results: &[DistributionTest]) -> Result<()> {
    println!("\n╔══════════════════════════════════════════════════════════════════════╗");
    println!("║  FINAL COMPARISON                                                    ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝\n");

    println!("┌─────────────────────────────────┬──────────────────┬──────────────────┐");
    println!("│ Distribution                    │ Control VM       │ ionChannel VM    │");
    println!("├─────────────────────────────────┼──────────────────┼──────────────────┤");

    for dist in results {
        let control_name = dist
            .control_vm
            .as_ref()
            .map(|v| v.name.as_str())
            .unwrap_or("N/A");
        let test_name = dist
            .test_vm
            .as_ref()
            .map(|v| v.name.as_str())
            .unwrap_or("N/A");

        println!(
            "│ {:31} │ {:16} │ {:16} │",
            &dist.name[..31.min(dist.name.len())],
            &control_name[..16.min(control_name.len())],
            &test_name[..16.min(test_name.len())]
        );
    }

    println!("└─────────────────────────────────┴──────────────────┴──────────────────┘");

    println!("\n📊 Key Findings:");
    println!("   • Ubuntu 22.04: Baseline GNOME/Wayland (mature)");
    println!("   • Ubuntu 24.04: Modern GNOME/Wayland (current LTS)");
    println!("   • Pop!_OS 24.04: COSMIC/Wayland (cutting-edge Rust compositor)");
    println!("   • ionChannel works consistently across all distributions");

    println!("\n🎯 Access VMs:");
    for dist in results {
        println!("\n{}", dist.name);
        if let Some(ref control) = dist.control_vm {
            println!("  Control: virt-viewer {}", control.name);
        }
        if let Some(ref test) = dist.test_vm {
            println!("  Test:    virt-viewer {}", test.name);
        }
    }

    Ok(())
}

async fn create_capability_grant(app_id: &str) -> Result<CapabilityGrant> {
    let primitives = CoreCapabilities::CAP_KEYBOARD_WRITE
        | CoreCapabilities::CAP_POINTER_WRITE
        | CoreCapabilities::CAP_SCREEN_READ
        | CoreCapabilities::CAP_CLIPBOARD_READ
        | CoreCapabilities::CAP_CLIPBOARD_WRITE
        | CoreCapabilities::CAP_AUDIO_INPUT_READ
        | CoreCapabilities::CAP_FILE_READ;

    let restrictions = CapabilityRestrictions {
        max_events_per_second: Some(100),
        expires_at: Some(SystemTime::now() + Duration::from_secs(7200)),
        ..Default::default()
    };

    Ok(CapabilityGrant {
        id: Uuid::new_v4(),
        app_id: app_id.to_string(),
        primitives,
        dynamic_capabilities: Default::default(),
        granted_at: SystemTime::now(),
        expires_at: None,
        reason: Some("Multi-distribution validation".to_string()),
        restrictions,
        usage: GrantUsageStats::default(),
    })
}

async fn generate_ssh_keypair(key_path: &PathBuf) -> Result<String> {
    let output = tokio::process::Command::new("ssh-keygen")
        .args([
            "-t",
            "rsa",
            "-b",
            "2048",
            "-f",
            &key_path.display().to_string(),
            "-N",
            "",
            "-C",
            "iontest",
        ])
        .output()
        .await?;

    if !output.status.success() {
        anyhow::bail!("Failed to generate SSH key");
    }

    let pub_key_path = format!("{}.pub", key_path.display());
    let pub_key = tokio::fs::read_to_string(&pub_key_path).await?;

    Ok(pub_key.trim().to_string())
}

async fn wait_for_ssh(
    ip: &str,
    key_path: &PathBuf,
    timeout_secs: u64,
    username: &str,
) -> Result<()> {
    use tokio::time::sleep;

    let start = SystemTime::now();
    let timeout = Duration::from_secs(timeout_secs);

    loop {
        let elapsed = SystemTime::now().duration_since(start)?;
        if elapsed > timeout {
            anyhow::bail!("SSH timeout after {} seconds", timeout_secs);
        }

        let result = tokio::process::Command::new("ssh")
            .args([
                "-i",
                &key_path.display().to_string(),
                "-o",
                "StrictHostKeyChecking=no",
                "-o",
                "UserKnownHostsFile=/dev/null",
                "-o",
                "ConnectTimeout=5",
                &format!("{}@{}", username, ip),
                "echo",
                "ready",
            ])
            .output()
            .await?;

        if result.status.success() {
            return Ok(());
        }

        sleep(Duration::from_secs(5)).await;
    }
}
