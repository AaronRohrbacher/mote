#!/usr/bin/env python3
"""
Create a VNC password file in the format expected by VNC viewers.
This mimics what vncpasswd does without requiring the vncpasswd tool.
"""
import sys
import getpass
import struct
from Crypto.Cipher import DES

def create_vnc_passwd(password, output_file):
    """Create a VNC password file from a plain text password."""
    # VNC password encryption uses DES with the password as both key and data
    # Pad password to 8 bytes for DES key
    key = password[:8].ljust(8).encode('latin-1')
    # Pad password to 8 bytes for DES data
    data = password[:8].ljust(8).encode('latin-1')
    
    # Create DES cipher in ECB mode
    cipher = DES.new(key, DES.MODE_ECB)
    encrypted = cipher.encrypt(data)
    
    # Write to file
    with open(output_file, 'wb') as f:
        f.write(encrypted)
    
    # Set permissions to 600 (read/write for owner only)
    import os
    os.chmod(output_file, 0o600)

if __name__ == '__main__':
    if len(sys.argv) < 2:
        password = getpass.getpass("VNC Password: ")
        output_file = sys.argv[1] if len(sys.argv) > 1 else '/home/m/mote/vncpasswd'
    else:
        password = sys.argv[1]
        output_file = sys.argv[2] if len(sys.argv) > 2 else '/home/m/mote/vncpasswd'
    
    create_vnc_passwd(password, output_file)
    print(f"Password file created: {output_file}")

