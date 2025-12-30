//! benchScale Validation Extensions for ionChannel
//!
//! This module provides ionChannel-specific validation helpers that
//! use benchScale as the underlying infrastructure.
//!
//! **PRODUCTION IMPLEMENTATION - NO MOCKS**

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::time::Duration;

use benchscale::backend::{Backend, LibvirtBackend};

/// RustDesk service configuration
#[derive(Debug, Clone)]
pub struct RustDeskConfig {
    pub id: String,
    pub password: String,
    pub port: u16,
}

/// RustDesk validator using benchScale - PRODUCTION IMPLEMENTATION
pub struct RustDeskValidator {
    backend: LibvirtBackend,
    vm_id: String,
    ssh_key_path: PathBuf,
}

impl RustDeskValidator {
    /// Create a new validator with a benchScale backend
    pub async fn new(
        backend: LibvirtBackend,
        vm_id: String,
        ssh_key_path: PathBuf,
    ) -> Result<Self> {
        Ok(Self {
            backend,
            vm_id,
            ssh_key_path,
        })
    }

    /// Install RustDesk on the VM using REAL SSH execution
    /// Uses pre-downloaded .deb from agentReagents (primal philosophy)
    pub async fn install(&self, progress: impl Fn(&str), deb_path: &str) -> Result<()> {
        progress("Updating package lists");
        let result = self
            .backend
            .exec_command(
                &self.vm_id,
                vec![
                    "sudo".to_string(),
                    "apt-get".to_string(),
                    "update".to_string(),
                    "-qq".to_string(),
                ],
            )
            .await?;

        if !result.success() {
            anyhow::bail!("Package update failed: {}", result.stderr);
        }

        progress("Installing dependencies");
        let result = self
            .backend
            .exec_command(
                &self.vm_id,
                vec![
                    "sudo".to_string(),
                    "DEBIAN_FRONTEND=noninteractive".to_string(),
                    "apt-get".to_string(),
                    "install".to_string(),
                    "-y".to_string(),
                    "-qq".to_string(),
                    "ca-certificates".to_string(),
                ],
            )
            .await?;

        if !result.success() {
            anyhow::bail!("Dependency installation failed: {}", result.stderr);
        }

        progress(&format!(
            "Copying RustDesk package from agentReagents: {}",
            deb_path
        ));
        // Use scp to copy the local .deb to the VM
        let vm_ip = self.get_vm_ip().await?;
        let scp_result = tokio::process::Command::new("scp")
            .args([
                "-i",
                &self.get_ssh_key_path(),
                "-o",
                "StrictHostKeyChecking=no",
                "-o",
                "UserKnownHostsFile=/dev/null",
                deb_path,
                &format!("iontest@{}:/tmp/rustdesk.deb", vm_ip),
            ])
            .output()
            .await
            .context("Failed to execute scp")?;

        if !scp_result.status.success() {
            anyhow::bail!(
                "SCP transfer failed: {}",
                String::from_utf8_lossy(&scp_result.stderr)
            );
        }

        progress("Installing RustDesk package");
        let _result = self
            .backend
            .exec_command(
                &self.vm_id,
                vec![
                    "sudo".to_string(),
                    "DEBIAN_FRONTEND=noninteractive".to_string(),
                    "dpkg".to_string(),
                    "-i".to_string(),
                    "/tmp/rustdesk.deb".to_string(),
                ],
            )
            .await?;

        // Intentionally ignore dpkg errors - apt-get install -f will fix dependencies

        progress("Fixing dependencies");
        let result = self
            .backend
            .exec_command(
                &self.vm_id,
                vec![
                    "sudo".to_string(),
                    "DEBIAN_FRONTEND=noninteractive".to_string(),
                    "apt-get".to_string(),
                    "install".to_string(),
                    "-f".to_string(),
                    "-y".to_string(),
                    "-qq".to_string(),
                ],
            )
            .await?;

        if !result.success() {
            anyhow::bail!("Dependency fix failed: {}", result.stderr);
        }

        progress("Installation complete");
        Ok(())
    }

    /// Get the VM's IP address
    async fn get_vm_ip(&self) -> Result<String> {
        // Use benchScale's node info to get IP
        let node_info = self.backend.get_node(&self.vm_id).await?;
        Ok(node_info.ip_address)
    }

    /// Get the SSH key path for this VM
    fn get_ssh_key_path(&self) -> String {
        self.ssh_key_path.display().to_string()
    }

