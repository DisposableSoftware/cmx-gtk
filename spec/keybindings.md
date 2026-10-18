# Keybindings

## Requirements

| ID | Requirement | Status | Verified by |
|----|-------------|--------|-------------|
| `KEY-1` | Every user-facing action has a binding name, and its defaults are listed in the table below. | done | `spec_tests::keybindings_match_spec` |
| `KEY-2` | Users override a binding in the `[keybindings]` group of `$XDG_CONFIG_HOME/template-app/keybindings.ini`, writing `name = accel;accel`. An empty value unbinds it. A binding that isn't listed keeps its defaults. | done | `config::tests::reads_settings_and_keybindings_from_their_files`, `keys::tests::accelerators` |
| `KEY-3` | Unknown binding names and accelerators that can't be parsed are warned about on stderr and skipped. They never crash the app. | done | `keys::tests::accelerators` (a bad accelerator is dropped) |
| `KEY-4` | Accelerator syntax is GTK's (`gtk_accelerator_parse`): `<Ctrl>`, `<Shift>`, `<Alt>`, `<Super>` plus a GDK key name. | done | `keys::tests::accelerators` |

## Default bindings

<!-- spec:keybindings:begin -->
| Name | Action | Default keys |
|------|--------|--------------|
| `new-window` | `win.new-window` | `<Ctrl>n` |
| `close-window` | `window.close` | `<Ctrl>w` |
| `quit` | `app.quit` | `<Ctrl>q` |
<!-- spec:keybindings:end -->

## Implementation notes

- To add a binding, edit the table above, `keys::BINDINGS`, `data/keybindings.example.ini` and the man page's KEY BINDINGS section (`INT-32`). The spec tests fail until they agree.
- Bindings without Ctrl, Alt or Super (letters, Space, Delete) set with `set_accels_for_action` also fire while typing in a text field. Before adding one, attach those to the window with a capture-phase `ShortcutController` that passes keys through while an `Editable` has focus.
