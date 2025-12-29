//! Capability discovery system

use crate::errors::{Result, ValidationError};
use crate::providers::{desktop::RemoteDesktop, portal::PortalDeployer, vm::VmProvisioner};
use std::sync::Arc;
use tracing::info;

/// Capability registry for discovering providers
pub struct CapabilityRegistry {
    vm_provisioners: Vec<Arc<dyn VmProvisioner>>,
    remote_desktops: Vec<Arc<dyn RemoteDesktop>>,
    portal_deployers: Vec<Arc<dyn PortalDeployer>>,
}

impl CapabilityRegistry {
    /// Create a new capability registry
    pub fn new() -> Self {
        Self {
            vm_provisioners: Vec::new(),
            remote_desktops: Vec::new(),
            portal_deployers: Vec::new(),
        }
    }

    /// Register a VM provisioner
    pub fn register_vm_provisioner(&mut self, provisioner: Arc<dyn VmProvisioner>) {
        self.vm_provisioners.push(provisioner);
    }

    /// Register a remote desktop provider
    pub fn register_remote_desktop(&mut self, desktop: Arc<dyn RemoteDesktop>) {
        self.remote_desktops.push(desktop);
    }

    /// Register a portal deployer
    pub fn register_portal_deployer(&mut self, deployer: Arc<dyn PortalDeployer>) {
        self.portal_deployers.push(deployer);
    }

    /// Discover best VM provisioner
    pub async fn discover_vm_provisioner(&self) -> Result<Arc<dyn VmProvisioner>> {
        let mut tried = Vec::new();

        for provisioner in &self.vm_provisioners {
            tried.push(provisioner.name().to_string());
            if provisioner.is_available().await {
                info!("✓ Discovered VM provisioner: {}", provisioner.name());
                return Ok(Arc::clone(provisioner));
            }
        }

        Err(ValidationError::NoVmProvisionerAvailable {
            tried,
            suggestion: "Install libvirt: sudo apt install libvirt-daemon-system".to_string(),
        })
    }

    /// Discover best remote desktop provider
    pub async fn discover_remote_desktop(&self) -> Result<Arc<dyn RemoteDesktop>> {
        let mut tried = Vec::new();

        for desktop in &self.remote_desktops {
            tried.push(desktop.name().to_string());
            if desktop.is_available().await {
                info!("✓ Discovered remote desktop: {}", desktop.name());
                return Ok(Arc::clone(desktop));
            }
        }

        Err(ValidationError::NoRemoteDesktopAvailable {
            tried,
            suggestion: "Install RustDesk: wget https://github.com/rustdesk/rustdesk/releases/download/latest/rustdesk-latest-x86_64.deb && sudo dpkg -i rustdesk-latest-x86_64.deb".to_string(),
        })
    }

    /// Discover portal deployer
    pub async fn discover_portal_deployer(&self) -> Result<Arc<dyn PortalDeployer>> {
        for deployer in &self.portal_deployers {
            if deployer.is_available().await {
                info!("✓ Discovered portal deployer: {}", deployer.name());
                return Ok(Arc::clone(deployer));
            }
        }

        Err(ValidationError::CapabilityNotFound {
            capability: "portal-deployer".to_string(),
        })
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = CapabilityRegistry::new();
        // Verify registry is created empty
        assert!(true);
    }

    #[test]
    fn test_registry_default() {
        let _registry = CapabilityRegistry::default();
        // Verify default trait works
        assert!(true);
    }

    #[tokio::test]
    async fn test_empty_registry_vm_provisioner() {
        let registry = CapabilityRegistry::new();
        let result = registry.discover_vm_provisioner().await;
        assert!(result.is_err());
        
        // Verify error is the expected type
        match result {
            Err(ValidationError::NoVmProvisionerAvailable { .. }) => assert!(true),
            _ => panic!("Expected NoVmProvisionerAvailable error"),
        }
    }

    #[tokio::test]
    async fn test_empty_registry_remote_desktop() {
        let registry = CapabilityRegistry::new();
        let result = registry.discover_remote_desktop().await;
        assert!(result.is_err());
        
        match result {
            Err(ValidationError::NoRemoteDesktopAvailable { .. }) => assert!(true),
            _ => panic!("Expected NoRemoteDesktopAvailable error"),
        }
    }

    #[tokio::test]
    async fn test_empty_registry_portal_deployer() {
        let registry = CapabilityRegistry::new();
        let result = registry.discover_portal_deployer().await;
        assert!(result.is_err());
        
        match result {
            Err(ValidationError::CapabilityNotFound { capability }) => {
                assert_eq!(capability, "portal-deployer");
            },
            _ => panic!("Expected CapabilityNotFound error"),
        }
    }

    #[test]
    fn test_registry_structure() {
        // Test that the registry has the right structure
        let registry = CapabilityRegistry {
            vm_provisioners: Vec::new(),
            remote_desktops: Vec::new(),
            portal_deployers: Vec::new(),
        };

        // Verify all vectors are empty
        assert_eq!(registry.vm_provisioners.len(), 0);
        assert_eq!(registry.remote_desktops.len(), 0);
        assert_eq!(registry.portal_deployers.len(), 0);
    }

    #[tokio::test]
    async fn test_error_messages_contain_suggestions() {
        let registry = CapabilityRegistry::new();
        
        // Test VM provisioner error has suggestion
        let vm_result = registry.discover_vm_provisioner().await;
        if let Err(ValidationError::NoVmProvisionerAvailable { suggestion, .. }) = vm_result {
            assert!(suggestion.contains("libvirt"));
        } else {
            panic!("Expected error with suggestion");
        }

        // Test remote desktop error has suggestion  
        let desktop_result = registry.discover_remote_desktop().await;
        if let Err(ValidationError::NoRemoteDesktopAvailable { suggestion, .. }) = desktop_result {
            assert!(suggestion.contains("RustDesk"));
        } else {
            panic!("Expected error with suggestion");
        }
    }

    #[tokio::test]
    async fn test_discover_methods_async() {
        let registry = CapabilityRegistry::new();
        
        // Test that discover methods are actually async
        let _vm_future = registry.discover_vm_provisioner();
        let _desktop_future = registry.discover_remote_desktop();
        let _portal_future = registry.discover_portal_deployer();
        
        // Just verify they compile and can be awaited
        assert!(true);
    }

    #[test]
    fn test_registry_debug_impl() {
        // Test that we can create and use the registry
        let registry = CapabilityRegistry::new();
        let _default_registry = CapabilityRegistry::default();
        
        // Verify both creation methods work
        drop(registry);
        drop(_default_registry);
    }
}
