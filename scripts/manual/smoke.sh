#! /usr/bin/env nix-shell
#! nix-shell -i bash -p wtype jq grim
# Spec: WIN-1, WIN-2, WIN-3, WIN-4, CONF-3.
# Run from the repo root after `nix build`, with no Template App windows open. Uses a temp
# XDG_CONFIG_HOME and XDG_STATE_HOME. Screenshots go to $OUT.
cd "$(dirname "$0")/../.." || exit 1
. scripts/manual/lib.sh
require_no_instance

T=$(mktemp -d); trap 'cleanup; rm -rf "$T"' EXIT
OUT=${OUT:-$T/shots}; mkdir -p "$OUT"
export XDG_STATE_HOME=$T/state

launch "$T/cfg"
check "one window (WIN-1)" 1 "$(win_count)"
check "app ID (WIN-3)" "$APP" "$(active .class)"
check "title" "Template App" "$(active .title)"
# Hyprland tiles new windows, so only the headless sway (floating) shows the default size.
[ -n "${HEADLESS:-}" ] && check "default size in logical px" "800 600" "$(active '"\(.size[0]) \(.size[1])"')"
check "config.ini created (CONF-3)" yes "$([ -f "$T/cfg/template-app/config.ini" ] && echo yes)"
check "keybindings.ini created" yes "$([ -f "$T/cfg/template-app/keybindings.ini" ] && echo yes)"
shot "$OUT/window.png"

keys -M ctrl n -m ctrl
check "new-window opens a second window (WIN-2)" 2 "$(win_count)"
check "in the same process" 1 "$(wins | jq '[.[].pid] | unique | length')"
keys -M ctrl w -m ctrl
check "close-window closes one (WIN-4)" 1 "$(win_count)"
focus_app; sleep 0.3
keys -M ctrl n -m ctrl
keys -M ctrl q -m ctrl
sleep 0.5
check "quit closes every window" 0 "$(win_count)"
check "the process exits" no "$(kill -0 "$PID" 2>/dev/null && echo yes || echo no)"

echo "screenshots in $OUT"
[ "$FAILS" = 0 ]
