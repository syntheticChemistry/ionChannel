//! Test configuration from environment
//!
//! Provides environment-driven test configuration following primal philosophy:
//! - Self-knowledge: Test knows what it needs
//! - Runtime discovery: Values from environment
//! - Sensible defaults: Common test scenarios

use std::net::IpAddr;

/// Get test VM IP from environment or use common default
pub fn test_vm_ip() -> String {
    std::env::var("TEST_VM_IP").unwrap_or_else(|_| "192.168.122.100".to_string())
}

/// Get test VM IP with specific default
pub fn test_vm_ip_or(default: &str) -> String {
    std::env::var("TEST_VM_IP").unwrap_or_else(|_| default.to_string())
}

/// Get test SSH user from environment
pub fn test_ssh_user() -> String {
    std::env::var("TEST_SSH_USER").unwrap_or_else(|_| "iontest".to_string())
}

/// Get test SSH key path from environment
pub fn test_ssh_key() -> String {
    std::env::var("TEST_SSH_KEY")
        .unwrap_or_else(|_| "~/.ssh/id_rsa".to_string())
}

/// Parse IP from environment, validate format
pub fn test_vm_ip_parsed() -> Result<IpAddr, std::net::AddrParseError> {
    test_vm_ip().parse()
}

/// Get test RustDesk ID (for examples/docs)
pub fn test_rustdesk_id() -> String {
    std::env::var("TEST_RUSTDESK_ID")
        .unwrap_or_else(|_| "123456789".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_ip() {
        let ip = test_vm_ip();
        assert!(!ip.is_empty());
        assert!(ip.contains('.'));
    }

    #[test]
    fn test_ip_parsing() {
        let ip = test_vm_ip_parsed();
        assert!(ip.is_ok());
    }

    #[test]
    fn test_custom_default() {
        let ip = test_vm_ip_or("10.0.0.1");
        assert!(!ip.is_empty());
    }
}

