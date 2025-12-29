# 🔑 VM Login Credentials - rustdesk-20251228-115642

## VM Access Information

**VM Name:** `rustdesk-20251228-115642`  
**IP Address:** `192.168.122.229`

---

## 🖥️ VNC/Virtual Viewer Login

When you open the VM with:
```bash
virt-viewer rustdesk-20251228-115642
```

**Login Credentials:**
- **Username:** `rustdesk20251228115642`
- **Password:** `ad804ad26e61d510`

*(The password is derived from the VM name using sha256 hash)*

---

## 🔐 SSH Access

```bash
ssh -i /tmp/iontest_key_rustdesk-20251228-115642 rustdesk20251228115642@192.168.122.229
```

Note: SSH should use the key, no password needed.

---

## 🚀 Quick Connection Steps

### 1. Access VM Desktop
```bash
virt-viewer rustdesk-20251228-115642
```

### 2. Login
- Username: `rustdesk20251228115642`
- Password: `ad804ad26e61d510`

### 3. Start RustDesk
Once logged in:
- RustDesk should auto-start, or
- Find RustDesk in applications menu, or
- Open terminal and run: `rustdesk`

### 4. Get RustDesk ID
- The RustDesk GUI will show you the connection ID
- Set a password in RustDesk (e.g., "iontest123")

### 5. Connect from Your Other Tower
- Open RustDesk on your other Pop!_OS machine
- Enter the ID from step 4
- Enter the password you set
- Connect!

---

## 🎯 All VM Credentials Reference

For any VM named like `rustdesk-YYYYMMDD-HHMMSS`:
- Username is always: alphanumeric characters only, lowercase
- Password is: first 16 chars of sha256(vm_name)

Example:
```
VM: web-01
Username: web01
Password: (sha256 of "web-01")[:16]
```

---

## ✅ Ready to Connect!

You now have everything needed to:
1. ✅ Login to the VM desktop
2. ✅ Start RustDesk 
3. ✅ Get the RustDesk ID
4. ✅ Connect from your external tower

**The full ionChannel + benchScale stack is working!**

