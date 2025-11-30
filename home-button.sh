#!/bin/bash
# Home button overlay for Mote VNC viewer

SCREEN_WIDTH=$(swaymsg -t get_outputs | jq -r '.[0].current_mode.width' 2>/dev/null || echo "1920")
BUTTON_SIZE=80
BUTTON_X=$((SCREEN_WIDTH - BUTTON_SIZE - 20))
BUTTON_Y=20

return_to_desktop() {
    pkill -f "ssvncviewer.*10.1.1.79"
    swaymsg workspace 1 2>/dev/null
    exit 0
}

# Keep showing the button while VNC is running
while pgrep -f "ssvncviewer.*10.1.1.79" > /dev/null; do
    # Use yad with simple button - remove conflicting options
    yad --button="🏠:0" \
        --undecorated \
        --skip-taskbar \
        --on-top \
        --geometry=${BUTTON_SIZE}x${BUTTON_SIZE}+${BUTTON_X}+${BUTTON_Y} \
        --text="<span font='Sans 48'>🏠</span>" \
        --timeout=0 \
        --close-on-unfocus=false
    
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 0 ]; then
        return_to_desktop
    fi
    
    # If window closed, wait before recreating
    sleep 0.5
done
