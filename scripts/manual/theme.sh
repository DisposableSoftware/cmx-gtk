#! /usr/bin/env nix-shell
#! nix-shell -i bash -p wtype jq grim
# Spec: LOOK-2, LOOK-3, LOOK-4, LOOK-5, LOOK-7, LOOK-10, CONF-6. Screenshots of the empty
# window, a comic with the info panel open and About, per flavor, go to $OUT.
# Run through headless.sh after `nix build`.
cd "$(dirname "$0")/../.." || exit 1
. scripts/manual/lib.sh
[ -n "${HEADLESS:-}" ] || { echo "run through scripts/manual/headless.sh" >&2; exit 2; }
require_no_instance

T=$(mktemp -d); trap 'cleanup; rm -rf "$T"' EXIT
OUT=${OUT:-$T/shots}; mkdir -p "$OUT"
export XDG_STATE_HOME=$T/state XDG_DATA_HOME=$T/data

for flavor in ${FLAVORS:-mocha latte}; do
  mkdir -p "$T/cfg/cmx"
  printf '[appearance]\nflavor=%s\naccent=%s\n' "$flavor" "${ACCENT:-mauve}" > "$T/cfg/cmx/config.ini"
  rm -rf "$T/state"
  launch "$T/cfg"
  check "$flavor: empty window" cmx "$(title)"
  shot "$OUT/$flavor-empty.png"
  keys -k F1
  shot "$OUT/$flavor-about.png"
  keys -k Escape
  keys q; sleep 1

  launch "$T/cfg" "$FIX/six.cbz"
  wait_for "$flavor: six opens" title "Fixture Saga #3 — The Wide Page · 1 / 6"
  keys -M ctrl i -m ctrl
  shot "$OUT/$flavor-info.png"
  keys q; sleep 1
done
echo "screenshots in $OUT"
[ "$FAILS" = 0 ]
