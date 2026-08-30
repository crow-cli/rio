# crowterm — TODO

## **DO NOT ASK USER FOR FEEDBACK — THIS IS THE USER FEEDBACK.**
## **DO NOT ASK USER FOR NEXT STEPS — THESE ARE THE NEXT STEPS.**

Mission: turn rio into a TUI IDE ("crowterm") — zed/vscode-style chrome where
every pane is still just a terminal. No zellij: built in Rust on rio's existing
tabs (Island) + taffy splits. crow-cli TUI, helix, yazi are first-class
citizens. "The protocol is the terminal": each pane = a sized PTY running a TUI
app; crowterm glues them.

User's layout (fs sidebar on the RIGHT — user corrected this twice, 29Aug2026):

    ┌─────────────────────┬───────┐
    │ tab1 │ tab2 │   [+] │       │
    ├─────────────────────┤  fs   │
    │                     │(yazi) │
    │   main panel        │       │
    │   (tabs only)       │       │
    ├─────────────────────┴───────┤
    │      bottom terminal bar    │
    └─────────────────────────────┘

- main panel (left): tabbed work area, tabs only in v1
- fs sidebar (right): yazi in a locked pane, follows pwd of active main tab
- bottom bar (full width): terminal drawer for shell/agents while editing above
- [+] dropdown on the RIGHT of the tab bar: New terminal / New crow-cli session
  / New file (hx)

## NEXT TURN — highest priority (after current phase is stable + user-tested)

- [ ] BUG (user report 29Aug2026): clicking a file in the sidebar yazi opens
      it INSIDE the yazi pane (yazi's own opener/preview). Wrong target. The
      click must open the file in the MAIN panel: spawn (or focus) a helix
      tab and load the file there. This is "the point" of the layout.
      Fix shape: yazi opener config (~/.config/yazi/yazi.toml `[opener]` /
      rules for text files) → shell out to crowterm's open-file path →
      spawn/focus hx tab in main panel with the file (Phase 4 socket or
      direct spawn `hx <path>` as a new tab via the [+] app machinery).
      Do NOT change the current PLAN until the dock phase is handed off green.

## Scope (unordered)

- [ ] Dock frame: main tab area | fs sidebar (right, locked, yazi) / bottom
      terminal bar. Toggle keybinds, sizes remembered.
- [ ] Explorer follows pwd of the active main tab.
- [ ] Main panel: tabs only, one pane per tab (splits deferred).
- [ ] Tab-bar [+] dropdown (right side) + config-driven app registry.
      Builtins: terminal ($SHELL), crow-cli session (bare `crow-cli`), new
      file (hx).
- [ ] Control socket: unix, JSON lines — list/spawn/focus/write/close/open-file.
      This replaces zellij's role in the zide picture.
- [ ] yazi opener glue: open file → focus-or-spawn hx tab, inject `:open <path>`.
      (zide reference clone: /tmp/zide — EDITOR-hijack + write-chars pattern,
      done natively via PTY write instead.)
- [ ] crow-cli TUI first-class: menu entry + keybind.
- [ ] SOP theme + DejaVu Sans Mono defaults shipped in-repo (fresh install
      looks right with zero user config).
- [ ] Rebrand to crowterm — LAST. User: "least important thing... build what
      we want out of it THEN worry about rebrand".

## Deferred (with reasons)

- Main-panel splits (left/right/up/down): user — "leave splitting... for
  exercise much later, for now we focus on tabs". Existing split actions stay
  functional, just not part of the dock model.
- Session persistence (restore tabs/layout on restart): after v1 works.
- Native file-tree widget instead of the yazi pane: maybe someday; yazi-in-pane
  is the philosophy.
- nvim/helix ACP client plugins: user — "potentially someday, but not today".
  We keep using crow_cli.tui's built-in ACP client.
- Shader-filter fun (hello-kitty-style pastel preset): user is down, after core.
- Serve crowterm on the WEB (user 29Aug2026): rio's wgpu backend is the
  on-ramp (sugarloaf has a wasm/web target upstream). Investigate building
  rioterm for wasm + a web shell, then polish via playwright-cli + the
  video-frames skill: record the rendered page, click-interact with the
  wgpu output, vision-judge the frames. No full MCP tooling needed.
- crow-cli TUI cancel lag: commit c739dec6 did NOT fix the felt lag (user,
  29Aug2026). Revisit during the full python-sdk ACP-ification of crow-cli.tui;
  tracked in crow-cli/TODO.md, not here.

## Grounded inventory (verified in source 29Aug2026)

- Tabs: `frontends/rioterm/src/renderer/island.rs` (~2000 lines). Actions:
  TabCreateNew, SelectTab(n), SelectLastTab, SelectNext/PrevTab,
  TabCloseCurrent, TabCloseUnfocused, MoveCurrentTabToPrev/Next.
- Splits: taffy 0.10 flexbox (`frontends/rioterm/Cargo.toml`). Actions:
  SplitRight, SplitDown, SelectNext/PrevSplit, SelectNext/PrevSplitOrTab,
  MoveDividerUp/Down/Left/Right, CloseCurrentSplitOrTab.
  `Screen::split_right_with_config()` (screen/mod.rs:1537) spawns a split with
  its own config/command — the seed of pane-command-spawn.
- Command palette: OpenCommandPalette (Super+Shift+P), fuzzy over actions
  (renderer/command_palette.rs).
- ToggleQuake — quake-style dropdown terminal already exists (ancestor of the
  bottom-bar toggle).
- Config hot-reload incl. bindings (screen/mod.rs update path); Run(Program)
  action; `rio -e cmd --working-dir`.
- Missing (our work): dock regions, + menu / app registry, ANY IPC/socket,
  open-file glue, pwd-follow, persistence.
