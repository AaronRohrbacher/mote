#!/bin/bash
# Wrapper script to connect to VNC with password from environment variable

VNC_PASSWORD="${VNC_PASSWORD:-KonstaKANG}"
VNC_HOST="10.1.1.79"
HOME_BUTTON_SCRIPT="/home/m/home-button.sh"
PASSWD_FILE="/tmp/mote_vnc_passwd_$$"

# Create VNC password file - try vncpasswd first (most reliable)
if command -v vncpasswd >/dev/null 2>&1; then
    # vncpasswd -f reads password from stdin
    printf "%s\n%s\n" "$VNC_PASSWORD" "$VNC_PASSWORD" | vncpasswd "$PASSWD_FILE" 2>/dev/null
    # If that doesn't work, try -f flag if supported
    if [ ! -f "$PASSWD_FILE" ] || [ ! -s "$PASSWD_FILE" ]; then
        echo "$VNC_PASSWORD" | vncpasswd -f > "$PASSWD_FILE" 2>/dev/null
    fi
else
    # No vncpasswd available - need to install tigervnc-standalone-server
    echo "Error: vncpasswd not found. Install with: sudo apt install tigervnc-standalone-server" >&2
    exit 1
fi

# Verify file was created
if [ ! -f "$PASSWD_FILE" ] || [ ! -s "$PASSWD_FILE" ]; then
    echo "Error: Failed to create VNC password file" >&2
    exit 1
fi

chmod 600 "$PASSWD_FILE"

# Connect with password file
ssvncviewer -quality 9 -compresslevel 0 -fullscreen -scale '800x480' -passwd "$PASSWD_FILE" "$VNC_HOST" &

# Clean up password file after a delay
(sleep 5 && rm -f "$PASSWD_FILE") &

# Start home button after a delay
sleep 1
$HOME_BUTTON_SCRIPT &
