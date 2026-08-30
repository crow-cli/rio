//! crowterm dock — the window-level IDE frame around the tab strip.
//!
//! The dock reserves two locked regions around the main tab area: a
//! filesystem sidebar on the RIGHT (yazi) and a terminal bar across the
//! bottom ($SHELL). Each region is a real PTY grid (`GridKind::Sidebar`
//! / `GridKind::BottomBar`) living at the tail of the ContextManager —
//! "the protocol is the terminal": the dock never embeds widgets, it
//! only decides visibility, size, and the margins that place each grid.

use crate::layout::GridKind;

/// Default sidebar width as a fraction of window width.
pub const DEFAULT_SIDEBAR_FRAC: f32 = 0.28;
/// Default bottom bar height as a fraction of window height.
pub const DEFAULT_BOTTOM_FRAC: f32 = 0.25;
/// Dock panes smaller than this are useless; clamp up.
pub const MIN_DOCK_PX: f32 = 80.0;

/// Program the filesystem sidebar runs.
pub const SIDEBAR_PROGRAM: &str = "yazi";

#[derive(Debug, Clone)]
pub struct DockState {
    pub sidebar_visible: bool,
    pub bottom_visible: bool,
    /// Sidebar width as a fraction of window width.
    pub sidebar_frac: f32,
    /// Bottom bar height as a fraction of window height.
    pub bottom_frac: f32,
}

impl Default for DockState {
    fn default() -> Self {
        Self {
            // crowterm IS the IDE frame: dock panes start visible. If a
            // pane's program fails to spawn, Screen flips its flag off.
            sidebar_visible: true,
            bottom_visible: true,
            sidebar_frac: DEFAULT_SIDEBAR_FRAC,
            bottom_frac: DEFAULT_BOTTOM_FRAC,
        }
    }
}

impl DockState {
    /// Sidebar width in physical px for a window `width` wide.
    #[inline]
    pub fn sidebar_width(&self, width: f32) -> f32 {
        if self.sidebar_visible {
            (width * self.sidebar_frac).max(MIN_DOCK_PX)
        } else {
            0.0
        }
    }

    /// Bottom bar height in physical px for a window `height` tall.
    #[inline]
    pub fn bottom_height(&self, height: f32) -> f32 {
        if self.bottom_visible {
            (height * self.bottom_frac).max(MIN_DOCK_PX)
        } else {
            0.0
        }
    }

    #[inline]
    pub fn grid_is_expected(&self, kind: GridKind) -> bool {
        match kind {
            GridKind::Sidebar => self.sidebar_visible,
            GridKind::BottomBar => self.bottom_visible,
            GridKind::Tab => true,
        }
    }
}

/// crowterm gives the sidebar yazi a stable per-window DDS client id so
/// focus changes can drive it with `ya emit-to <id> cd <cwd>`. yazi
/// requires a NUMERIC id ("globally unique number"), so mix the pid
/// (unique per rio process) with the window id.
pub fn yazi_client_id(window_id: u64) -> String {
    let id = ((std::process::id() as u64) << 32) | (window_id & 0xFFFF_FFFF);
    id.to_string()
}

/// Resolve the `ya` CLI (yazi's DDS companion). PATH first, then the
/// usual `~/.cargo/bin` home — rio launched from a desktop shortcut may
/// not carry the shell's PATH.
pub fn resolve_ya() -> Option<std::path::PathBuf> {
    let name = "ya";
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    let home = std::env::var_os("HOME")?;
    let candidate = std::path::PathBuf::from(home).join(".cargo/bin").join(name);
    candidate.is_file().then_some(candidate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yazi_client_id_is_numeric_and_unique_per_window() {
        // yazi rejects non-numeric --client-id values ("must be a
        // globally unique number"), so the id must parse as u64.
        let a = yazi_client_id(1);
        let b = yazi_client_id(2);
        assert!(a.chars().all(|c| c.is_ascii_digit()), "{a}");
        assert!(b.chars().all(|c| c.is_ascii_digit()), "{b}");
        assert_ne!(a, b);
        assert_eq!(a.parse::<u64>().unwrap() & 0xFFFF_FFFF, 1);
    }

    #[test]
    fn sidebar_width_and_bottom_height_collapse_when_hidden() {
        let mut dock = DockState::default();
        assert!(dock.sidebar_width(1000.0) > 0.0);
        assert!(dock.bottom_height(1000.0) > 0.0);
        dock.sidebar_visible = false;
        dock.bottom_visible = false;
        assert_eq!(dock.sidebar_width(1000.0), 0.0);
        assert_eq!(dock.bottom_height(1000.0), 0.0);
        assert!(!dock.grid_is_expected(GridKind::Sidebar));
        assert!(!dock.grid_is_expected(GridKind::BottomBar));
        assert!(dock.grid_is_expected(GridKind::Tab));
    }

    #[test]
    fn dock_panes_clamp_up_to_minimum_size() {
        let dock = DockState::default();
        // A tiny window would give a sub-80px pane; clamp up.
        assert_eq!(dock.sidebar_width(100.0), MIN_DOCK_PX);
        assert_eq!(dock.bottom_height(100.0), MIN_DOCK_PX);
    }
}
