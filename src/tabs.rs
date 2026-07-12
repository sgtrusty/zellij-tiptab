use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use zellij_tile::prelude::*;

use crate::formatter;

pub struct TabState {
    pub tab_dirs: BTreeMap<u64, PathBuf>,
    pub tab_cmds: BTreeMap<u64, String>,
    pub term_cmds: BTreeMap<u32, String>,
    pub tab_ongoing: BTreeSet<u64>,
}

impl Default for TabState {
    fn default() -> Self {
        Self::new()
    }
}

impl TabState {
    pub fn new() -> Self {
        Self {
            tab_dirs: BTreeMap::new(),
            tab_cmds: BTreeMap::new(),
            term_cmds: BTreeMap::new(),
            tab_ongoing: BTreeSet::new(),
        }
    }

    pub fn ingest_pipe(&mut self, tab_id: u64, pwd: &str, bin: &str) {
        self.tab_dirs.insert(tab_id, PathBuf::from(pwd));
        let bin = formatter::fmt_bin(bin.trim());
        if bin.is_empty() || bin.starts_with("_tiptab_") {
            self.tab_cmds.remove(&tab_id);
            self.tab_ongoing.remove(&tab_id);
        } else {
            self.tab_cmds.insert(tab_id, bin);
            self.tab_ongoing.insert(tab_id);
        }
    }

    pub fn cleanup_tab(&mut self, tab_id: u64) {
        self.tab_dirs.remove(&tab_id);
        self.tab_cmds.remove(&tab_id);
        self.tab_ongoing.remove(&tab_id);
    }

    pub fn populate_last_cmds(&mut self, panes: &PaneManifest) {
        for panes in panes.panes.values() {
            for pane in panes {
                if pane.is_plugin || pane.is_suppressed {
                    continue;
                }
                if let Some(bin) = Self::binary_name(pane.terminal_command.as_deref().unwrap_or("")) {
                    self.term_cmds.insert(pane.id, formatter::fmt_bin(&bin));
                }
            }
        }
    }

    pub fn binary_name(terminal_command: &str) -> Option<String> {
        let command = terminal_command.trim();
        if command.is_empty() {
            return None;
        }
        let binary = command.split_whitespace().next()?;
        let name = Path::new(binary).file_name()?.to_str()?.to_string();
        if name.is_empty() || name.starts_with("_tiptab_") {
            None
        } else {
            Some(name)
        }
    }
}
