//! Observable validation events for AI agents

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

/// Observable validation event
///
/// These events are emitted during validation execution and can be consumed
/// by AI agents, MCP clients, or human operators to monitor progress.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ValidationEvent {
    /// Validation started
    Started {
        timestamp: DateTime<Utc>,
        plan_id: String,
    },

    /// VM provisioning started
    ProvisioningStarted {
        timestamp: DateTime<Utc>,
        vm_name: String,
    },

    /// VM successfully provisioned
    VmProvisioned {
        timestamp: DateTime<Utc>,
        vm_id: String,
        vm_name: String,
        ip: String,
        duration: Duration,
    },

    /// SSH connection established
    SshConnected {
        timestamp: DateTime<Utc>,
        host: String,
        port: u16,
    },

    /// Package installation started
    InstallingPackage {
        timestamp: DateTime<Utc>,
        package: String,
    },

    /// Package successfully installed
    PackageInstalled {
        timestamp: DateTime<Utc>,
        package: String,
        version: String,
    },

    /// Remote desktop ready with ID
    RemoteDesktopReady {
        timestamp: DateTime<Utc>,
        desktop_id: String,
    },

    /// Portal deployment started
    DeployingPortal {
        timestamp: DateTime<Utc>,
        target: String,
    },

    /// Portal successfully deployed
    PortalDeployed {
        timestamp: DateTime<Utc>,
        deployment_id: String,
        services: Vec<String>,
    },

    /// Verification complete
    VerificationComplete {
        timestamp: DateTime<Utc>,
        success: bool,
        details: String,
    },

    /// Service deployment started
    DeployingService {
        timestamp: DateTime<Utc>,
        service: String,
        components: Vec<String>,
    },

    /// Service successfully started
    ServiceStarted {
        timestamp: DateTime<Utc>,
        service: String,
        endpoint: Option<Url>,
        pid: Option<u32>,
    },

    /// Health check performed
    HealthCheck {
        timestamp: DateTime<Utc>,
        service: String,
        healthy: bool,
        details: Option<String>,
    },

    /// Progress update
    Progress {
        timestamp: DateTime<Utc>,
        phase: u8,
        phase_name: String,
        progress_percent: u8,
        message: String,
    },

    /// Phase completed successfully
    PhaseComplete {
        timestamp: DateTime<Utc>,
        phase: u8,
        phase_name: String,
        duration: Duration,
    },

    /// Warning occurred (non-fatal)
    Warning {
        timestamp: DateTime<Utc>,
        message: String,
        context: Option<String>,
    },

    /// Error occurred
    Error {
        timestamp: DateTime<Utc>,
        phase: u8,
        error_type: String,
        message: String,
        retryable: bool,
        suggestion: Option<String>,
    },

    /// Full validation complete
    Complete {
        timestamp: DateTime<Utc>,
        rustdesk_id: String,
        total_duration: Duration,
        phases_completed: u8,
        metrics: ValidationMetrics,
    },
}

/// Validation metrics for AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMetrics {
    /// Total duration of validation
    pub total_duration: Duration,
    /// Time spent on VM provisioning
    pub provisioning_duration: Duration,
    /// Time spent on package installation
    pub installation_duration: Duration,
    /// Time spent on service deployment
    pub deployment_duration: Duration,
    /// Time spent on health checks
    pub verification_duration: Duration,
    /// Number of retries performed
    pub retries: u32,
    /// Peak memory usage (MB)
    pub peak_memory_mb: Option<u64>,
}

