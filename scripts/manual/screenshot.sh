#! /usr/bin/env nix-shell
#! nix-shell -i bash -p wtype jq grim
# Takes data/screenshots/reader.png (README and AppStream, INT-41): the demo comic in
# dual-page layout with the info panel open. Run through headless.sh:
#   HEADLESS_SCALE=1 HEADLESS_RES=1920x1080 scripts/manual/headless.sh scripts/manual/screenshot.sh
cd "$(dirname "$0")/../.." || exit 1
. scripts/manual/lib.sh
[ -n "${HEADLESS:-}" ] || { echo "run through scripts/manual/headless.sh" >&2; exit 2; }
require_no_instance

T=$(mktemp -d); trap 'cleanup; rm -rf "$T"' EXIT
export XDG_STATE_HOME=$T/state XDG_DATA_HOME=$T/data

launch "$T/cfg" "$FIX/demo.cbz"
wait_for "demo opens" title "Night Reading #1 — Just One More Page · 1 / 3"
swaymsg '[app_id="'"$APP"'"] resize set 1440 900' >/dev/null; sleep 0.5
keys d
keys -M ctrl i -m ctrl
check "dual-page with the panel" "Night Reading #1 — Just One More Page · 1–2 / 3" "$(title)"
shot data/screenshots/reader.png
keys q
[ "$FAILS" = 0 ]
