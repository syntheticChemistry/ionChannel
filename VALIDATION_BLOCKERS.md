# Validation Blockers - December 28, 2025

## Current Situation

**Status:** E2E validation is blocked by cloud-init SSH key authentication failure

**Progress:**
- ✅ benchScale integration complete
- ✅ VM provisioning works (VMs boot, get IPs)
- ✅ RustDesk .deb available in agentReagents
- ✅ Cloud-init user-data generation working
- ✅ Cloud-init ISO created and attached
- ❌ SSH key authentication failing on all VMs

## Root Cause

Cloud-init is either:
1. Not recognizing the NoCloud datasource (cidata ISO)
2. Not processing the user-data correctly
3. Missing kernel parameters for datasource detection

**Evidence:**
- All VMs (including "working" ones) fail SSH public key auth
- Cloud-init user-data file is correctly formatted with SSH keys
- ISO is attached to VM (`virsh domblklist` confirms)
- SSH server is running (get "Permission denied" not "Connection refused")
- Both `iontest` and default `ubuntu` user fail authentication

## User's Correct Insight

> "i thought we were using agentReagents/ for images and other systems rather than fresh install every time?"

**This is absolutely correct.** The primal philosophy is to use pre-built, tested artifacts from `agentReagents/`, not provision from scratch each time.

## Solutions (In Order of Alignment with Primal Philosophy)

### Solution 1: Pre-Built RustDesk Image (RECOMMENDED)
**Alignment:** 100% - Pure primal philosophy

Create a template image with RustDesk pre-installed:
```bash
# In agentReagents/images/
mkdir -p templates/
# Create rustdesk-ubuntu-22.04-template.qcow2 with:
# - Ubuntu 22.04
# - RustDesk pre-installed
# - SSH properly configured
# - Cloud-init enabled for network/user customization only
```

**Benefits:**
- No internet dependency
- Faster provisioning (seconds, not minutes)
- Consistent, tested environment
- True primal approach (pre-built reagent)

**Implementation:**
1. Manually create one perfect VM
2. Install RustDesk from `agentReagents/debs/`
3. Configure for cloud-init (cloud-init clean)
4. Sysprep and save as template
5. Store in `agentReagents/images/templates/`

### Solution 2: Fix Cloud-Init NoCloud Detection
**Alignment:** 70% - Still uses agentReagents .deb but requires cloud-init complexity

Add kernel parameters or ensure proper datasource configuration:
```bash
# In virt-install, add:
--cloud-init user-data=/tmp/user-data,meta-data=/tmp/meta-data
# Or kernel param:
--extra-args="ds=nocloud;seedfrom=/dev/sr0/"
```

**Complexity:** High - cloud-init debugging is notoriously difficult

### Solution 3: Password-Based Initial Auth
**Alignment:** 50% - Workaround, not elegant

Set a known password in cloud-init, SSH in with password, then install key:
```yaml
users:
  - name: iontest
    passwd: $6$rounds=4096$... # hashed password
    ssh_authorized_keys: [...]
```

**Issue:** Requires `sshpass` or manual interaction

## Recommendation

**Implement Solution 1** - Create a RustDesk template image in `agentReagents/`.

This is:
- **Faster** (30 seconds vs 5 minutes)
- **More reliable** (pre-tested)
- **Primal** (uses reagents, not fresh synthesis)
- **Sovereignty-respecting** (no external deps)

## Next Steps

1. Create `agentReagents/images/templates/` directory
2. Build one perfect RustDesk VM manually
3. Test it thoroughly
4. Save as template qcow2
5. Update benchScale to use template images
6. Update ionChannel validation to reference template

## Current Validation Status

**Phase:** Blocked at SSH authentication  
**VMs Created:** 7 test VMs (all with same cloud-init issue)  
**Time Spent:** ~3 hours on cloud-init debugging  
**Better Approach:** 30 min to create template, save 3 hours  

## Files Updated for agentReagents Integration

- ✅ `ion-validation/src/rustdesk.rs` - Now uses local .deb via SCP
- ✅ `autonomous-rustdesk-benchscale.rs` - References agentReagents path
- ⏳ Need: Template image creation workflow
- ⏳ Need: benchScale template support

---

**Conclusion:** The user's intuition was correct. We should use pre-built images from `agentReagents/`, not provision from scratch. This aligns with primal philosophy and will be faster and more reliable.

