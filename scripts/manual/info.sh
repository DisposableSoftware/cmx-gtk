#! /usr/bin/env nix-shell
#! nix-shell -i bash -p wtype jq grim
# Spec: WIN-13, KEY-5. Screenshots of the info panel with and without metadata go to $OUT.
cd "$(dirname "$0")/../.." || exit 1
. scripts/manual/lib.sh
require_no_instance

T=$(mktemp -d); trap 'cleanup; rm -rf "$T"' EXIT
OUT=${OUT:-$T/shots}; mkdir -p "$OUT"
export XDG_STATE_HOME=$T/state XDG_DATA_HOME=$T/data

launch "$T/cfg" "$FIX/six.cbz"
wait_for "six opens" title "Fixture Saga #3 — The Wide Page · 1 / 6"
before=$(size)
keys -M ctrl i -m ctrl
check "the panel doesn't resize the window" "$before" "$(size)"
shot "$OUT/info-metadata.png"
keys l
check "paging still works with the panel open (KEY-5)" "Fixture Saga #3 — The Wide Page · 2 / 6" "$(title)"
# Tab moves focus into the panel's selectable values; paging keys still page.
keys -k Tab -k Tab
keys -k Right j
check "paging works with focus in the panel's text (KEY-5)" "Fixture Saga #3 — The Wide Page · 4 / 6" "$(title)"
shot "$OUT/info-focused.png"
keys -M ctrl i -m ctrl
shot "$OUT/info-hidden.png"
keys q; sleep 1

launch "$T/cfg" "$FIX/plain.cbz"
wait_for "plain opens" title "plain · 1 / 3"
keys -M ctrl i -m ctrl
shot "$OUT/info-none.png"
keys q; sleep 1

echo "screenshots in $OUT"
[ "$FAILS" = 0 ]
