# crowterm — PLAN

## **DO NOT ASK USER FOR FEEDBACK — THIS IS THE USER FEEDBACK.**
## **DO NOT ASK USER FOR NEXT STEPS — THESE ARE THE NEXT STEPS.**

Build: `cargo build -p rioterm`
Install: `cargo install --path frontends/rioterm --features wgpu,audio --force`
Test gate: `cargo test -p rioterm -p rio-backend` + live eyeball (this is UI work)
Upstream: `upstream` = raphamorim/rio, `origin` = crow-cli/rio (our fork)

## Phase 1 — Recon + design doc
1.1 Write `docs/crowterm.md`: dock model (taffy tree), socket protocol
    (list/spawn/focus/write/close/open-file), app-registry TOML, open-file +
    pwd-follow flows, default keymap.
    ✓ when: doc references real code paths (file:line) and the protocol covers
    all six commands. (29Aug2026: inventory done, doc written.)

## Phase 2 — Dock frame v1
2.1 Fixed outer frame around the existing per-tab trees:
    root column → [top row → (main area | sidebar), bottom bar].
2.2 Sidebar spawns yazi (locked: not in tab strip, survives tab close);
    bottom bar spawns $SHELL.
2.3 Toggle binds: Super+B sidebar, Super+J bottom bar; divider sizes stored as
    fractions per window.
    ✓ when: `cargo build -p rioterm` clean; live layout matches the ASCII in
    TODO.md; toggles work; tab switching unaffected.

## Phase 3 — [+] new menu / app registry
3.1 Config `[[apps]]` (name/command); compiled-in defaults: New terminal
    ($SHELL), New crow-cli session (`crow-cli`), New file (hx). User entries
    append.
3.2 [+] button right of the tab bar in Island; click → dropdown; entry spawns
    a main-panel tab. Mirror entries into the command palette.
    ✓ when: menu opens on click; all three builtins spawn the right thing;
    a user-defined app appears and spawns.

## Phase 4 — Control socket
4.1 Unix socket server in the rioterm event loop, `$XDG_RUNTIME_DIR/crowterm.sock`,
    JSON lines, 0600. Responses `{"ok":true,...}` / `{"ok":false,"error":...}`.
4.2 Commands: list, spawn{command,cwd,target}, focus{tab}, write{target,text},
    close{tab}, open-file{path}. v1 targets the focused window.
4.3 CLI client `rio ctl '<json>'` so scripts/yazi need no socat.
    ✓ when: cargo test covers parse/dispatch; manual spawn+focus+write
    round-trip from a shell works.

## Phase 5 — Glue (the zide replacement)
5.1 open-file: hx tab exists → focus + write `:open <path>\n` (zide-edit
    semantics, native PTY write — no sleep hack); else spawn `hx <path>`.
5.2 Ship yazi opener config: opener invokes `rio ctl` open-file.
5.3 Explorer pwd-follows active tab: try `ya pub` DDS into running yazi first
    (small shipped lua plugin); fallback respawn sidebar with new cwd.
    ✓ when: click file in yazi → opens in hx tab (focused if already open);
    switching to a tab with a different cwd moves the explorer.

## Phase 6 — crow-cli first-class + defaults
6.1 crow-cli menu entry + default keybind (see docs/crowterm.md keymap).
6.2 SOP theme + DejaVu Sans Mono as in-repo defaults (no user config needed).
    ✓ when: fresh install with no config renders the SOP look; the keybind and
    menu entry launch the crow TUI.

## Phase 7 — Rebrand (LAST)
7.1 Display strings/binary name → crowterm (keep crate names merge-friendly
    with upstream unless that hurts; decide here).
    ✓ when: installed binary is crowterm; an upstream merge from
    raphamorim/rio still applies without hand-fixing the brand.
