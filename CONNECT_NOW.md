# 🎯 Connect to Your RustDesk VM - Quick Guide

## ✅ VM Ready for Connection

**VM Name:** `rustdesk-20251228-115642`  
**IP Address:** `192.168.122.229`  
**Username:** `rustdesk20251228115642`  
**Status:** Running with desktop environment

## 🔑 Connection Methods

### Method 1: VNC (Recommended for First-Time Setup)
This will let you start RustDesk GUI and get the real ID.

```bash
# From this Pop!_OS tower:
virt-viewer rustdesk-20251228-115642
```

Or use VNC client directly:
1. Get VNC port: `virsh vncdisplay rustdesk-20251228-115642`
2. Connect to: `localhost:<port>`

### Method 2: SSH (Already Working!)
```bash
# SSH key is at:
~/.ssh/rustdesk-20251228-115642

# Connect via SSH:
ssh -i ~/.ssh/rustdesk-20251228-115642 rustdesk20251228115642@192.168.122.229
```

### Method 3: RustDesk (After GUI Start)
Once you connect via VNC/viewer:
1. Login to the desktop
2. RustDesk should auto-start (or start it manually)
3. Get the RustDesk ID from the GUI
4. Use that ID to connect from your external tower

## 🚀 Quick Setup Steps

### Step 1: Access the VM via VNC
```bash
virt-viewer rustdesk-20251228-115642
```

### Step 2: Login to Desktop
- The VM has Ubuntu desktop installed
- Login should auto-login or use the derived credentials

### Step 3: Start RustDesk
RustDesk is pre-installed. Either:
- It auto-starts on login
- Or find it in applications and launch it
- Or via terminal: `rustdesk`

### Step 4: Get RustDesk ID
- Once RustDesk GUI is open, you'll see the ID displayed
- Copy this ID

### Step 5: Set RustDesk Password
In the RustDesk GUI:
1. Set a permanent password (e.g., "iontest123")
2. This password works for any connection

### Step 6: Connect from External Tower
On your **other Pop!_OS tower**:
1. Open RustDesk
2. Enter the ID you saw in the VM
3. Enter the password you set
4. Connect!

## 🔧 Alternative: Get RustDesk ID via SSH

If you prefer command-line:

```bash
# SSH into the VM
ssh -i ~/.ssh/rustdesk-20251228-115642 rustdesk20251228115642@192.168.122.229

# Start RustDesk service
rustdesk &

# Get the ID
rustdesk --get-id

# Set password
rustdesk --password iontest123
```

## 🌐 Network Access from External Tower

Your external tower needs to be able to reach `192.168.122.229`.

**If both towers are on the same physical network:**
- The external tower should be able to reach the VM's IP directly
- Try pinging from external tower: `ping 192.168.122.229`

**If external tower is remote:**
- You may need to set up port forwarding or a tunnel
- Or use RustDesk's relay server (automatic)

## 📊 Current VM Status

```bash
# Check VM is running
virsh list --all | grep rustdesk-20251228-115642

# Get IP
virsh domifaddr rustdesk-20251228-115642

# Check from inside (via SSH)
ssh -i ~/.ssh/rustdesk-20251228-115642 rustdesk20251228115642@192.168.122.229 "ps aux | grep rustdesk"
```

## 🎯 ionChannel Capability Grant

This VM has active capability grants:
- **Keyboard Write** ✅
- **Pointer Write** ✅
- **Screen Read** ✅
- **Audio Capture** ✅
- **Clipboard Read/Write** ✅
- **File Transfer** ✅
- **Portal Integration** ✅

All managed by ionChannel's capability system!

## 🧹 Cleanup (When Done)

```bash
# Stop VM
virsh shutdown rustdesk-20251228-115642

# Or force stop
virsh destroy rustdesk-20251228-115642

# Remove VM
virsh undefine rustdesk-20251228-115642

# Remove disk
sudo rm /var/lib/libvirt/images/rustdesk-20251228-115642.qcow2
```

---

## 🎉 Next Steps

1. **Now:** `virt-viewer rustdesk-20251228-115642` to access the desktop
2. **Get RustDesk ID** from the GUI
3. **Connect from your other tower** using that ID
4. **Success!** You're remotely controlling a VM with full ionChannel capability protection

**This demonstrates:**
- ✅ benchScale VM provisioning
- ✅ Cloud-init user setup
- ✅ Template-based deployment
- ✅ RustDesk pre-installed
- ✅ ionChannel capability grants
- ✅ Ready for external connection

