use anyhow::Result;
use std::fs;

use crate::process;
use crate::types::{TerminalEmulator, TerminalTab, TerminalWindow};

pub fn detect() -> Result<Option<TerminalEmulator>> {
    // The kernel truncates comm to 15 chars: "gnome-terminal-"
    let pids = process::find_pids_by_name("gnome-terminal-");
    if pids.is_empty() {
        return Ok(None);
    }

    let server_pid = pids[0];
    let mut tabs = Vec::new();

    // Find child shell processes by scanning /proc for children of the server
    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let pid_str = entry.file_name();
            let pid_str = pid_str.to_string_lossy();
            let child_pid: u32 = match pid_str.parse() {
                Ok(p) => p,
                Err(_) => continue,
            };

            // Check if this process is a direct child of gnome-terminal-server
            let stat_path = format!("/proc/{}/stat", child_pid);
            let stat = match fs::read_to_string(&stat_path) {
                Ok(s) => s,
                Err(_) => continue,
            };

            // stat format: pid (comm) state ppid ...
            // Find ppid after the closing paren to handle comm with spaces
            let after_comm = match stat.rfind(')') {
                Some(pos) => &stat[pos + 2..],
                None => continue,
            };
            let fields: Vec<&str> = after_comm.split_whitespace().collect();
            // fields[0] = state, fields[1] = ppid
            let ppid: u32 = match fields.get(1).and_then(|s| s.parse().ok()) {
                Some(p) => p,
                None => continue,
            };

            if ppid != server_pid {
                continue;
            }

            // This is a direct child — get its TTY and shell info
            let comm = fs::read_to_string(format!("/proc/{}/comm", child_pid))
                .ok()
                .map(|s| s.trim().to_string());

            let tty = get_tty_for_pid(child_pid);
            let cwd = process::get_cwd(child_pid);

            let shell_name = comm.as_deref().map(|c| {
                let name = c.rsplit('/').next().unwrap_or(c);
                name.strip_prefix('-').unwrap_or(name).to_string()
            });

            tabs.push(TerminalTab {
                title: shell_name.clone().unwrap_or_default(),
                tty,
                shell_pid: Some(child_pid),
                shell: shell_name,
                cwd,
                columns: None,
                rows: None,
            });
        }
    }

    let windows = if tabs.is_empty() {
        vec![]
    } else {
        vec![TerminalWindow {
            id: server_pid.to_string(),
            tabs,
        }]
    };

    Ok(Some(TerminalEmulator {
        app: "GNOME Terminal".into(),
        pid: Some(server_pid),
        windows,
    }))
}

/// Read the TTY device path for a process from /proc/<pid>/fd/0.
fn get_tty_for_pid(pid: u32) -> Option<String> {
    fs::read_link(format!("/proc/{}/fd/0", pid))
        .ok()
        .and_then(|p| p.into_os_string().into_string().ok())
        .filter(|s| s.starts_with("/dev/pts/"))
}
