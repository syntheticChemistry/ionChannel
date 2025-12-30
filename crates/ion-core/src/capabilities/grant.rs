//! Capability grants and restrictions
//!
//! This module defines the structure of capability grants - the actual
//! permissions given to applications, along with any restrictions.

use chrono::{Datelike, Timelike};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashSet;
use std::time::SystemTime;
use uuid::Uuid;

use crate::capabilities::primitives::CoreCapabilities;

// Manual serde impl for CoreCapabilities (bitflags)
impl Serialize for CoreCapabilities {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.bits())
    }
}

impl<'de> Deserialize<'de> for CoreCapabilities {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bits = u64::deserialize(deserializer)?;
        Ok(CoreCapabilities::from_bits_truncate(bits))
    }
}

/// A granted set of capabilities to an application
///
/// This represents an approved permission grant, potentially with
/// restrictions on how and when it can be used.
///
/// # Example
/// ```rust
/// # use ion_core::capabilities::{CapabilityGrant, CoreCapabilities};
/// let grant = CapabilityGrant::new(
///     "org.rustdesk.RustDesk".to_string(),
///     CoreCapabilities::CAP_KEYBOARD_WRITE | CoreCapabilities::CAP_SCREEN_READ,
/// );
/// assert_eq!(grant.app_id, "org.rustdesk.RustDesk");
/// assert!(grant.has_primitive(CoreCapabilities::CAP_KEYBOARD_WRITE));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityGrant {
    /// Unique identifier for this grant
    pub id: Uuid,

    /// Application identifier (D-Bus name, app_id, etc.)
    pub app_id: String,

    /// Granted primitive capabilities (bitflags, fast path)
    pub primitives: CoreCapabilities,

    /// Additional dynamic capabilities (string-based)
    /// For AI agents, plugins, etc. that define custom capabilities
    pub dynamic_capabilities: HashSet<String>,

    /// When this grant was created
    pub granted_at: SystemTime,

    /// When this grant expires (None = session-only)
    pub expires_at: Option<SystemTime>,

    /// User-provided reason/note for this grant
    pub reason: Option<String>,

    /// Restrictions on usage
    pub restrictions: CapabilityRestrictions,

    /// Usage statistics
    pub usage: GrantUsageStats,
}

impl CapabilityGrant {
    /// Create a new grant
    #[must_use]
    pub fn new(app_id: String, primitives: CoreCapabilities) -> Self {
        Self {
            id: Uuid::new_v4(),
            app_id,
            primitives,
            dynamic_capabilities: HashSet::new(),
            granted_at: SystemTime::now(),
            expires_at: None,
            reason: None,
            restrictions: CapabilityRestrictions::default(),
            usage: GrantUsageStats::default(),
        }
    }

    /// Check if grant includes primitive capability (fast bitflag check)
    #[must_use]
    pub fn has_primitive(&self, cap: CoreCapabilities) -> bool {
        self.primitives.contains(cap)
    }

    /// Check if grant includes dynamic capability (string-based)
    #[must_use]
    pub fn has_dynamic(&self, cap: &str) -> bool {
        self.dynamic_capabilities.contains(cap)
    }

    /// Check if grant has expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.restrictions.expires_at {
            SystemTime::now() > expires
        } else {
            false
        }
    }

    /// Add a dynamic capability to this grant
    pub fn add_dynamic(&mut self, cap: String) {
        self.dynamic_capabilities.insert(cap);
    }

    /// Record usage of this grant
    pub fn record_use(&mut self) {
        self.usage.use_count += 1;
        self.usage.last_used = Some(SystemTime::now());
    }
}

/// Restrictions that can be applied to capability grants
///
/// These allow fine-grained control over when and how capabilities
/// can be used, implementing principle of least privilege.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRestrictions {
    /// Maximum events per second (rate limiting)
    pub max_events_per_second: Option<u32>,

    /// Allowed time windows (e.g., "only during work hours")
    pub allowed_time_ranges: Vec<TimeRange>,

    /// Maximum session duration
    pub max_session_duration: Option<std::time::Duration>,

    /// Optional expiration time
    pub expires_at: Option<SystemTime>,

    /// Require confirmation for specific operations
    pub require_confirmation_for: CoreCapabilities,
}

