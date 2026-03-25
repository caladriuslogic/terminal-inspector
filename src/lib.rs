//! Inspect running terminal emulators, multiplexer sessions, and browsers on macOS.
//!
//! # Example
//!
//! ```no_run
//! use workspace_inspector::{inspect_all, locate};
//!
//! // Get a canonical URI for the current terminal location
//! let uri = locate().unwrap();
//! println!("{}", uri); // terminal://iterm2/window:1229/tab:3/tmux:main/window:1/pane:0
//!
//! // Inspect all running terminals and multiplexers
//! let output = inspect_all().unwrap();
//! for term in &output.terminals {
//!     println!("{} (pid {:?})", term.app, term.pid);
//! }
//! ```

mod locate;
mod process;
mod shelldon;
pub mod terminals;
pub mod tmux;
mod types;
pub mod zellij;

pub use locate::locate;
pub use types::*;

use anyhow::Result;

/// Inspect all running terminals and multiplexer sessions.
pub fn inspect_all() -> Result<InspectorOutput> {
    Ok(InspectorOutput {
        terminals: terminals::detect_all()?,
        tmux: tmux::detect()?,
        shelldon: shelldon::detect()?,
        zellij: zellij::detect()?,
    })
}

/// Inspect only running terminal emulators.
pub fn inspect_terminals() -> Result<Vec<TerminalEmulator>> {
    terminals::detect_all()
}

/// Inspect only tmux sessions.
pub fn inspect_tmux() -> Result<Vec<TmuxSession>> {
    tmux::detect()
}

/// Inspect only shelldon instances.
pub fn inspect_shelldon() -> Result<Vec<ShelldonInstance>> {
    shelldon::detect()
}

/// Inspect only zellij sessions.
pub fn inspect_zellij() -> Result<Vec<ZellijSession>> {
    zellij::detect()
}
