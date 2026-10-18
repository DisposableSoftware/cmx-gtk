#! /usr/bin/env nix-shell
#! nix-shell -i bash -p wtype jq grim
# Spec: WIN-1, WIN-2, WIN-8, WIN-9, WIN-10, WIN-11, READ-40, READ-41, READ-42, INT-2.
# Run from the repo root after `nix build`, with no cmx windows open; headless by default
# (scripts/manual/headless.sh), where windows float, so WIN-11's resize is visible.
cd "$(dirname "$0")/../.." || exit 1
. scripts/manual/lib.sh
require_no_instance

T=$(mktemp -d); trap 'cleanup; rm -rf "$T"' EXIT
OUT=${OUT:-$T/shots}; mkdir -p "$OUT"
export XDG_STATE_HOME=$T/state XDG_DATA_HOME=$T/data
mkdir -p "$T/comics"
cp "$FIX/six.cbz" "$FIX/plain.cbz" "$FIX/rar-disguised.cbz" "$T/comics/"
ln -s "$T/comics/six.cbz" "$T/comics/link.cbz"
C=$T/comics

launch "$T/cfg" "$C/six.cbz" "$C/plain.cbz"
wait_for "a window per file (WIN-1)" win_count 2
titles=$(wins | jq -r '[.[].title] | sort | join(" | ")')
check "both comics open" "Fixture Saga #3 — The Wide Page · 1 / 6 | plain · 1 / 3" "$titles"

if [ -n "${HEADLESS:-}" ]; then
  focus_app; swaymsg '[title="^Fixture"] focus' >/dev/null; sleep 0.3
  # 1920×1080 logical: a never-sized window takes 85% of the height for the page area.
  read -r w h <<<"$(size)"
  check "fits the page's aspect (WIN-11): width" 612 "$w"
  check "page area 918 px high, plus the title bar if any" yes "$([ "$h" -ge 918 ] && [ "$h" -lt 1000 ] && echo yes)"
  shot "$OUT/six-fitted.png"
fi

XDG_CONFIG_HOME=$T/cfg "$BIN" "$C/link.cbz" & sleep 1.5
check "the same comic again, through a symlink, opens nothing (WIN-9)" 2 "$(win_count)"
XDG_CONFIG_HOME=$T/cfg "$BIN" "$C/rar-disguised.cbz" & sleep 1.5
wait_for "a RAR opens a window and an error dialog (WIN-2, WIN-10)" win_count 4
check "in the running process" 1 "$(wins | jq '[.[].pid] | unique | length')"
shot "$OUT/rar-error.png"
keys -k Return
wait_for "OK closes the dialog" win_count 3
check "the failed window shows the placeholder" 1 "$(wins | jq '[.[] | select(.title == "cmx")] | length')"

# Ctrl+O in the empty window opens the comic there (WIN-8). Typing a path opens the
# file chooser's location entry.
swaymsg '[title="^cmx$"] focus' >/dev/null 2>&1 || focus_app; sleep 0.3
if [ "$(title)" = cmx ]; then
  cp "$FIX/sample.cbz" "$C/"
  keys -M ctrl o -m ctrl
  sleep 1
  shot "$OUT/file-dialog.png"
  wtype -d 30 -k Shift_L "$C/sample.cbz"; sleep 0.5
  keys -k Return
  sleep 1
  check "Ctrl+O reuses the empty window" 3 "$(win_count)"
  wait_for "and shows the comic" "wins | jq -r '[.[] | select(.title | startswith(\"Test Series\"))] | length'" 1
fi

# Progress: move two pages on in six, quit, and start again with no files.
swaymsg '[title="^Fixture"] focus' >/dev/null 2>&1 || focus_app; sleep 0.3
check "six is still at page 1" "Fixture Saga #3 — The Wide Page · 1 / 6" "$(title)"
keys ll
check "two pages on" "Fixture Saga #3 — The Wide Page · 3 / 6" "$(title)"
keys q; sleep 1
check "quit" 0 "$(win_count)"
check "recent files list six (READ-42)" yes "$(grep -q "six.cbz" "$T/data/recently-used.xbel" && echo yes)"
check "but not the RAR" no "$(grep -q "rar-disguised" "$T/data/recently-used.xbel" && echo yes || echo no)"

launch "$T/cfg" "$C/six.cbz"
wait_for "reopening resumes at the saved page (READ-40)" title "Fixture Saga #3 — The Wide Page · 3 / 6"
keys q; sleep 1
launch "$T/cfg"
wait_for "no files: the last comic opens (READ-41)" title "Fixture Saga #3 — The Wide Page · 3 / 6"
keys q; sleep 1
mv "$C/six.cbz" "$C/moved.cbz"
launch "$T/cfg"
sleep 1
check "a last comic that moved leaves the placeholder, no error" "1 cmx" "$(win_count) $(title)"
keys q; sleep 1

echo "screenshots in $OUT"
[ "$FAILS" = 0 ]