impl Default for CapabilityRestrictions {
    fn default() -> Self {
        Self {
            max_events_per_second: None,
            allowed_time_ranges: Vec::new(),
            max_session_duration: None,
            expires_at: None,
            require_confirmation_for: CoreCapabilities::empty(),
        }
    }
}

/// Time range for allowed access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start time (wall clock)
    pub start: (u8, u8), // (hour, minute)

    /// End time (wall clock)
    pub end: (u8, u8), // (hour, minute)

    /// Which days of week (1 = Monday, 7 = Sunday)
    pub days: Vec<u8>,
}

impl TimeRange {
    /// Check if current time is within this range
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.is_active_at(SystemTime::now())
    }

    /// Check if a specific time is within this range
    #[must_use]
    pub fn is_active_at(&self, time: SystemTime) -> bool {
        // Convert SystemTime to DateTime for proper time checking
        let datetime: chrono::DateTime<chrono::Local> = time.into();
        
        // Check day of week (chrono uses 1-7 for Mon-Sun, same as our format)
        let current_day = datetime.weekday().number_from_monday() as u8;
        if !self.days.is_empty() && !self.days.contains(&current_day) {
            return false;
        }
        
        // Extract current hour and minute
        let current_hour = datetime.hour() as u8;
        let current_minute = datetime.minute() as u8;
        let current_time = (current_hour, current_minute);
        
        // Check if current time is within range
        // Handle cases where end < start (crosses midnight)
        if self.end < self.start {
            // Range crosses midnight: 22:00 - 06:00
            current_time >= self.start || current_time <= self.end
        } else {
            // Normal range: 09:00 - 17:00
            current_time >= self.start && current_time <= self.end
        }
    }

    /// Create a business hours range (9 AM - 5 PM, Monday-Friday)
    #[must_use]
    pub fn business_hours() -> Self {
        Self {
            start: (9, 0),
            end: (17, 0),
            days: vec![1, 2, 3, 4, 5], // Mon-Fri
        }
    }

    /// Create a 24/7 range (always active)
    #[must_use]
    pub fn always() -> Self {
        Self {
            start: (0, 0),
            end: (23, 59),
            days: vec![1, 2, 3, 4, 5, 6, 7], // All days
        }
    }

    /// Create a night hours range (10 PM - 6 AM)
    #[must_use]
    pub fn night_hours() -> Self {
        Self {
            start: (22, 0),
            end: (6, 0),
            days: vec![1, 2, 3, 4, 5, 6, 7], // All days
        }
    }
}

