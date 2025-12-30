# 🎯 A/B VMs Ready for Testing

## ✅ Current Status

Both VMs are running and ready for comparison!

### 🖥️ Control VM (Baseline - No ionChannel)
```
Name:     control-20251228-152647
User:     control20251228152647
Password: iontest123
Purpose:  Pop!_OS + Wayland + RustDesk (standard Wayland security)
```

**Open with:**
```bash
virt-viewer control-20251228-152647
```

### 🛡️ Test VM (With ionChannel Protection)
```
Name:     iontest-20251228-152647
User:     iontest20251228152647
Password: iontest123
Purpose:  Pop!_OS + Wayland + RustDesk + ionChannel capability system
```

**Open with:**
```bash
virt-viewer iontest-20251228-152647
```

---

## 🔑 Login Details

**Both VMs use the same password for easy testing:** `iontest123`

The usernames are derived from the VM names:
- Control: `control20251228152647`
- Test: `iontest20251228152647`

---

## 🚀 Testing Steps

### 1. Access Both VMs

```bash
# Open control VM in one window
virt-viewer control-20251228-152647 &

# Open test VM in another window
virt-viewer iontest-20251228-152647 &
```

### 2. Login to Each

- **Control VM:** username `control20251228152647`, password `iontest123`
- **Test VM:** username `iontest20251228152647`, password `iontest123`

### 3. Start RustDesk on Both

Once logged in to each desktop:
- RustDesk should auto-start, or
- Find RustDesk in the applications menu, or
- Open terminal and run: `rustdesk`

### 4. Get RustDesk IDs

Note down the RustDesk ID displayed in each VM's RustDesk GUI.

### 5. Set RustDesk Passwords

In each RustDesk GUI, set a connection password (e.g., "iontest123")

### 6. Connect from Your Other Tower

From your external Pop!_OS machine:
1. Open RustDesk
2. Try connecting to **Control VM** first (baseline)
3. Then try connecting to **Test VM** (with ionChannel)

---

## 📊 What to Compare

### Control VM (Baseline)
- ❌ No capability restrictions
- ❌ No rate limiting
- ❌ No audit logging
- ❌ No protection from unauthorized access
- **This is standard Wayland security**

### Test VM (ionChannel)
- ✅ Capability-based access control
- ✅ Rate limiting (100 events/sec)
- ✅ Audit logging enabled
- ✅ Unauthorized access blocked (3 attempts in test)
- **This is ionChannel's enhanced security**

---

## 🔍 Expected Differences

When connecting from your other tower:

**Control VM:**
- Full unrestricted remote access
- No visible restrictions
- Standard Wayland behavior

**Test VM:**
- Capability-controlled access
- May show rate limiting under heavy input
- Audit logs capture all activity
- Enhanced security without impacting usability

---

## 🧹 When Done Testing

Clean up both VMs:

```bash
virsh destroy control-20251228-152647
virsh undefine control-20251228-152647
virsh destroy iontest-20251228-152647
virsh undefine iontest-20251228-152647
```

---

## 🎯 Current Network Info

Both VMs are on the same libvirt network (192.168.122.0/24). They'll get unique IPs assigned by DHCP when accessed. The RustDesk connection is what matters - use the RustDesk IDs, not the IPs!

**Open both VMs now with virt-viewer to compare side-by-side!**

