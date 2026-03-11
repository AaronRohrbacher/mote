#!/usr/bin/env bash
set -e

export SSHPASS="k"
HOST="m@mote.local"
SSHOPTS="-o StrictHostKeyChecking=no -o ConnectTimeout=10"

run_ssh()  { sshpass -e ssh  $SSHOPTS "$HOST" "$@"; }
run_scp()  { sshpass -e scp  $SSHOPTS "$@" "$HOST:~"; }

echo "1. Kill old process"
run_ssh "pkill desktop-icons || true"

echo "2. Build"
make pi4

echo "3. Deploy binary"
run_scp desktop-icons

echo "4. Reboot"
run_ssh "sudo reboot" || true

echo "Done. Pi is rebooting."
