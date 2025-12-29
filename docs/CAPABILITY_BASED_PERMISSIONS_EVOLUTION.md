# ionChannel Evolution: Capability-Based Permissions

**Inspired by:** Reddit critique of Wayland security model  
**Date:** December 28, 2025

---

## Problem Statement

Wayland's current portal system is criticized for:
- Binary all-or-nothing permissions
- No persistent capability grants
- Treating users as threats to themselves
- Forcing repeated consent dialogs

**Quote from Reddit:**
> "Wayland could have been designed like a microkernel approach. Minimal core protocol, well defined extension points, capability system where compositors grant specific permissions to specific apps."

---

## Current ionChannel Approach

### Session-Level Permissions
```rust
// Current: Binary remote desktop access
struct RemoteDesktopSession {
    can_inject_input: bool,  // All or nothing
    can_capture_screen: bool, // All or nothing
}
```

**Issues:**
- No granularity (can't say "keyboard only")
- No persistence (must re-approve every session)
- No app-specific policies
- No user-configurable trust

---

## Proposed Evolution: Capability Tokens

### Linux Kernel Style Capabilities

```rust
use bitflags::bitflags;

bitflags! {
    /// Fine-grained remote desktop capabilities
    /// Inspired by Linux CAP_* system
    pub struct RemoteDesktopCapabilities: u64 {
        // Input capabilities
        const CAP_INJECT_KEYBOARD    = 1 << 0;
        const CAP_INJECT_MOUSE       = 1 << 1;
        const CAP_INJECT_TOUCH       = 1 << 2;
        
        // Screen capabilities  
        const CAP_CAPTURE_SCREEN     = 1 << 3;
        const CAP_CAPTURE_WINDOW     = 1 << 4;
        const CAP_CAPTURE_REGION     = 1 << 5;
        
        // Window management
        const CAP_QUERY_WINDOWS      = 1 << 6;
        const CAP_FOCUS_WINDOWS      = 1 << 7;
        const CAP_POSITION_WINDOWS   = 1 << 8;
        
        // Clipboard
        const CAP_READ_CLIPBOARD     = 1 << 9;
        const CAP_WRITE_CLIPBOARD    = 1 << 10;
        
        // Audio
        const CAP_CAPTURE_AUDIO      = 1 << 11;
        const CAP_INJECT_AUDIO       = 1 << 12;
        
        // Meta capabilities
        const CAP_PERSIST_GRANT      = 1 << 30; // Remember this choice
        const CAP_BACKGROUND_ACCESS  = 1 << 31; // Work without focus
    }
}

/// Capability grant with context
#[derive(Debug, Clone)]
pub struct CapabilityGrant {
    /// What capabilities are granted
    pub capabilities: RemoteDesktopCapabilities,
    
    /// For which application (by app_id)
    pub app_id: String,
    
    /// For how long (None = this session only)
    pub expires: Option<std::time::SystemTime>,
    
    /// User-provided reason (for audit)
    pub reason: Option<String>,
    
    /// Restrictions
    pub restrictions: CapabilityRestrictions,
}

#[derive(Debug, Clone)]
pub struct CapabilityRestrictions {
    /// Rate limiting
    pub max_events_per_second: Option<u32>,
    
    /// Time windows (e.g., "only during work hours")
    pub allowed_times: Vec<TimeRange>,
    
    /// IP restrictions (e.g., "only from local network")
    pub allowed_sources: Vec<IpRange>,
    
    /// Screen region restrictions
    pub allowed_regions: Vec<ScreenRegion>,
}
```

---

## User Experience Flow

### Initial Request (Detailed Consent)
```
┌─────────────────────────────────────────────────────────┐
│  RustDesk wants to access your desktop                  │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  Application: rustdesk (1.2.3)                          │
│  Requesting:                                             │
│    ☑ View your screen                                   │
│    ☑ Control keyboard and mouse                         │
│    ☐ Access clipboard                                   │
│    ☐ Reposition windows                                 │
│                                                          │
│  Advanced Options ▼                                      │
│    ☐ Remember this choice (until logout)                │
│    ☐ Allow background access (when window not focused)  │
│    ☐ Apply rate limiting (100 events/sec)               │
│                                                          │
│  Why: "Remote support for family member"                │
│                                                          │
│         [Deny]    [Allow Selected]    [Allow All]       │
└─────────────────────────────────────────────────────────┘
```

### Power User Mode (Config File)
```toml
# ~/.config/ionChannel/capabilities.toml

[[grant]]
app_id = "rustdesk"
capabilities = [
    "CAP_INJECT_KEYBOARD",
    "CAP_INJECT_MOUSE",
    "CAP_CAPTURE_SCREEN"
]
expires = "2025-12-31"
max_events_per_second = 100
reason = "Trusted remote desktop tool"

[[grant]]
app_id = "obs-studio"
capabilities = ["CAP_CAPTURE_SCREEN", "CAP_CAPTURE_AUDIO"]
restrictions.allowed_regions = [
    { x = 0, y = 0, width = 1920, height = 1080 } # Primary monitor only
]

[[deny]]
app_id = "untrusted-app"
reason = "Known malware"
```

---

## Implementation Architecture

### Capability Store
```rust
/// Persistent capability grants
pub struct CapabilityStore {
    /// Active grants
    grants: HashMap<String, Vec<CapabilityGrant>>,
    
    /// Audit log
    log: AuditLog,
    
    /// Policy engine
    policy: PolicyEngine,
}

impl CapabilityStore {
    /// Check if app has capability
    pub fn check(&self, app_id: &str, cap: RemoteDesktopCapabilities) -> Result<bool> {
        let grants = self.grants.get(app_id)?;
        
        for grant in grants {
            // Check expiration
            if let Some(expires) = grant.expires {
                if SystemTime::now() > expires {
                    continue;
                }
            }
            
            // Check if capability is granted
            if grant.capabilities.contains(cap) {
                // Check restrictions
                if self.policy.check_restrictions(&grant.restrictions)? {
                    self.log.record_access(app_id, cap);
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    /// Prompt user for capability
    pub async fn request(&mut self, app_id: &str, caps: RemoteDesktopCapabilities) 
        -> Result<CapabilityGrant> 
    {
        // Show consent dialog with granular options
        let grant = self.show_consent_dialog(app_id, caps).await?;
        
        // Store grant if user selected "remember"
        if grant.capabilities.contains(RemoteDesktopCapabilities::CAP_PERSIST_GRANT) {
            self.grants.entry(app_id.to_string())
                .or_default()
                .push(grant.clone());
            self.save()?;
        }
        
        Ok(grant)
    }
}
```

### Audit Trail
```rust
/// Every capability use is logged
pub struct AuditLog {
    entries: Vec<AuditEntry>,
}

#[derive(Debug, Clone)]
pub struct AuditEntry {
    timestamp: SystemTime,
    app_id: String,
    capability: RemoteDesktopCapabilities,
    source_ip: Option<IpAddr>,
    denied: bool,
    reason: Option<String>,
}

impl AuditLog {
    /// User can review what happened
    pub fn query(&self, filter: AuditFilter) -> Vec<AuditEntry> {
        // "Show me all times rustdesk accessed my keyboard"
        // "Show me denied access attempts"
        // "Show me access from outside my home network"
    }
}
```

---

## Benefits Over Current Wayland Portals

### 1. **User Freedom**
- Users CHOOSE what to allow
- Can grant persistent access (convenience)
- Can revoke at any time (control)

### 2. **Granularity**
- Not all-or-nothing
- Mix and match capabilities
- Per-app policies

### 3. **Transparency**
- Audit log shows what actually happened
- Users can review and understand access
- Not a black box

### 4. **Security Through Understanding**
- Users make informed decisions
- Rate limiting prevents abuse
- Restrictions allow "yes, but carefully"

### 5. **Power User Friendly**
- Config file for automation
- No repeated dialogs for trusted apps
- Programmatic access to policy

---

## Comparison to Criticized Wayland Model

| Aspect | Current Wayland | Proposed ionChannel |
|--------|----------------|---------------------|
| Permission Model | Binary (yes/no) | Granular (capabilities) |
| Persistence | Session-only | User choice |
| Configuration | GUI only | GUI + config files |
| Audit | None | Full audit log |
| Trust Model | "Protect from user" | "Empower user" |
| Flexibility | Rigid | Configurable |
| Power User | Hostile | Friendly |

---

## Addressing Reddit Critiques

### "Wayland treats users like threats"
**Our Response:** Users are in control. They decide what to allow.

### "No capability system like CAP_NET_ADMIN"
**Our Response:** We implement exactly that - CAP_INJECT_KEYBOARD, etc.

### "Portal or nothing, even if compositor doesn't implement"
**Our Response:** Capability discovery - we work with what exists.

### "Automation tools broken"
**Our Response:** CAP_INJECT_* capabilities enable automation with consent.

### "Repeated consent dialogs are annoying"
**Our Response:** CAP_PERSIST_GRANT - remember trusted apps.

---

## Migration Path

### Phase 1: Internal Architecture (1-2 weeks)
- Implement capability bitflags
- Add CapabilityStore
- Create audit logging

### Phase 2: Enhanced Consent UI (2-3 weeks)
- Granular permission dialogs
- "Remember this choice" option
- User-friendly capability names

### Phase 3: Power User Features (2-3 weeks)
- Config file support
- Audit log viewer
- Policy management CLI

### Phase 4: Documentation & Advocacy (1-2 weeks)
- Document capability model
- Write blog post about approach
- Engage with Wayland/COSMIC community

---

## Upstream Contribution Strategy

### Short-term: Enhance xdg-desktop-portal Spec
Propose additions to the RemoteDesktop portal:
- Granular device selection (keyboard only, mouse only, etc.)
- Persistent grants with expiration
- Capability flags

### Long-term: Advocate for Philosophy Change
- Show that security AND freedom are possible
- Demonstrate capability model in production
- Build community support

---

## Risk Assessment

### Potential Criticisms
1. **"This makes security worse"**
   - Response: Informed consent is better than silent bypass
   - Audit log provides accountability
   - Rate limiting prevents abuse

2. **"Users will just click Allow All"**
   - Response: That's their choice on their computer
   - Default is still secure (no persistent grants)
   - Power users who want control get it

3. **"Complexity for users"**
   - Response: Defaults are simple
   - Advanced options hidden
   - Config file optional

### Security Considerations
- Capability grants are per-user (not system-wide)
- Expire by default (must be explicitly persistent)
- Audit log cannot be disabled
- Policy engine can enforce organizational rules

---

## Conclusion

This evolution would make ionChannel a **demonstration that Wayland security and user freedom are NOT mutually exclusive**.

We can:
- ✅ Provide granular, capability-based permissions
- ✅ Respect user choice and control
- ✅ Enable power user workflows
- ✅ Maintain security through transparency
- ✅ Offer escape hatches without compromising defaults

**This is exactly what the Reddit poster wanted: A capability system like Linux kernel caps, not binary restrictions.**

---

**Next Actions:**
1. Discuss with team
2. Prototype capability bitflags
3. Design consent UI mockups
4. Write blog post about our philosophy
5. Engage COSMIC/Wayland community

**Status:** Proposal - ready for implementation