impl ValidationEvent {
    /// Get the timestamp of the event
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::Started { timestamp, .. }
            | Self::ProvisioningStarted { timestamp, .. }
            | Self::VmProvisioned { timestamp, .. }
            | Self::SshConnected { timestamp, .. }
            | Self::InstallingPackage { timestamp, .. }
            | Self::PackageInstalled { timestamp, .. }
            | Self::RemoteDesktopReady { timestamp, .. }
            | Self::DeployingPortal { timestamp, .. }
            | Self::PortalDeployed { timestamp, .. }
            | Self::VerificationComplete { timestamp, .. }
            | Self::DeployingService { timestamp, .. }
            | Self::ServiceStarted { timestamp, .. }
            | Self::HealthCheck { timestamp, .. }
            | Self::Progress { timestamp, .. }
            | Self::PhaseComplete { timestamp, .. }
            | Self::Warning { timestamp, .. }
            | Self::Error { timestamp, .. }
            | Self::Complete { timestamp, .. } => *timestamp,
        }
    }

    /// Check if this is an error event
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }

    /// Check if this is a completion event
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Complete { .. })
    }

    /// Get human-readable description
    pub fn description(&self) -> String {
        match self {
            Self::Started { .. } => "Validation started".to_string(),
            Self::ProvisioningStarted { vm_name, .. } => {
                format!("Provisioning VM: {}", vm_name)
            },
            Self::VmProvisioned { vm_id, ip, .. } => {
                format!("VM provisioned: {} at {}", vm_id, ip)
            },
            Self::SshConnected { host, port, .. } => {
                format!("SSH connected: {}:{}", host, port)
            },
            Self::InstallingPackage { package, .. } => format!("Installing: {}", package),
            Self::PackageInstalled {
                package, version, ..
            } => {
                format!("Installed: {} {}", package, version)
            },
            Self::RemoteDesktopReady { desktop_id, .. } => {
                format!("Remote Desktop ready: {}", desktop_id)
            },
            Self::DeployingPortal { target, .. } => format!("Deploying portal to {}", target),
            Self::PortalDeployed { services, .. } => {
                format!("Portal deployed: {} services", services.len())
            },
            Self::VerificationComplete {
                success, details, ..
            } => {
                format!(
                    "Verification: {} - {}",
                    if *success { "SUCCESS" } else { "FAILED" },
                    details
                )
            },
            Self::DeployingService { service, .. } => format!("Deploying: {}", service),
            Self::ServiceStarted { service, .. } => format!("Started: {}", service),
            Self::HealthCheck {
                service, healthy, ..
            } => {
                format!(
                    "Health check {}: {}",
                    service,
                    if *healthy { "✓" } else { "✗" }
                )
            },
            Self::Progress { message, .. } => message.clone(),
            Self::PhaseComplete { phase_name, .. } => format!("Phase complete: {}", phase_name),
            Self::Warning { message, .. } => format!("Warning: {}", message),
            Self::Error { message, .. } => format!("Error: {}", message),
            Self::Complete { rustdesk_id, .. } => format!("Complete! RustDesk ID: {}", rustdesk_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "mcp")]
    fn test_event_serialization() {
        use crate::test_config;
        
        let event = ValidationEvent::VmProvisioned {
            timestamp: Utc::now(),
            vm_id: "test-vm".to_string(),
            vm_name: "iontest".to_string(),
            ip: test_config::get_test_vm_ip(),
            duration: Duration::from_secs(30),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("vm_provisioned"));
        assert!(json.contains("test-vm"));
    }

    #[test]
    fn test_event_description() {
        let event = ValidationEvent::PackageInstalled {
            timestamp: Utc::now(),
            package: "rustdesk".to_string(),
            version: "1.2.3".to_string(),
        };

        let desc = event.description();
        assert!(desc.contains("rustdesk"));
    }

    #[test]
    fn test_validation_metrics() {
        let metrics = ValidationMetrics {
            total_duration: Duration::from_secs(120),
            provisioning_duration: Duration::from_secs(30),
            installation_duration: Duration::from_secs(40),
            deployment_duration: Duration::from_secs(25),
            verification_duration: Duration::from_secs(25),
            retries: 2,
            peak_memory_mb: Some(512),
        };

        assert_eq!(metrics.total_duration.as_secs(), 120);
        assert_eq!(metrics.retries, 2);
        assert_eq!(metrics.peak_memory_mb, Some(512));
    }

    #[test]
    fn test_event_started() {
        let event = ValidationEvent::Started {
            timestamp: Utc::now(),
            plan_id: "plan-123".to_string(),
        };

        assert!(!event.is_error());
        assert!(!event.is_complete());
        assert!(event.description().contains("started"));
    }

    #[test]
    fn test_event_ssh_connected() {
        let event = ValidationEvent::SshConnected {
            timestamp: Utc::now(),
            host: "192.168.1.100".to_string(),
            port: 22,
        };

        let desc = event.description();
        assert!(desc.contains("192.168.1.100"));
        assert!(desc.contains("22"));
    }

    #[test]
    fn test_event_portal_deployed() {
        let event = ValidationEvent::PortalDeployed {
            timestamp: Utc::now(),
            deployment_id: "deploy-456".to_string(),
            services: vec!["portal".to_string(), "input".to_string()],
        };

        let desc = event.description();
        assert!(desc.contains("2 services"));
    }

    #[test]
    fn test_event_health_check_healthy() {
        let event = ValidationEvent::HealthCheck {
            timestamp: Utc::now(),
            service: "portal".to_string(),
            healthy: true,
            details: Some("All systems operational".to_string()),
        };

        let desc = event.description();
        assert!(desc.contains("✓"));
    }

    #[test]
    fn test_event_health_check_unhealthy() {
        let event = ValidationEvent::HealthCheck {
            timestamp: Utc::now(),
            service: "portal".to_string(),
            healthy: false,
            details: None,
        };

        let desc = event.description();
        assert!(desc.contains("✗"));
    }

    #[test]
    fn test_event_error() {
        let event = ValidationEvent::Error {
            timestamp: Utc::now(),
            phase: 2,
            error_type: "NetworkError".to_string(),
            message: "Connection timeout".to_string(),
            retryable: true,
            suggestion: Some("Check network connectivity".to_string()),
        };

        assert!(event.is_error());
        assert!(!event.is_complete());
        assert!(event.description().contains("Connection timeout"));
    }

    #[test]
    fn test_event_complete() {
        let metrics = ValidationMetrics {
            total_duration: Duration::from_secs(180),
            provisioning_duration: Duration::from_secs(60),
            installation_duration: Duration::from_secs(50),
            deployment_duration: Duration::from_secs(40),
            verification_duration: Duration::from_secs(30),
            retries: 0,
            peak_memory_mb: Some(1024),
        };

        let event = ValidationEvent::Complete {
            timestamp: Utc::now(),
            rustdesk_id: "rustdesk-789".to_string(),
            total_duration: Duration::from_secs(180),
            phases_completed: 5,
            metrics,
        };

        assert!(event.is_complete());
        assert!(!event.is_error());
        let desc = event.description();
        assert!(desc.contains("rustdesk-789"));
    }

    #[test]
    fn test_event_timestamp() {
        let now = Utc::now();
        let event = ValidationEvent::Progress {
            timestamp: now,
            phase: 1,
            phase_name: "Provisioning".to_string(),
            progress_percent: 50,
            message: "Half way there".to_string(),
        };

        assert_eq!(event.timestamp(), now);
    }

    #[test]
    fn test_event_warning() {
        let event = ValidationEvent::Warning {
            timestamp: Utc::now(),
            message: "Slow connection detected".to_string(),
            context: Some("SSH".to_string()),
        };

        assert!(!event.is_error());
        let desc = event.description();
        assert!(desc.contains("Warning"));
        assert!(desc.contains("Slow connection"));
    }

    #[test]
    fn test_event_phase_complete() {
        let event = ValidationEvent::PhaseComplete {
            timestamp: Utc::now(),
            phase: 1,
            phase_name: "Provisioning".to_string(),
            duration: Duration::from_secs(45),
        };

        let desc = event.description();
        assert!(desc.contains("Provisioning"));
    }

    #[test]
    fn test_event_service_started() {
        use std::str::FromStr;
        let endpoint = Url::from_str("http://localhost:8080").unwrap();
        
        let event = ValidationEvent::ServiceStarted {
            timestamp: Utc::now(),
            service: "ion-portal".to_string(),
            endpoint: Some(endpoint),
            pid: Some(12345),
        };

        let desc = event.description();
        assert!(desc.contains("ion-portal"));
    }

    #[test]
    fn test_event_verification_complete_success() {
        let event = ValidationEvent::VerificationComplete {
            timestamp: Utc::now(),
            success: true,
            details: "All tests passed".to_string(),
        };

        let desc = event.description();
        assert!(desc.contains("SUCCESS"));
        assert!(desc.contains("All tests passed"));
    }

    #[test]
    fn test_event_verification_complete_failure() {
        let event = ValidationEvent::VerificationComplete {
            timestamp: Utc::now(),
            success: false,
            details: "Connection failed".to_string(),
        };

        let desc = event.description();
        assert!(desc.contains("FAILED"));
    }

    #[test]
    fn test_event_installing_package() {
        let event = ValidationEvent::InstallingPackage {
            timestamp: Utc::now(),
            package: "cosmic-comp".to_string(),
        };

        let desc = event.description();
        assert!(desc.contains("cosmic-comp"));
    }

    #[test]
    fn test_event_deploying_service() {
        let event = ValidationEvent::DeployingService {
            timestamp: Utc::now(),
            service: "portal".to_string(),
            components: vec!["backend".to_string(), "frontend".to_string()],
        };

        let desc = event.description();
        assert!(desc.contains("portal"));
    }

    #[test]
    fn test_validation_metrics_clone() {
        let metrics = ValidationMetrics {
            total_duration: Duration::from_secs(100),
            provisioning_duration: Duration::from_secs(25),
            installation_duration: Duration::from_secs(25),
            deployment_duration: Duration::from_secs(25),
            verification_duration: Duration::from_secs(25),
            retries: 1,
            peak_memory_mb: None,
        };

        let cloned = metrics.clone();
        assert_eq!(cloned.retries, 1);
        assert_eq!(cloned.total_duration, Duration::from_secs(100));
    }

    #[test]
    fn test_event_clone() {
        let event = ValidationEvent::Started {
            timestamp: Utc::now(),
            plan_id: "test".to_string(),
        };

        let _cloned = event.clone();
        // Just verify clone works
    }

    #[test]
    fn test_event_debug() {
        let event = ValidationEvent::Progress {
            timestamp: Utc::now(),
            phase: 2,
            phase_name: "Installation".to_string(),
            progress_percent: 75,
            message: "Almost done".to_string(),
        };

        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("Progress"));
    }
}
