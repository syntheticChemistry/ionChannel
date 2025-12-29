//! Persistent storage for capability grants
//!
//! This module provides interfaces and implementations for storing
//! and retrieving capability grants persistently across sessions.

use super::grant::CapabilityGrant;
use super::registry::Capability;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Trait for persisting capability grants
#[async_trait]
pub trait GrantStorage: Send + Sync {
    /// Save a grant to persistent storage
    ///
    /// # Errors
    /// Returns error if storage operation fails
    async fn save(&self, grant: &CapabilityGrant) -> Result<(), StorageError>;

    /// Load a specific grant by ID
    ///
    /// # Errors
    /// Returns error if grant not found or storage operation fails
    async fn load(&self, id: &uuid::Uuid) -> Result<CapabilityGrant, StorageError>;

    /// Load all grants for an application
    async fn load_for_app(&self, app_id: &str) -> Result<Vec<CapabilityGrant>, StorageError>;

    /// Load all grants matching a capability
    async fn load_matching(
        &self,
        capability: &Capability,
    ) -> Result<Vec<CapabilityGrant>, StorageError>;

    /// Delete a grant
    ///
    /// # Errors
    /// Returns error if grant not found or storage operation fails
    async fn delete(&self, id: &uuid::Uuid) -> Result<(), StorageError>;

    /// List all stored grant IDs
    async fn list_ids(&self) -> Result<Vec<uuid::Uuid>, StorageError>;

    /// Clean up expired grants
    async fn cleanup_expired(&self) -> Result<usize, StorageError>;
}

/// File system based grant storage
///
/// Stores grants as JSON files in a directory structure:
/// ```text
/// grants/
///   app-id/
///     grant-uuid.json
///     grant-uuid.json
/// ```
pub struct FileSystemGrantStorage {
    /// Base directory for grants
    base_path: PathBuf,
}

