# crowterm — design

rio as a TUI IDE: zed/vscode-style chrome where every pane is still just a
terminal. No zellij, no embedded widgets — "the protocol is the terminal":
each pane is a sized PTY running a TUI app (helix, yazi, crow-cli, $SHELL),
and crowterm is the multiplexer with IDE chrome that glues them.

Target layout (fs sidebar on the RIGHT):

    ┌─────────────────────┬───────┐
    │ tab1 │ tab2 │   [+] │       │
    ├─────────────────────┤  fs   │
    │                     │(yazi) │
    │   main panel        │       │
    │   (tabs only)       │       │
    ├─────────────────────┴───────┤
    │      bottom terminal bar    │
    └─────────────────────────────┘

## Dock model

Per window, a fixed outer taffy tree wraps the existing per-tab trees
(`frontends/rioterm/src/screen/mod.rs`, layout pass `apply_taffy_layout`;
taffy 0.10 flexbox):

    root (column)
    ├── top (row, flex 1)
    │   ├── main (flex 1): Island tab strip + current tab's grid(s)
    │   └── sidebar (fixed width fraction, toggleable): yazi grid
    └── bottom (fixed height fraction, toggleable): shell grid

- Sidebar and bottom are **window-owned grids**, not tabs: they never appear
  in the Island strip and are exempt from `CloseCurrentSplitOrTab`.
- Sizes: sidebar width and bottom height stored as fractions; adjusted with
  the existing divider-move machinery (`MoveDivider*`); remembered per window
  for the session (persistence across restarts is deferred).
- Main panel v1 = tabs only, one grid per tab. Existing `SplitRight`/
  `SplitDown` keep working *inside* a tab (harmless), but dock regions
  themselves never split and the split UX is not advertised until later.
- Toggles: `Super+B` sidebar, `Super+J` bottom bar (new Actions, wired in
  `frontends/rioterm/src/bindings/mod.rs` alongside `ToggleQuake`, whose
  show/hide animation path is the closest existing precedent).

## App registry (the [+] menu)

Config (rio config.toml, hot-reloaded like everything else):

    [[apps]]
    name = "New terminal"
    command = ""            # empty = default shell

    [[apps]]
    name = "New crow-cli session"
    command = "crow-cli"    # bare invocation = the TUI

    [[apps]]
    name = "New file (hx)"
    command = "hx"

Those three are compiled-in defaults; user `[[apps]]` entries append (same
name = override). UI: a [+] button at the RIGHT end of the Island tab bar
(`renderer/island.rs`) opening a dropdown (rendering approach: reuse the
command-palette overlay machinery in `renderer/command_palette.rs` where
possible). Every entry also lands in the command palette as
"New: <name>". Selection spawns a main-panel tab via the same path as
`split_right_with_config`/tab creation (screen/mod.rs:1537 shows how a pane
gets its own config/program).

## Control socket

Path: `$XDG_RUNTIME_DIR/crowterm.sock`, mode 0600, single owner instance.
Framing: one JSON object per line in, one line out:
`{"ok":true,...}` or `{"ok":false,"error":"..."}`.

| command | args | effect |
|---|---|---|
| `list` | — | windows/tabs: id, title, cwd, focused; sidebar/bottom state |
| `spawn` | `command[]`, `cwd?`, `target: tab\|sidebar\|bottom` | spawn pane; returns tab id |
| `focus` | `tab` | switch to tab |
| `write` | `target: tab\|sidebar\|bottom`, `tab?`, `text` | raw PTY write (keystroke injection) |
| `close` | `tab` | close tab |
| `open-file` | `path` | focus-or-spawn helix (below) |

v1 targets the focused window. Client: `rio ctl '<json>'` subcommand so yazi
hooks and scripts need no socat.

## Flows

### Open file from yazi (replaces zide-pick/zide-edit, ref /tmp/zide)

yazi's opener config ships with crowterm and invokes
`rio ctl '{"cmd":"open-file","path":"..."}'`. crowterm then:

1. If a main tab is a helix session (tracked at spawn; title heuristic as
   backup) → `focus` it and `write` `:open <path>\n` to its PTY.
2. Else `spawn` `hx <path>` in a new main tab.

zide achieved this with `zellij action write-chars` plus `sleep $(len/150)`
timing hacks; we write straight into the PTY, and the kernel pty buffer
handles flow — no sleeps. Verify with long paths during Phase 5.

### Explorer pwd-follow

On main-tab switch, the sidebar yazi follows the tab's cwd:

1. Preferred: `ya pub` (yazi DDS) into the running instance; a small shipped
   lua plugin subscribes and changes directory.
2. Fallback: respawn the sidebar grid with the new cwd (loses yazi state —
   acceptable only if DDS proves impractical).

### crow-cli first-class

`crow-cli` (bare = TUI) is a builtin app-registry entry plus a default
keybind (`Super+Shift+A`, agent — final choice in Phase 6). Long-term:
crow-cli drives crowterm over the same socket (spawn panes, read output),
which is what makes this an *AI*-native terminal.

## Defaults shipped in-repo

SOP theme (`#1E1D40` bg, Dracula-purple chrome accents) + DejaVu Sans Mono,
as compiled-in defaults so a fresh install with no user config already looks
right. (User config at ~/.config/rio/config.toml still overrides — the live
one there today is the reference.)

## Explicit non-goals (v1)

Main-panel split UX, session persistence, native file-tree widget, multi-window
socket routing, rebrand (Phase 7, last).

## Phase 2 integration points (verified 29Aug2026)

Model: `ContextGrid` (layout/mod.rs:108) = one TAB = its own `TaffyTree`
(root_node → panel leaves → `ContextGridItem` = PTY `Context`).
`ContextManager` (context/mod.rs:132) = one WINDOW = `SmallVec<ContextGrid>`
+ `current_index`. `Screen` (screen/mod.rs) owns ContextManager + sugarloaf
+ Island.

Layout flow on resize:
`Screen::resize` (screen/mod.rs:583) → `resize_all_contexts` (:680) →
`ContextGrid::resize(w, h, sugarloaf)` (layout/mod.rs:1265) →
`apply_taffy_layout` (:897) → `compute_layout` (:290); each grid item lands
in `layout_rect`, which the renderer reads directly.

Dock approach: Screen gains `sidebar_frac`/`bottom_frac` + visibility flags.
On resize, tab ContextGrids receive the MAIN-area rect (window minus island,
sidebar, bottom) instead of the full window; two window-owned Contexts
(sidebar=yazi, bottom=$SHELL, not in the tab SmallVec) get their rects from
the fractions. Island height offset already exists (see
`split_right_with_config`, screen/mod.rs:1537). Toggles modeled on
`ToggleQuake`'s show/hide path.
