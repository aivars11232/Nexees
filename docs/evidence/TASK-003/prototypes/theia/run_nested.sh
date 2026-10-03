#!/bin/bash
# Runs a command against a nested KWin that renders to a virtual framebuffer,
# on a private D-Bus session and with an isolated HOME, so a desktop app can
# start without touching the owner's screen, session bus, keyring or settings:
#
#   run_nested.sh <log-file> <command>...
#
# Chromium's headless Ozone platform was tried first; with Electron 42.8.1
# its renderer never started (see the TASK-003 evidence).
set -u
log=$1
shift
T=/mnt/F/Nexees-toolchains
exec env -u WAYLAND_DISPLAY -u DISPLAY -u ELECTRON_RUN_AS_NODE -u DBUS_SESSION_BUS_ADDRESS \
    HOME=$T/home XDG_CONFIG_HOME=$T/home/.config XDG_DATA_HOME=$T/home/.local/share \
    XDG_CACHE_HOME=$T/home/.cache XDG_STATE_HOME=$T/home/.local/state \
    dbus-run-session -- bash -c '
        log=$1
        shift
        socket=wayland-nexees-check-$$
        kwin_wayland --virtual --socket "$socket" --width 1280 --height 800 > "$log" 2>&1 &
        kwin=$!
        for _ in $(seq 1 100); do [ -S "$XDG_RUNTIME_DIR/$socket" ] && break; timeout 0.1 tail -f /dev/null; done
        WAYLAND_DISPLAY=$socket XDG_SESSION_TYPE=wayland NEXEES_PRIVATE_SESSION=1 "$@"
        status=$?
        kill "$kwin"
        wait "$kwin" 2>/dev/null
        # D-Bus activated services (portals, the secret service) outlive the
        # bus; stop every process that belongs to this private session.
        for environ in /proc/[0-9]*/environ; do
            pid=${environ#/proc/}
            pid=${pid%/environ}
            [ "$pid" = $$ ] && continue
            if tr "\0" "\n" < "$environ" 2>/dev/null | grep -qxF "DBUS_SESSION_BUS_ADDRESS=$DBUS_SESSION_BUS_ADDRESS"; then
                kill "$pid" 2>/dev/null
            fi
        done
        exit $status
    ' nested "$log" "$@"
