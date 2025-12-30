#!/bin/bash
# Helper script to get RustDesk ID from the test VM

set -e

VM_NAME="rustdesk-wayland-test-20251229-134919"
VM_IP="192.168.122.71"
SSH_KEY="/tmp/iontest_key_${VM_NAME}"

echo "╔══════════════════════════════════════════════════════════════════════════╗"
echo "║                                                                          ║"
echo "║                  🔑 GET RUSTDESK ID                                      ║"
echo "║                                                                          ║"
echo "╚══════════════════════════════════════════════════════════════════════════╝"
echo ""

echo "VM: $VM_NAME"
echo "IP: $VM_IP"
echo ""

# Method 1: Check if config file exists
echo "Method 1: Checking config file..."
ID=$(ssh -i "$SSH_KEY" -o StrictHostKeyChecking=no ubuntu@$VM_IP \
    'cat ~/.config/rustdesk/RustDesk.toml 2>/dev/null | grep "^id" | cut -d= -f2 | tr -d " \"" || true')

if [ -n "$ID" ]; then
    echo "✅ RustDesk ID found: $ID"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📋 USE THIS ID IN YOUR REMOTE RUSTDESK CLIENT:"
    echo "   ID: $ID"
    echo "   Password: ubuntu"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 0
fi

# Method 2: RustDesk needs GUI session
echo "⚠️  RustDesk ID not found in config"
echo ""
echo "RustDesk needs to run in a GUI session to generate the ID."
echo ""
echo "═══════════════════════════════════════════════════════════════════════════"
echo "OPTION 1: Use virt-viewer (Recommended)"
echo "═══════════════════════════════════════════════════════════════════════════"
echo ""
echo "1. Install virt-viewer (if not installed):"
echo "   sudo apt install virt-viewer"
echo ""
echo "2. Connect to VM console:"
echo "   virt-viewer $VM_NAME"
echo ""
echo "3. Log in at the desktop:"
echo "   Username: ubuntu"
echo "   Password: ubuntu"
echo ""
echo "4. Once logged in, RustDesk should auto-start"
echo "   Look for RustDesk icon in system tray"
echo "   Click it to see the ID"
echo ""
echo "5. OR run this script again after logging in"
echo ""
echo "═══════════════════════════════════════════════════════════════════════════"
echo "OPTION 2: Use virsh console (Text-based)"
echo "═══════════════════════════════════════════════════════════════════════════"
echo ""
echo "   sudo virsh console $VM_NAME"
echo "   (Then log in and run: rustdesk --get-id)"
echo ""
echo "═══════════════════════════════════════════════════════════════════════════"

