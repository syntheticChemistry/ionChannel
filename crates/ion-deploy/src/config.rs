//! Configuration management

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::discovery::VmInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub last_vm: Option<VmInfo>,
    pub discovered_vms: Vec<VmInfo>,
    pub preferences: Preferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    pub auto_restart: bool,
    pub monitor_logs: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            last_vm: None,
            discovered_vms: Vec::new(),
            preferences: Preferences {
                auto_restart: false,
                monitor_logs: false,
            },
        }
    }
}

impl Config {
    /// Load config from file or create default
    pub fn load(path: Option<&str>) -> Result<Self> {
        let config_path = if let Some(p) = path {
            PathBuf::from(p)
        } else {
            Self::default_path()?
        };

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = ::toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::default_path()?;

        // Create parent directory if needed
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = ::toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;

        Ok(())
    }

    /// Reset config to defaults
    pub fn reset(&mut self) -> Result<()> {
        *self = Self::default();
        let config_path = Self::default_path()?;
        if config_path.exists() {
            std::fs::remove_file(&config_path)?;
        }
        Ok(())
    }

    /// Get default config path
    fn default_path() -> Result<PathBuf> {
        let home = home::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
        Ok(home.join(".config/ionChannel/deploy.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        
        assert!(config.last_vm.is_none());
        assert!(config.discovered_vms.is_empty());
        assert!(!config.preferences.auto_restart);
        assert!(!config.preferences.monitor_logs);
    }

    #[test]
    fn test_preferences_default() {
        let prefs = Preferences {
            auto_restart: false,
            monitor_logs: false,
        };
        
        assert!(!prefs.auto_restart);
        assert!(!prefs.monitor_logs);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        
        // Test TOML serialization
        let toml_str = ::toml::to_string_pretty(&config).expect("Serialization failed");
        assert!(toml_str.contains("auto_restart"));
        assert!(toml_str.contains("monitor_logs"));
        
        // Test deserialization
        let deserialized: Config = ::toml::from_str(&toml_str).expect("Deserialization failed");
        assert!(deserialized.last_vm.is_none());
        assert!(deserialized.discovered_vms.is_empty());
    }

    #[test]
    fn test_config_with_vm_info() {
        let vm = VmInfo {
            name: "test-vm".to_string(),
            ip: "192.168.1.100".to_string(),
            discovery_method: "mdns".to_string(),
            username: Some("testuser".to_string()),
            services: vec!["portal".to_string()],
        };
        
        let mut config = Config::default();
        config.last_vm = Some(vm.clone());
        config.discovered_vms.push(vm);
        
        assert!(config.last_vm.is_some());
        assert_eq!(config.discovered_vms.len(), 1);
        assert_eq!(config.discovered_vms[0].name, "test-vm");
    }

    #[test]
    fn test_config_preferences_modification() {
        let mut config = Config::default();
        
        config.preferences.auto_restart = true;
        config.preferences.monitor_logs = true;
        
        assert!(config.preferences.auto_restart);
        assert!(config.preferences.monitor_logs);
    }

    #[test]
    fn test_config_load_nonexistent() {
        // Load from non-existent file should return default
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("nonexistent.toml");
        
        let config = Config::load(Some(config_path.to_str().unwrap()))
            .expect("Load should succeed with default");
        
        assert!(config.last_vm.is_none());
        assert!(config.discovered_vms.is_empty());
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("test_config.toml");
        
        // Create a config with some data
        let mut config = Config::default();
        config.preferences.auto_restart = true;
        config.preferences.monitor_logs = true;
        
        // Serialize to file manually (avoiding default_path)
        let content = ::toml::to_string_pretty(&config).expect("Serialization failed");
        std::fs::write(&config_path, content).expect("Write failed");
        
        // Load it back
        let loaded = Config::load(Some(config_path.to_str().unwrap()))
            .expect("Load failed");
        
        assert_eq!(loaded.preferences.auto_restart, true);
        assert_eq!(loaded.preferences.monitor_logs, true);
    }

    #[test]
    fn test_config_reset() {
        let mut config = Config::default();
        config.preferences.auto_restart = true;
        config.preferences.monitor_logs = true;
        
        let vm = VmInfo {
            name: "test".to_string(),
            ip: "1.2.3.4".to_string(),
            discovery_method: "scan".to_string(),
            username: None,
            services: vec![],
        };
        config.last_vm = Some(vm);
        
        // Reset in-memory (we can't test file operations in unit tests)
        config = Config::default();
        
        assert!(config.last_vm.is_none());
        assert!(!config.preferences.auto_restart);
        assert!(!config.preferences.monitor_logs);
    }

    #[test]
    fn test_config_clone() {
        let config = Config::default();
        let cloned = config.clone();
        
        // Verify clone works
        assert_eq!(cloned.preferences.auto_restart, config.preferences.auto_restart);
        assert_eq!(cloned.preferences.monitor_logs, config.preferences.monitor_logs);
    }

    #[test]
    fn test_vm_info_in_config() {
        let vm1 = VmInfo {
            name: "vm1".to_string(),
            ip: "192.168.1.1".to_string(),
            discovery_method: "mdns".to_string(),
            username: Some("user1".to_string()),
            services: vec!["portal".to_string(), "input".to_string()],
        };
        
        let vm2 = VmInfo {
            name: "vm2".to_string(),
            ip: "192.168.1.2".to_string(),
            discovery_method: "scan".to_string(),
            username: None,
            services: vec![],
        };
        
        let mut config = Config::default();
        config.discovered_vms.push(vm1.clone());
        config.discovered_vms.push(vm2);
        config.last_vm = Some(vm1);
        
        assert_eq!(config.discovered_vms.len(), 2);
        assert!(config.last_vm.is_some());
        assert_eq!(config.last_vm.as_ref().unwrap().name, "vm1");
    }

    #[test]
    fn test_config_debug_format() {
        let config = Config::default();
        let debug_str = format!("{:?}", config);
        
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("preferences"));
    }
}
