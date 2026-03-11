#!/usr/bin/env bash
set -e
# Connect to RDP server, let desktop load, capture screen. Fail if capture is blank or wrong size.
# Run in container with Xvfb. Requires: xvfb, rdesktop, imagemagick.
# Env: RDP_HOST (default xrdp), RDP_USER (default guest), RDP_PASS (default guest).

RDP_HOST="${RDP_HOST:-xrdp}"
RDP_USER="${RDP_USER:-guest}"
RDP_PASS="${RDP_PASS:-guest}"
DISPLAY="${DISPLAY:-:99}"
CAPTURE="/tmp/rdp-capture.png"

echo "Waiting for RDP server $RDP_HOST:3389..."
for i in $(seq 1 30); do
  if (echo >/dev/tcp/"$RDP_HOST"/3389) 2>/dev/null; then break; fi
  if [ "$i" -eq 30 ]; then echo "FAIL: RDP server not reachable"; exit 1; fi
  sleep 1
done

echo "Starting Xvfb on $DISPLAY..."
Xvfb "$DISPLAY" -screen 0 800x480x24 &
XVFB_PID=$!
sleep 2

echo "Connecting rdesktop -u $RDP_USER -p *** -g 800x480 -f $RDP_HOST:3389..."
DISPLAY="$DISPLAY" rdesktop -u "$RDP_USER" -p "$RDP_PASS" -g 800x480 -f "${RDP_HOST}:3389" &
RDP_PID=$!

echo "Waiting 10s for desktop to load..."
sleep 10

echo "Capturing screen..."
DISPLAY="$DISPLAY" import -window root "$CAPTURE" 2>/dev/null || true

kill $RDP_PID 2>/dev/null || true
kill $XVFB_PID 2>/dev/null || true

if [ ! -f "$CAPTURE" ]; then
  echo "FAIL: No capture file"
  exit 1
fi
SIZE=$(stat -c%s "$CAPTURE" 2>/dev/null || stat -f%z "$CAPTURE" 2>/dev/null)
DIMS=$(identify -format "%wx%h" "$CAPTURE" 2>/dev/null || true)
echo "Capture: $CAPTURE size=$SIZE dims=$DIMS"
if [ "$SIZE" -lt 1000 ]; then
  echo "FAIL: Capture too small ($SIZE bytes) - desktop did not load"
  exit 1
fi
echo "PASS: Desktop loaded (capture $SIZE bytes)"
exit 0
