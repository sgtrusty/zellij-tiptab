use zellij_tile::prelude::*;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use bin_parser::BinParser;
use parent_seek::ParentSeeker;

mod bin_parser;
mod formatter;
mod parent_seek;
mod resolver;
mod validation;

#[derive(Default)]
struct State {
    userspace_configuration: BTreeMap<String, String>,
    permissions: Option<PermissionStatus>,
    tabs: Vec<TabInfo>,
    panes: PaneManifest,
    parser: BinParser,
    seeker: ParentSeeker,
    pending_renames: BTreeMap<u64, String>,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.userspace_configuration = configuration;
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::RunCommands,
        ]);
        subscribe(&[
            EventType::Timer,
            EventType::TabUpdate,
            EventType::PaneUpdate,
            EventType::PaneClosed,
            EventType::CwdChanged,
            EventType::PermissionRequestResult,
            EventType::RunCommandResult,
        ]);
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        if pipe_message.name != "tiptab" {
            return false;
        }

        let Some(payload) = pipe_message.payload else {
            return false;
        };

        let mut parts = payload.splitn(3, ' ');
        let (Some(tab_pos_str), Some(pwd), Some(bin)) =
            (parts.next(), parts.next(), parts.next())
        else {
            return false;
        };

        let Ok(tab_pos) = tab_pos_str.parse::<u32>() else {
            return false;
        };

        self.parser.ingest_pipe(tab_pos, pwd, bin);
        let home_dir = self.userspace_configuration.get("home_dir").map(|s| s.as_str());
        resolver::organize(
            &self.tabs,
            &mut self.pending_renames,
            &self.parser,
            &self.seeker,
            &self.panes,
            self.permissions,
            home_dir,
        );
        false
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::Timer(_timer_id) => {
                let pending = std::mem::take(&mut self.pending_renames);
                for (tab_id, new_name) in pending {
                    rename_tab_with_id(tab_id, &new_name);
                }
            }
            Event::TabUpdate(tabs) => {
                let old: BTreeSet<u64> =
                    self.tabs.iter().map(|t| t.tab_id as u64).collect();
                let new: BTreeSet<u64> =
                    tabs.iter().map(|t| t.tab_id as u64).collect();
                for created in new.difference(&old) {
                    validation::log(format!("tab created (tab_id={created})"));
                }
                self.tabs = tabs;
                let home_dir = self.userspace_configuration.get("home_dir").map(|s| s.as_str());
                resolver::organize(
                    &self.tabs,
                    &mut self.pending_renames,
                    &self.parser,
                    &self.seeker,
                    &self.panes,
                    self.permissions,
                    home_dir,
                );
            }
            Event::PaneUpdate(panes) => {
                self.panes = panes;
                self.parser.populate_last_cmds(&self.panes);
                let home_dir = self.userspace_configuration.get("home_dir").map(|s| s.as_str());
                resolver::organize(
                    &self.tabs,
                    &mut self.pending_renames,
                    &self.parser,
                    &self.seeker,
                    &self.panes,
                    self.permissions,
                    home_dir,
                );
            }
            Event::PaneClosed(_pane_id_enum) => {
                let home_dir = self.userspace_configuration.get("home_dir").map(|s| s.as_str());
                resolver::organize(
                    &self.tabs,
                    &mut self.pending_renames,
                    &self.parser,
                    &self.seeker,
                    &self.panes,
                    self.permissions,
                    home_dir,
                );
            }
            Event::CwdChanged(_pane_id_enum, _cwd, _raw) => {
                let home_dir = self.userspace_configuration.get("home_dir").map(|s| s.as_str());
                resolver::organize(
                    &self.tabs,
                    &mut self.pending_renames,
                    &self.parser,
                    &self.seeker,
                    &self.panes,
                    self.permissions,
                    home_dir,
                );
            }
            Event::PermissionRequestResult(status) => self.permissions = Some(status),
            Event::RunCommandResult(exit_code, stdout, _stderr, context) => {
                if context.get("plugin") != Some(&String::from("tiptab"))
                    || context.get("fn") != Some(&String::from("get_git_ws"))
                {
                    return false;
                }

                let Some(path) = context.get("path") else {
                    return false;
                };

                if exit_code != Some(0) {
                    self.seeker
                        .git_cache
                        .insert(PathBuf::from(path), None);
                    let home_dir = self.userspace_configuration.get("home_dir").map(|s| s.as_str());
                    resolver::organize(
                        &self.tabs,
                        &mut self.pending_renames,
                        &self.parser,
                        &self.seeker,
                        &self.panes,
                        self.permissions,
                        home_dir,
                    );
                    return false;
                }

                let Ok(root) = String::from_utf8(stdout) else {
                    return false;
                };

                self.seeker.git_cache.insert(
                    PathBuf::from(path),
                    Some(PathBuf::from(root.trim())),
                );
                let home_dir = self.userspace_configuration.get("home_dir").map(|s| s.as_str());
                resolver::organize(
                    &self.tabs,
                    &mut self.pending_renames,
                    &self.parser,
                    &self.seeker,
                    &self.panes,
                    self.permissions,
                    home_dir,
                );
            }
            _ => {}
        }
        false
    }

    fn render(&mut self, _rows: usize, _cols: usize) {}
}