impl FileSystemGrantStorage {
    /// Create a new file system storage
    ///
    /// # Errors
    /// Returns error if directory cannot be created
    pub async fn new(base_path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let base_path = base_path.as_ref().to_path_buf();

        // Ensure directory exists
        fs::create_dir_all(&base_path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        Ok(Self { base_path })
    }

    /// Get directory for an application's grants
    fn app_dir(&self, app_id: &str) -> PathBuf {
        self.base_path.join(app_id)
    }

    /// Get path for a specific grant file
    fn grant_path(&self, app_id: &str, grant_id: &uuid::Uuid) -> PathBuf {
        self.app_dir(app_id).join(format!("{}.json", grant_id))
    }

    /// Ensure app directory exists
    async fn ensure_app_dir(&self, app_id: &str) -> Result<(), StorageError> {
        let dir = self.app_dir(app_id);
        fs::create_dir_all(&dir)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl GrantStorage for FileSystemGrantStorage {
    async fn save(&self, grant: &CapabilityGrant) -> Result<(), StorageError> {
        let app_id = &grant.app_id;
        self.ensure_app_dir(app_id).await?;

        let path = self.grant_path(app_id, &grant.id);

        // Serialize grant to JSON
        let json = serde_json::to_string_pretty(grant)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        // Write atomically using temp file
        let temp_path = path.with_extension("tmp");
        let mut file = fs::File::create(&temp_path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        file.write_all(json.as_bytes())
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        file.flush()
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        drop(file);

        // Atomic rename
        fs::rename(temp_path, path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        Ok(())
    }

    async fn load(&self, id: &uuid::Uuid) -> Result<CapabilityGrant, StorageError> {
        // Need to search all app directories
        let mut entries = fs::read_dir(&self.base_path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?
        {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let grant_file = path.join(format!("{}.json", id));
            if grant_file.exists() {
                let content = fs::read_to_string(&grant_file)
                    .await
                    .map_err(|e| StorageError::IoError(e.to_string()))?;

                let grant: CapabilityGrant = serde_json::from_str(&content)
                    .map_err(|e| StorageError::DeserializationError(e.to_string()))?;

                return Ok(grant);
            }
        }

        Err(StorageError::NotFound(id.to_string()))
    }

    async fn load_for_app(&self, app_id: &str) -> Result<Vec<CapabilityGrant>, StorageError> {
        let app_dir = self.app_dir(app_id);

        if !app_dir.exists() {
            return Ok(Vec::new());
        }

        let mut grants = Vec::new();
        let mut entries = fs::read_dir(&app_dir)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            let content = fs::read_to_string(&path)
                .await
                .map_err(|e| StorageError::IoError(e.to_string()))?;

            match serde_json::from_str::<CapabilityGrant>(&content) {
                Ok(grant) => grants.push(grant),
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to deserialize grant {}: {}",
                        path.display(),
                        e
                    );
                    continue;
                },
            }
        }

        Ok(grants)
    }

    async fn load_matching(
        &self,
        capability: &Capability,
    ) -> Result<Vec<CapabilityGrant>, StorageError> {
        let mut matching_grants = Vec::new();
        let mut entries = fs::read_dir(&self.base_path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?
        {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            // Load all grants in this app directory
            if let Some(app_id) = path.file_name().and_then(|s| s.to_str()) {
                let grants = self.load_for_app(app_id).await?;

                for grant in grants {
                    // Check if grant has this capability
                    if grant
                        .dynamic_capabilities
                        .iter()
                        .any(|c| c == &capability.to_string())
                    {
                        matching_grants.push(grant);
                    }
                }
            }
        }

        Ok(matching_grants)
    }

    async fn delete(&self, id: &uuid::Uuid) -> Result<(), StorageError> {
        // Search for the grant file
        let mut entries = fs::read_dir(&self.base_path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?
        {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let grant_file = path.join(format!("{}.json", id));
            if grant_file.exists() {
                fs::remove_file(&grant_file)
                    .await
                    .map_err(|e| StorageError::IoError(e.to_string()))?;
                return Ok(());
            }
        }

        Err(StorageError::NotFound(id.to_string()))
    }

    async fn list_ids(&self) -> Result<Vec<uuid::Uuid>, StorageError> {
        let mut ids = Vec::new();
        let mut entries = fs::read_dir(&self.base_path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?
        {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let mut grant_entries = fs::read_dir(&path)
                .await
                .map_err(|e| StorageError::IoError(e.to_string()))?;

            while let Some(grant_entry) = grant_entries
                .next_entry()
                .await
                .map_err(|e| StorageError::IoError(e.to_string()))?
            {
                let grant_path = grant_entry.path();
                if grant_path.extension().and_then(|s| s.to_str()) != Some("json") {
                    continue;
                }

                if let Some(stem) = grant_path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(id) = uuid::Uuid::parse_str(stem) {
                        ids.push(id);
                    }
                }
            }
        }

        Ok(ids)
    }

    async fn cleanup_expired(&self) -> Result<usize, StorageError> {
        let mut cleaned = 0;

        let ids = self.list_ids().await?;

        for id in ids {
            match self.load(&id).await {
                Ok(grant) => {
                    if grant.is_expired() {
                        self.delete(&id).await?;
                        cleaned += 1;
                    }
                },
                Err(_) => continue,
            }
        }

        Ok(cleaned)
    }
}

/// Storage errors
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// I/O error
    #[error("I/O error: {0}")]
    IoError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// Grant not found
    #[error("Grant not found: {0}")]
    NotFound(String),
}

#[cfg(test)]
mod tests {
    use super::super::primitives::CoreCapabilities;
    use super::*;
    use std::time::{Duration, SystemTime};
    use tempfile::TempDir;

    async fn create_test_grant(app_id: &str) -> CapabilityGrant {
        CapabilityGrant::new(
            app_id.to_string(),
            CoreCapabilities::CAP_KEYBOARD_WRITE | CoreCapabilities::CAP_POINTER_WRITE,
        )
    }

    #[tokio::test]
    async fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileSystemGrantStorage::new(temp_dir.path()).await.unwrap();

        let grant = create_test_grant("test_app").await;
        let grant_id = grant.id;

        // Save
        storage.save(&grant).await.unwrap();

        // Load
        let loaded = storage.load(&grant_id).await.unwrap();
        assert_eq!(loaded.id, grant_id);
        assert_eq!(loaded.app_id, "test_app");
    }

    #[tokio::test]
    async fn test_load_for_app() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileSystemGrantStorage::new(temp_dir.path()).await.unwrap();

        // Save multiple grants for same app
        let grant1 = create_test_grant("test_app").await;
        let grant2 = create_test_grant("test_app").await;

        storage.save(&grant1).await.unwrap();
        storage.save(&grant2).await.unwrap();

        // Load all grants for app
        let grants = storage.load_for_app("test_app").await.unwrap();
        assert_eq!(grants.len(), 2);
    }

    #[tokio::test]
    async fn test_delete() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileSystemGrantStorage::new(temp_dir.path()).await.unwrap();

        let grant = create_test_grant("test_app").await;
        let grant_id = grant.id;

        storage.save(&grant).await.unwrap();
        storage.delete(&grant_id).await.unwrap();

        // Should not be found
        let result = storage.load(&grant_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_ids() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileSystemGrantStorage::new(temp_dir.path()).await.unwrap();

        let grant1 = create_test_grant("app1").await;
        let grant2 = create_test_grant("app2").await;

        storage.save(&grant1).await.unwrap();
        storage.save(&grant2).await.unwrap();

        let ids = storage.list_ids().await.unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&grant1.id));
        assert!(ids.contains(&grant2.id));
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileSystemGrantStorage::new(temp_dir.path()).await.unwrap();

        // Create grant that expires in the past
        let mut grant = create_test_grant("test_app").await;
        grant.restrictions.expires_at = Some(SystemTime::now() - Duration::from_secs(3600));

        storage.save(&grant).await.unwrap();

        // Cleanup should remove it
        let cleaned = storage.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 1);

        // Should not be found
        let result = storage.load(&grant.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_atomic_write() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileSystemGrantStorage::new(temp_dir.path()).await.unwrap();

        let grant = create_test_grant("test_app").await;

        // Save multiple times (should be atomic)
        for _ in 0..10 {
            storage.save(&grant).await.unwrap();
        }

        // Should still load correctly
        let loaded = storage.load(&grant.id).await.unwrap();
        assert_eq!(loaded.id, grant.id);
    }
}