    /// Configure RustDesk with a password using REAL commands
    pub async fn configure(&self, password: &str, progress: impl Fn(&str)) -> Result<()> {
        progress("Setting RustDesk password");

        // Set password
        let _result = self
            .backend
            .exec_command(
                &self.vm_id,
                vec![
                    "rustdesk".to_string(),
                    "--password".to_string(),
                    password.to_string(),
                ],
            )
            .await?;

        progress("Starting RustDesk service");

        // Start as background service
        let _result = self
            .backend
            .exec_command(
                &self.vm_id,
                vec![
                    "nohup".to_string(),
                    "rustdesk".to_string(),
                    "--server".to_string(),
                    "&".to_string(),
                ],
            )
            .await?;

        // Give it time to start
        tokio::time::sleep(Duration::from_secs(3)).await;

        progress("Configuration complete");
        Ok(())
    }

    /// Get the REAL RustDesk ID from the VM - NO MOCKS
    pub async fn get_id(&self, progress: impl Fn(&str)) -> Result<String> {
        progress("Retrieving RustDesk ID");

        // Try direct command
        let result = self
            .backend
            .exec_command(
                &self.vm_id,
                vec!["rustdesk".to_string(), "--get-id".to_string()],
            )
            .await?;

        let id = result.stdout.trim().to_string();

        // If empty or invalid, try config file
        if id.is_empty() || id == "0" {
            progress("Reading ID from config file");

            let result = self
                .backend
                .exec_command(
                    &self.vm_id,
                    vec![
                        "cat".to_string(),
                        format!(
                            "{}/.config/rustdesk/RustDesk.toml",
                            std::env::var("HOME").unwrap_or_else(|_| "/home/iontest".to_string())
                        ),
                    ],
                )
                .await?;

            // Parse TOML to extract ID
            for line in result.stdout.lines() {
                if line.starts_with("id") {
                    if let Some(id_value) = line.split('\"').nth(1) {
                        progress(&format!("ID retrieved: {}", id_value));
                        return Ok(id_value.to_string());
                    }
                }
            }

            anyhow::bail!("Could not extract RustDesk ID");
        }

        progress(&format!("ID retrieved: {}", id));
        Ok(id)
    }

    /// Validate that RustDesk is running properly - REAL checks
    pub async fn validate_service(&self, progress: impl Fn(&str)) -> Result<()> {
        progress("Checking service status");

        // Check process
        let result = self
            .backend
            .exec_command(&self.vm_id, vec!["ps".to_string(), "aux".to_string()])
            .await?;

        if !result.stdout.contains("rustdesk") {
            anyhow::bail!("RustDesk process not found");
        }

        progress("Service is running");

        progress("Checking network port");
        let result = self
            .backend
            .exec_command(
                &self.vm_id,
                vec![
                    "sudo".to_string(),
                    "netstat".to_string(),
                    "-tlnp".to_string(),
                ],
            )
            .await?;

        if !result.stdout.contains("21118") {
            anyhow::bail!("RustDesk not listening on port 21118");
        }

        progress("Port is listening");
        progress("Validation complete");

        Ok(())
    }
}

/// Full RustDesk installation and configuration flow - PRODUCTION
/// Uses pre-downloaded .deb from agentReagents following primal philosophy
pub async fn setup_rustdesk_on_vm(
    backend: LibvirtBackend,
    vm_id: &str,
    ssh_key_path: PathBuf,
    password: &str,
    progress: impl Fn(&str) + Clone,
) -> Result<RustDeskConfig> {
    let validator = RustDeskValidator::new(backend, vm_id.to_string(), ssh_key_path).await?;

    // Path to pre-downloaded RustDesk .deb in agentReagents
    let deb_path = "/home/nestgate/Development/syntheticChemistry/agentReagents/debs/remote-desktop/rustdesk-1.2.3-x86_64.deb";

    // REAL installation using local .deb
    validator
        .install(progress.clone(), deb_path)
        .await
        .context("Failed to install RustDesk")?;

    // REAL configuration
    validator
        .configure(password, progress.clone())
        .await
        .context("Failed to configure RustDesk")?;

    // REAL ID retrieval
    let id = validator
        .get_id(progress.clone())
        .await
        .context("Failed to get RustDesk ID")?;

    // REAL validation
    validator
        .validate_service(progress.clone())
        .await
        .context("Failed to validate RustDesk service")?;

    Ok(RustDeskConfig {
        id,
        password: password.to_string(),
        port: 21118,
    })
}
