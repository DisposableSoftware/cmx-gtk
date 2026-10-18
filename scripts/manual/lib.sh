# Helpers for driving the app on Hyprland, or in a headless sway under headless.sh.
# Source from a script run in the repo root. See "Manual testing" in CLAUDE.md for why
# each guard exists.
set -u

APP=io.github.DisposableSoftware.cmx
BIN=${BIN:-./result/bin/cmx}

wins() { hyprctl clients -j | jq "[.[] | select(.class==\"$APP\")]"; }
win_count() { wins | jq length; }
active() { hyprctl activewindow -j | jq -r "$1"; }
focus_app() { hyprctl dispatch "hl.dsp.focus({ window = \"class:$APP\" })" >/dev/null; }
display_scale() { hyprctl monitors -j | jq ".[] | select(.focused) | .scale"; }

# Under scripts/manual/headless.sh the windows live in a headless sway with its own
# D-Bus session, so the user's running app and lock screen don't matter.
if [ -n "${HEADLESS:-}" ]; then
  # Same fields as hyprctl clients -j: class, pid, title, at, size, floating, focused.
  sway_wins() {
    swaymsg -t get_tree | jq "[recurse(.nodes[]?, .floating_nodes[]?) | select(.app_id != null)
      | {class: .app_id, pid, title: .name, at: [.rect.x, .rect.y],
         size: [.rect.width, .rect.height], floating: (.type == \"floating_con\"), focused}]"
  }
  wins() { sway_wins | jq "[.[] | select(.class==\"$APP\")]"; }
  active() { sway_wins | jq -r "(.[] | select(.focused)) // {} | $1"; }
  focus_app() { swaymsg "[app_id=\"$APP\"] focus" >/dev/null; }
  display_scale() { swaymsg -t get_outputs | jq ".[] | select(.focused) | .scale"; }
  # Scripts read the pointer with hyprctl cursorpos and move it with wlrctl. Sway has no
  # cursor query, so track the pointer here: park it at 0,0 on the first move, then add
  # each move, clamped to the output like sway does.
  PTR=
  wlrctl() {
    if [ "$1 $2" != "pointer move" ]; then command wlrctl "$@"; return; fi
    local w h x y
    read -r w h < <(swaymsg -t get_outputs | jq -r '.[0].rect | "\(.width) \(.height)"')
    if [ -z "$PTR" ]; then command wlrctl pointer move -100000 -100000; PTR="0 0"; fi
    command wlrctl pointer move "$3" "$4"
    read -r x y <<<"$PTR"
    x=$((x + $3)); y=$((y + $4))
    x=$((x < 0 ? 0 : x > w - 1 ? w - 1 : x)); y=$((y < 0 ? 0 : y > h - 1 ? h - 1 : y))
    PTR="$x $y"
  }
  # Never reach the user's real Hyprland: only queries the headless sway can answer work.
  hyprctl() {
    local x y
    case "$1" in
      cursorpos) read -r x y <<<"${PTR:-0 0}"; printf '{"x": %d, "y": %d}\n' "$x" "$y" ;;
      monitors) swaymsg -t get_outputs | jq '[.[] | {name, focused, scale, x: .rect.x, y: .rect.y}]' ;;
      *) echo "headless: hyprctl $* is not available; run this check on Hyprland" >&2; return 1 ;;
    esac
  }
  # ydotool types into the kernel's input devices, so it would act on the real session.
  ydotool() { echo "headless: ydotool acts on the real session; run this check on Hyprland" >&2; return 1; }
fi

# The app is single-instance: launching while the user has it open would join their
# process, and killing it would close their windows.
require_no_instance() {
  if [ "$(win_count)" != 0 ]; then
    echo "$APP is already running; close it first (refusing to touch the user's windows)" >&2
    exit 2
  fi
  [ -n "${HEADLESS:-}" ] && return
  if pidof hyprlock >/dev/null; then echo "screen is locked" >&2; exit 2; fi
}

# launch [XDG_CONFIG_HOME] [ARGS...]: starts the app and focuses it. Sets PID.
launch() {
  local cfg=${1:-}
  shift || true
  [ -x "$BIN" ] || { echo "run nix build first" >&2; exit 2; }
  if [ -n "$cfg" ]; then XDG_CONFIG_HOME=$cfg "$BIN" "$@" >"${LOG:-/dev/null}" 2>&1 &
  else "$BIN" "$@" >"${LOG:-/dev/null}" 2>&1 & fi
  for _ in $(seq 25); do [ "$(win_count)" -gt 0 ] && break; sleep 0.2; done
  sleep 0.4
  PID=$(wins | jq -r '.[0].pid')
  focus_app
  sleep 0.3
}

# keys WTYPE-ARGS...: sends keys only if one of the app's windows is focused.
keys() {
  if [ "$(active .class)" != "$APP" ]; then
    echo "$APP lost focus; stopping so keys don't go to another app" >&2
    cleanup; exit 3
  fi
  # Without a per-key delay wtype sends a whole sequence instantly, faster than GTK
  # handles focus moves; real typing is never that fast.
  # Headless sway drops the first key after wtype uploads its keymap; a Shift tap absorbs it.
  if [ -n "${HEADLESS:-}" ]; then wtype -d 100 -k Shift_L "$@"; else wtype -d 100 "$@"; fi
  sleep 0.6
}

shot() { grim -g "$(active '"\(.at[0]),\(.at[1]) \(.size[0])x\(.size[1])"')" "$1"; }

cleanup() { [ -n "${PID:-}" ] && kill "$PID" 2>/dev/null; true; }

FAILS=0
check() { # check DESCRIPTION EXPECTED ACTUAL
  if [ "$2" = "$3" ]; then echo "PASS  $1"; else echo "FAIL  $1 (expected '$2', got '$3')"; FAILS=$((FAILS+1)); fi
}

FIX=$PWD/tests/fixtures
title() { active .title; }
size() { active '"\(.size[0]) \(.size[1])"'; }
# wait_for DESCRIPTION COMMAND EXPECTED: polls COMMAND for up to 5 s, then checks it.
wait_for() {
  local got
  for _ in $(seq 25); do got=$(eval "$2"); [ "$got" = "$3" ] && break; sleep 0.2; done
  check "$1" "$3" "$got"
}
