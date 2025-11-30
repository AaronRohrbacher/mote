#!/bin/bash
# Close button overlay for Mote VNC viewer

SCREEN_WIDTH=$(swaymsg -t get_outputs | jq -r '.[0].current_mode.width' 2>/dev/null || echo "1920")
BUTTON_SIZE=80
BUTTON_X=$((SCREEN_WIDTH - BUTTON_SIZE - 20))
BUTTON_Y=20

close_vnc() {
    pkill -f "ssvncviewer.*10.1.1.79"
    exit 0
}

while pgrep -f "ssvncviewer.*10.1.1.79" > /dev/null; do
    yad --button="✕:0" --undecorated --skip-taskbar --on-top \
        --geometry=${BUTTON_SIZE}x${BUTTON_SIZE}+${BUTTON_X}+${BUTTON_Y} \
        --text="<span font='Sans 48'>✕</span>" --timeout=0 \
        --close-on-unfocus=false 2>/dev/null
    
    if [ $? -eq 0 ]; then
        close_vnc
    fi
    sleep 0.5
done

