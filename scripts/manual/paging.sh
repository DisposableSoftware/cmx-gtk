#! /usr/bin/env nix-shell
#! nix-shell -i bash -p wtype jq grim
# Spec: READ-5, READ-20, READ-21, READ-30, READ-31, WIN-6, WIN-7, WIN-12, KEY-5, INT-21.
# Run from the repo root after `nix build`, with no cmx windows open.
cd "$(dirname "$0")/../.." || exit 1
. scripts/manual/lib.sh
require_no_instance

T=$(mktemp -d); trap 'cleanup; rm -rf "$T"' EXIT
OUT=${OUT:-$T/shots}; mkdir -p "$OUT"
export XDG_STATE_HOME=$T/state XDG_DATA_HOME=$T/data
P="Fixture Saga #3 — The Wide Page ·"

launch "$T/cfg" "$FIX/six.cbz"
wait_for "opens at page 1 (WIN-7)" title "$P 1 / 6"
shot "$OUT/page1.png"
keys -k Left -k Home
check "previous and first do nothing at the start (READ-20)" "$P 1 / 6" "$(title)"
before=$(size)
keys d
check "dual-page pairs 1 and 2 (READ-30)" "$P 1–2 / 6" "$(title)"
check "toggling doesn't resize (READ-31)" "$before" "$(size)"
shot "$OUT/spread-1-2.png"
keys -k space
check "a wide page 3 stands alone" "$P 3 / 6" "$(title)"
shot "$OUT/wide-3.png"
keys -k Right
check "then 4–5" "$P 4–5 / 6" "$(title)"
keys j
check "then 6 alone" "$P 6 / 6" "$(title)"
keys -k Down -k End
check "next and last do nothing at the end" "$P 6 / 6" "$(title)"
keys k
check "back two in dual-page" "$P 4–5 / 6" "$(title)"
keys -M shift -k space -m shift
check "shift-space goes back two; 2 stands alone next to wide 3" "$P 2 / 6" "$(title)"

keys d -k Home
check "single page again" "$P 1 / 6" "$(title)"
# Five quick presses (READ-21): the last one wins.
wtype -d 20 -k Shift_L lllll; sleep 1
check "fast paging lands on the last page" "$P 6 / 6" "$(title)"
keys -k Home
wtype -k Shift_L -P l -s 1500 -p l; sleep 1
check "holding l repeats to the last page (KEY-5)" "$P 6 / 6" "$(title)"

keys f
sleep 0.5
check "f goes fullscreen (WIN-12)" "1920 1080" "$(size)"
shot "$OUT/fullscreen.png"
keys -k Escape
sleep 0.5
check "Escape leaves" yes "$([ "$(size)" != "1920 1080" ] && echo yes)"
keys -k Escape
check "Escape outside fullscreen does nothing" 1 "$(win_count)"
keys q; sleep 1

launch "$T/cfg" "$FIX/formats.cbz"
wait_for "a WebP page decodes (INT-21)" title "formats · 1 / 2"
check "no error" 1 "$(win_count)"
shot "$OUT/webp.png"
keys l
check "an AVIF page decodes" "formats · 2 / 2 1" "$(title) $(win_count)"
shot "$OUT/avif.png"
keys q; sleep 1

launch "$T/cfg" "$FIX/broken-page.cbz"
wait_for "broken-page opens" title "broken-page · 1 / 3"
keys l
wait_for "an undecodable page shows an error (READ-5)" win_count 2
shot "$OUT/undecodable.png"
keys -k Return
wait_for "OK" win_count 1
keys l
check "the next page still works" "broken-page · 3 / 3" "$(title)"
keys q; sleep 1

echo "screenshots in $OUT"
[ "$FAILS" = 0 ]
