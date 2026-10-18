#! /usr/bin/env nix-shell
#! nix-shell -i bash -p wtype jq grim
# Spec: WIN-1, WIN-2, WIN-3, WIN-4, WIN-5, WIN-14, WIN-15, CONF-3.
# Run from the repo root after `nix build`, with no cmx windows open. Uses temp XDG dirs.
# Screenshots go to $OUT.
cd "$(dirname "$0")/../.." || exit 1
. scripts/manual/lib.sh
require_no_instance

T=$(mktemp -d); trap 'cleanup; rm -rf "$T"' EXIT
OUT=${OUT:-$T/shots}; mkdir -p "$OUT"
export XDG_STATE_HOME=$T/state XDG_DATA_HOME=$T/data

launch "$T/cfg"
check "one window (WIN-1)" 1 "$(win_count)"
check "app ID (WIN-3)" "$APP" "$(active .class)"
check "title with no comic (WIN-7)" "cmx" "$(title)"
# Hyprland tiles new windows, so only the headless sway (floating) shows the default size.
[ -n "${HEADLESS:-}" ] && check "starting size in logical px (CONF-4)" "800 1000" "$(size)"
check "config.ini created (CONF-3)" yes "$([ -f "$T/cfg/cmx/config.ini" ] && echo yes)"
check "keybindings.ini created" yes "$([ -f "$T/cfg/cmx/keybindings.ini" ] && echo yes)"
shot "$OUT/placeholder.png"

keys l -k End -k Home d
check "page keys do nothing without a comic (WIN-14)" "cmx" "$(title)"
check "and open nothing" 1 "$(win_count)"
keys -M ctrl i -m ctrl
shot "$OUT/placeholder-after-info-key.png"

keys -k F1
check "about opens a dialog (WIN-15)" 2 "$(win_count)"
check "titled About cmx" "About cmx" "$(title)"
shot "$OUT/about.png"
keys -k Escape
check "the dialog closes" 1 "$(win_count)"

focus_app; sleep 0.3
keys -M ctrl n -m ctrl
check "new-window opens a second window (WIN-2)" 2 "$(win_count)"
check "in the same process" 1 "$(wins | jq '[.[].pid] | unique | length')"
keys -M ctrl w -m ctrl
check "close-window closes one (WIN-4)" 1 "$(win_count)"
focus_app; sleep 0.3
keys -M ctrl n -m ctrl
keys q
sleep 0.5
check "quit (q) closes every window" 0 "$(win_count)"
check "the process exits" no "$(kill -0 "$PID" 2>/dev/null && echo yes || echo no)"

echo "screenshots in $OUT"
[ "$FAILS" = 0 ]
