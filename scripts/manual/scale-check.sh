#! /usr/bin/env nix-shell
#! nix-shell -i bash -p jq grim imagemagick
# Spec: LOOK-1. One scale; run through scales.sh, which starts a headless sway per scale.
cd "$(dirname "$0")/../.." || exit 1
. scripts/manual/lib.sh
[ -n "${HEADLESS:-}" ] || { echo "run through scripts/manual/scales.sh" >&2; exit 2; }
require_no_instance

T=$(mktemp -d); trap 'cleanup; rm -rf "$T"' EXIT
OUT=${OUT:-$T}
export XDG_STATE_HOME=$T/state XDG_DATA_HOME=$T/data
scale=$(display_scale)

launch "$T/cfg"
check "logical size at scale $scale" "800 1000" "$(active '"\(.size[0]) \(.size[1])"')"
shot "$OUT/window.png"
# grim captures at the output's scale, so the shot is in device px: never upscaled.
expect=$(awk -v s="$scale" 'BEGIN { printf "%d %d", 800 * s + 0.5, 1000 * s + 0.5 }')
check "screenshot in device px" "$expect" "$(magick identify -format '%w %h' "$OUT/window.png")"
[ "$FAILS" = 0 ]
