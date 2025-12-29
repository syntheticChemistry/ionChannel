//! E2E tests for ionChannel + benchScale integration
//!
//! These tests validate the full stack from VM provisioning to capability grants.

#[cfg(test)]
mod e2e_tests {
    use anyhow::Result;
    use std::path::PathBuf;

    /// Test cloud-init SSH key delivery
    #[tokio::test]
    #[ignore] // Requires libvirt setup
    async fn test_cloud_init_ssh_keys() -> Result<()> {
        use benchscale::{backend::LibvirtBackend, CloudInit};

        let backend = LibvirtBackend::new()?;
        let vm_name = "test-cloud-init-ssh";
        let template =
            PathBuf::from("/var/lib/libvirt/images/rustdesk-ubuntu-22.04-template.qcow2");

        // Generate test SSH key
        let key_path = PathBuf::from("/tmp/test_key");
        std::process::Command::new("ssh-keygen")
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
                "test",
            ])
            .output()?;

        let pub_key = std::fs::read_to_string(format!("{}.pub", key_path.display()))?;

        let cloud_init = CloudInit::builder()
            .add_derived_user(vm_name, pub_key)
            .build();

        // Create VM
        let node = backend
            .create_from_template(vm_name, &template, Some(&cloud_init), 1024, 1, false)
            .await?;

        // Test SSH connection
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

        let username: String = vm_name
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_lowercase();

        let ssh_result = tokio::process::Command::new("ssh")
            .args([
                "-i",
                &key_path.display().to_string(),
                "-o",
                "StrictHostKeyChecking=no",
                "-o",
                "UserKnownHostsFile=/dev/null",
                &format!("{}@{}", username, node.ip_address),
                "echo",
                "success",
            ])
            .output()
            .await?;

        // Cleanup
        std::process::Command::new("virsh")
            .args(["destroy", vm_name])
            .output()?;
        std::process::Command::new("virsh")
            .args(["undefine", vm_name])
            .output()?;

        assert!(ssh_result.status.success(), "SSH connection should work");
        assert_eq!(
            String::from_utf8_lossy(&ssh_result.stdout).trim(),
            "success"
        );

        Ok(())
    }

    /// Test multi-VM web deployment
    #[tokio::test]
    #[ignore] // Requires libvirt setup
    async fn test_multi_vm_web_deployment() -> Result<()> {
        use benchscale::{backend::LibvirtBackend, CloudInit};

        let backend = LibvirtBackend::new()?;
        let template =
            PathBuf::from("/var/lib/libvirt/images/rustdesk-ubuntu-22.04-template.qcow2");

        // Generate SSH key
        let key_path = PathBuf::from("/tmp/test_multi_key");
        std::process::Command::new("ssh-keygen")
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
                "test",
            ])
            .output()?;

        let pub_key = std::fs::read_to_string(format!("{}.pub", key_path.display()))?;

        // Deploy web-01, web-02, web-03
        let vm_names = vec!["web-01", "web-02", "web-03"];
        let mut nodes = Vec::new();

        for vm_name in vm_names.iter() {
            let cloud_init = CloudInit::builder()
                .add_derived_user(*vm_name, pub_key.clone())
                .build();

            let node = backend
                .create_from_template(vm_name, &template, Some(&cloud_init), 512, 1, false)
                .await?;

            nodes.push(node);
        }

        // Wait for all VMs to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(90)).await;

        // Test SSH to each VM with derived credentials
        for (i, vm_name) in vm_names.iter().enumerate() {
            let username: String = vm_name
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase();

            let ssh_result = tokio::process::Command::new("ssh")
                .args([
                    "-i",
                    &key_path.display().to_string(),
                    "-o",
                    "StrictHostKeyChecking=no",
                    "-o",
                    "UserKnownHostsFile=/dev/null",
                    &format!("{}@{}", username, nodes[i].ip_address),
                    "hostname",
                ])
                .output()
                .await?;

            assert!(
                ssh_result.status.success(),
                "SSH to {} should work",
                vm_name
            );
        }

        // Cleanup
        for vm_name in &vm_names {
            std::process::Command::new("virsh")
                .args(["destroy", vm_name])
                .output()?;
            std::process::Command::new("virsh")
                .args(["undefine", vm_name])
                .output()?;
        }

        Ok(())
    }

    /// Test capability grant creation and storage
    #[tokio::test]
    async fn test_capability_grant_lifecycle() -> Result<()> {
        use ion_core::capabilities::{
            CapabilityGrant, CapabilityRestrictions, CoreCapabilities, FileSystemGrantStorage,
            GrantStorage, GrantUsageStats,
        };
        use std::time::{Duration, SystemTime};
        use uuid::Uuid;

        let temp_dir = std::env::temp_dir().join(format!("test_grants_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;

        let storage = FileSystemGrantStorage::new(temp_dir.clone()).await?;

        let primitives = CoreCapabilities::CAP_KEYBOARD_WRITE
            | CoreCapabilities::CAP_POINTER_WRITE
            | CoreCapabilities::CAP_SCREEN_READ;

        let restrictions = CapabilityRestrictions {
            max_events_per_second: Some(100),
            expires_at: Some(SystemTime::now() + Duration::from_secs(3600)),
            ..Default::default()
        };

        let grant = CapabilityGrant {
            id: Uuid::new_v4(),
            app_id: "test-app".to_string(),
            primitives,
            dynamic_capabilities: Default::default(),
            granted_at: SystemTime::now(),
            expires_at: None,
            reason: Some("Test grant".to_string()),
            restrictions,
            usage: GrantUsageStats::default(),
        };

        // Test save
        storage.save(&grant).await?;

        // Test load
        let loaded = storage.load(&grant.id).await?;
        assert_eq!(loaded.id, grant.id);
        assert_eq!(loaded.app_id, grant.app_id);

        // Test list
        let grants = storage.load_for_app("test-app").await?;
        assert_eq!(grants.len(), 1);

        // Test delete
        storage.delete(&grant.id).await?;
        let result = storage.load(&grant.id).await;
        assert!(result.is_err());

        // Cleanup
        std::fs::remove_dir_all(&temp_dir)?;

        Ok(())
    }

    /// Test agnostic username derivation patterns
    #[test]
    fn test_username_patterns() {
        let test_cases = vec![
            ("web-01", "web01"),
            ("db-primary", "dbprimary"),
            ("load-balancer-1", "loadbalancer1"),
            ("cache_server-02", "cacheserver02"),
            ("api-gateway", "apigateway"),
            ("rustdesk-20251228-115834", "rustdesk20251228115834"),
        ];

        for (input, expected) in test_cases {
            let username: String = input
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase();
            assert_eq!(username, expected, "Failed for input: {}", input);
        }
    }
}