/// Track how grant is being used
///
/// This provides accountability and helps detect anomalies.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GrantUsageStats {
    /// Total times this grant was used
    pub use_count: u64,

    /// Last time grant was used
    pub last_used: Option<SystemTime>,

    /// Total events processed under this grant
    pub total_events: u64,

    /// Times user was warned about unusual usage
    pub warning_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grant_creation() {
        let grant = CapabilityGrant::new(
            "test-app".to_string(),
            CoreCapabilities::CAP_KEYBOARD_WRITE | CoreCapabilities::CAP_SCREEN_READ,
        );

        assert_eq!(grant.app_id, "test-app");
        assert!(grant.has_primitive(CoreCapabilities::CAP_KEYBOARD_WRITE));
        assert!(grant.has_primitive(CoreCapabilities::CAP_SCREEN_READ));
        assert!(!grant.has_primitive(CoreCapabilities::CAP_POINTER_WRITE));
    }

    #[test]
    fn test_grant_expiration() {
        use std::time::Duration;

        let mut grant =
            CapabilityGrant::new("test-app".to_string(), CoreCapabilities::CAP_SCREEN_READ);

        // Not expired (no expiration set)
        assert!(!grant.is_expired());

        // Set expiration in the past
        grant.restrictions.expires_at = Some(SystemTime::now() - Duration::from_secs(3600));
        assert!(grant.is_expired());

        // Set expiration in the future
        grant.restrictions.expires_at = Some(SystemTime::now() + Duration::from_secs(3600));
        assert!(!grant.is_expired());
    }

    #[test]
    fn test_dynamic_capabilities() {
        let mut grant =
            CapabilityGrant::new("ai-agent".to_string(), CoreCapabilities::CAP_SCREEN_READ);

        // Add dynamic capability
        grant.add_dynamic("agent:abc123.vision.analyze".to_string());

        assert!(grant.has_dynamic("agent:abc123.vision.analyze"));
        assert!(!grant.has_dynamic("nonexistent"));
    }

    #[test]
    fn test_usage_tracking() {
        let mut grant =
            CapabilityGrant::new("test-app".to_string(), CoreCapabilities::CAP_KEYBOARD_WRITE);

        assert_eq!(grant.usage.use_count, 0);
        assert!(grant.usage.last_used.is_none());

        grant.record_use();
        assert_eq!(grant.usage.use_count, 1);
        assert!(grant.usage.last_used.is_some());

        grant.record_use();
        assert_eq!(grant.usage.use_count, 2);
    }

    #[test]
    fn test_restrictions_default() {
        let restrictions = CapabilityRestrictions::default();
        assert!(restrictions.max_events_per_second.is_none());
        assert!(restrictions.allowed_time_ranges.is_empty());
        assert_eq!(
            restrictions.require_confirmation_for,
            CoreCapabilities::empty()
        );
    }

    #[test]
    fn test_time_range() {
        use chrono::{Datelike, Duration, Local, Timelike};
        
        // Business hours: 9 AM - 5 PM, Monday-Friday
        let business_hours = TimeRange::business_hours();
        assert_eq!(business_hours.start, (9, 0));
        assert_eq!(business_hours.end, (17, 0));
        assert_eq!(business_hours.days.len(), 5);
        
        // Test specific time: Tuesday 10 AM (should be active)
        let tuesday_10am = Local::now()
            .with_hour(10)
            .and_then(|dt| dt.with_minute(0))
            .and_then(|dt| dt.with_second(0))
            .unwrap();
        
        // Find next Tuesday if not already Tuesday
        let tuesday_10am = if tuesday_10am.weekday().number_from_monday() != 2 {
            let days_until_tuesday = (2 + 7 - tuesday_10am.weekday().number_from_monday()) % 7;
            tuesday_10am + Duration::days(days_until_tuesday as i64)
        } else {
            tuesday_10am
        };
        
        assert!(business_hours.is_active_at(tuesday_10am.into()));
        
        // Test outside hours: Tuesday 8 AM (should not be active)
        let tuesday_8am = tuesday_10am - Duration::hours(2);
        assert!(!business_hours.is_active_at(tuesday_8am.into()));
        
        // Test outside days: Saturday 10 AM (should not be active)
        let days_until_saturday = (6 + 7 - tuesday_10am.weekday().number_from_monday()) % 7;
        let saturday_10am = tuesday_10am + Duration::days(days_until_saturday as i64);
        assert!(!business_hours.is_active_at(saturday_10am.into()));

        // Test always range
        let always = TimeRange::always();
        assert!(always.is_active_at(tuesday_10am.into()));
        assert!(always.is_active_at(saturday_10am.into()));
        assert!(always.is_active_at(tuesday_8am.into()));
        
        // Test night hours (crosses midnight)
        let night_hours = TimeRange::night_hours();
        let tuesday_11pm = tuesday_10am
            .with_hour(23)
            .and_then(|dt| dt.with_minute(0))
            .unwrap();
        let tuesday_3am = tuesday_10am
            .with_hour(3)
            .and_then(|dt| dt.with_minute(0))
            .unwrap();
        let tuesday_noon = tuesday_10am
            .with_hour(12)
            .and_then(|dt| dt.with_minute(0))
            .unwrap();
        
        assert!(night_hours.is_active_at(tuesday_11pm.into())); // 11 PM - active
        assert!(night_hours.is_active_at(tuesday_3am.into()));  // 3 AM - active
        assert!(!night_hours.is_active_at(tuesday_noon.into())); // Noon - not active
    }

    #[test]
    fn test_grant_uniqueness() {
        let grant1 = CapabilityGrant::new("app".to_string(), CoreCapabilities::empty());
        let grant2 = CapabilityGrant::new("app".to_string(), CoreCapabilities::empty());

        // Each grant should have a unique ID
        assert_ne!(grant1.id, grant2.id);
    }
}
