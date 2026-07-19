use std::collections::BTreeMap;
use std::path::Path;

use zellij_tile::prelude::*;

use crate::tabs::TabState;
use crate::formatter;
use crate::parent_seek::ParentSeeker;
use crate::validation::{self, log};

pub fn format_name(
    terminal_command: &str,
    cwd: &Path,
    seeker: &ParentSeeker,
    permissions: Option<PermissionStatus>,
    home_dir: Option<&str>,
) -> String {
    if let Some(name) = TabState::binary_name(terminal_command) {
        return name;
    }
    if let Some(name) = seeker.git_name(cwd, permissions) {
        return name;
    }
    ParentSeeker::folder_name(cwd, home_dir)
}

pub fn resolve_label_smart(
    tab: &TabInfo,
    parser: &TabState,
    seeker: &ParentSeeker,
    panes: &PaneManifest,
    permissions: Option<PermissionStatus>,
    home_dir: Option<&str>,
) -> String {
    log(format!("resolve_label_smart tab.position={} tab_cmds={:?}", tab.position, parser.tab_cmds));

    if let Some(bin) = parser.tab_cmds.get(&(tab.tab_id as u64)) {
        return format!("*{bin}");
    }
    if let Some(cwd) = parser.tab_dirs.get(&(tab.tab_id as u64)) {
        return format_name("", cwd, seeker, permissions, home_dir);
    }

    if let Some(panes) = panes.panes.get(&tab.position) {
        for pane in panes.iter().filter(|p| !p.is_plugin && !p.is_suppressed) {
            if let Some(bin) = parser.term_cmds.get(&pane.id) {
                if !bin.starts_with("_tiptab_") {
                    return bin.clone();
                }
            }
        }
    }

    "loading...".to_string()
}

pub fn organize(
    tabs: &[TabInfo],
    pending_renames: &mut BTreeMap<u64, String>,
    parser: &mut TabState,
    seeker: &ParentSeeker,
    panes: &PaneManifest,
    permissions: Option<PermissionStatus>,
    home_dir: Option<&str>,
) {
    for tab in tabs {
        if tab.position == 0 {
            continue;
        }

        let tab_id = tab.tab_id as u64;
        let label = resolve_label_smart(tab, parser, seeker, panes, permissions, home_dir);
        let name = formatter::fmt_label(tab.position as u32, &label);

        let managed = parser
            .applied_labels
            .get(&tab_id)
            .is_some_and(|applied| applied.trim() == tab.name.trim());
        let is_default = validation::is_default_tab_name(&tab.name);
        if !managed && !is_default {
            continue;
        }

        if parser.applied_labels.get(&tab_id).map(|a| a.trim()) != Some(name.trim()) {
            log(format!("rename tab_id={tab_id} {} -> {}", tab.name, name));
            pending_renames.insert(tab_id, name.clone());
            parser.applied_labels.insert(tab_id, name);
        }
    }
}
