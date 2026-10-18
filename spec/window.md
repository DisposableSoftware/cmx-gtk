# Window

## Requirements

| ID | Requirement | Status | Verified by |
|----|-------------|--------|-------------|
| `WIN-1` | Starting the app opens a window titled "cmx" that shows `[view] greeting` (`CONF-5`) centred, at the size from `[window] width` and `height` (`CONF-4`). | done | `scripts/manual/smoke.sh` (screenshot; the window is 800×600 logical px by default) |
| `WIN-2` | The app is single-instance: starting it again while it runs opens another window in the running process. `new-window` does the same from a window. | done | `scripts/manual/smoke.sh` (`<Ctrl>n` opens a second window in the same pid) |
| `WIN-3` | Every window has the Wayland app ID `io.github.DisposableSoftware.cmx`, so compositor window rules match it. | done | `scripts/manual/smoke.sh` (headless: `swaymsg` reports the app ID) |
| `WIN-4` | `close-window` closes the focused window; closing the last window quits the app. `quit` closes every window. | done | `scripts/manual/smoke.sh` |
