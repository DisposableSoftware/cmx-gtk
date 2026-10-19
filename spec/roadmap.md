# Roadmap

The order of approved work: `planned` requirements, and `partial` ones with a known bug from `/bug`. Change it here when priorities change. `/implement` works through this file: the verification backlog first, then the requirements in the order listed.

`INT-52` is ready to implement. The rows under "Awaiting a decision" are not: `/implement` skips them until the user settles each one and the row text changes.

1. `INT-52`: README follows the style guide's shape.

## Awaiting a decision

Planned rows that change an existing requirement or contradict one. Each needs the user's choice first, and the choice may be to keep the current behaviour and record the exception.

- `LOOK-12`: latte contrast of the empty window on the black page area. Changing it amends `WIN-5` and `LOOK-3`.
- `LOOK-13`: the error dialog takes the flavor. Changing it amends `WIN-10`.
- `WIN-16`: shortcut overlay on `?`. Building it amends `WIN-6`; declining it records an exception to the style guide.

## Verification backlog

Built requirements that are not fully verified. Clear these before starting new planned work.

- `LOOK-6`: focus outline. Find a way to focus a button (the About dialog's) and take a screenshot.
- `LOOK-10`: `flavor = gtk`. Take a headless screenshot with a GTK theme set.
- `INT-50` in a file manager: the `.thumbnailer` entry is installed and `cmx --thumbnail` works, but no file manager has been seen using it yet.
