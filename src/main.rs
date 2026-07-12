use zellij_tile::prelude::*;
use zellij_tile::shim::get_session_list;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use tabs::TabState;
use parent_seek::ParentSeeker;

mod formatter;
mod parent_seek;
mod resolver;
mod tabs;
mod validation;
mod worker;

use worker::RENAME_WORKER;

#[derive(Default)]
struct State {
    userspace_configuration: BTreeMap<String, String>,
    permissions: Option<PermissionStatus>,
    tabs: Vec<TabInfo>,
    panes: PaneManifest,
    parser: TabState,
    seeker: ParentSeeker,
    session_name: Option<String>,
}

register_plugin!(State);
register_worker!(
    worker::RenameWorker,
    rename_worker,
    RENAME_WORKER_STATE
);

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.userspace_configuration = configuration;
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::RunCommands,
        ]);
        subscribe(&[
            EventType::TabUpdate,
            EventType::PaneUpdate,
            EventType::PaneClosed,
            EventType::CwdChanged,
            EventType::PermissionRequestResult,
            EventType::RunCommandResult,
            EventType::CustomMessage,
        ]);
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        match pipe_message.name.as_str() {
            "tiptab" => self.handle_pipe_tiptab(pipe_message),
            _ => false,
        }
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::TabUpdate(tabs) => {
                let old: BTreeSet<u64> =
                    self.tabs.iter().map(|t| t.tab_id as u64).collect();
                let new: BTreeSet<u64> =
                    tabs.iter().map(|t| t.tab_id as u64).collect();
                for created in new.difference(&old) {
                    validation::log(format!("tab created (tab_id={created})"));
                }
                for destroyed in old.difference(&new) {
                    validation::log(format!("tab destroyed (tab_id={destroyed})"));
                    self.parser.cleanup_tab(*destroyed);
                }
                self.tabs = tabs;
                self.organize_and_flush();
            }
            Event::PaneUpdate(panes) => {
                self.panes = panes;
                self.parser.populate_last_cmds(&self.panes);
                self.organize_and_flush();
            }
            Event::PaneClosed(_pane_id_enum) => {
                self.organize_and_flush();
            }
            Event::CwdChanged(_pane_id_enum, _cwd, _raw) => {
                self.organize_and_flush();
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
                    self.organize_and_flush();
                    return false;
                }

                let Ok(root) = String::from_utf8(stdout) else {
                    return false;
                };

                self.seeker.git_cache.insert(
                    PathBuf::from(path),
                    Some(PathBuf::from(root.trim())),
                );
                self.organize_and_flush();
            }
            Event::CustomMessage(name, payload) if name == "execute-renames" => {
                self.execute_renames(&payload);
            }
            _ => {}
        }
        false
    }

    fn render(&mut self, _rows: usize, _cols: usize) {}
}

impl State {
    fn handle_pipe_tiptab(&mut self, pipe_message: PipeMessage) -> bool {
        let Some(payload) = pipe_message.payload else {
            return false;
        };

        let (session, rest) = match payload.split_once('|') {
            Some((s, r)) => (Some(s.to_string()), r),
            None => (None, payload.as_str()),
        };
        if let (Some(own), Some(session)) = (self.current_session_name(), &session) {
            if own != *session {
                return false;
            }
        }

        let mut parts = rest.splitn(3, ' ');
        let (Some(pane_id_str), Some(pwd), Some(bin)) =
            (parts.next(), parts.next(), parts.next())
        else {
            return false;
        };

        let Ok(pane_id) = pane_id_str.parse::<u32>() else {
            return false;
        };

        let Some(tab_id) = self.find_tab_id_for_pane(pane_id) else {
            return false;
        };

        self.parser.ingest_pipe(tab_id, pwd, bin);
        self.organize_and_flush();
        false
    }

    fn find_tab_id_for_pane(&self, pane_id: u32) -> Option<u64> {
        for (tab_pos, panes) in &self.panes.panes {
            if panes.iter().any(|p| p.id == pane_id) {
                return self.tabs.iter()
                    .find(|t| t.position == *tab_pos)
                    .map(|t| t.tab_id as u64);
            }
        }
        None
    }

    fn current_session_name(&mut self) -> Option<String> {
        if self.session_name.is_none() {
            if let Ok(snapshot) = get_session_list() {
                for session in snapshot.live_sessions {
                    if session.is_current_session {
                        self.session_name = Some(session.name);
                        break;
                    }
                }
            }
        }
        self.session_name.clone()
    }

    fn execute_renames(&mut self, payload: &str) {
        let Ok(pending) = serde_json::from_str::<BTreeMap<u64, String>>(payload) else {
            return;
        };
        validation::log(format!("[plugin] execute-renames: {} tabs", pending.len()));
        for (tab_id, new_name) in pending {
            validation::log(format!("[plugin] renaming tab {tab_id} -> {new_name}"));
            rename_tab_with_id(tab_id, &new_name);
        }
    }

    fn organize_and_flush(&mut self) {
        let home_dir = self.userspace_configuration.get("home_dir").map(|s| s.as_str());
        let mut pending_renames = BTreeMap::new();
        resolver::organize(
            &self.tabs,
            &mut pending_renames,
            &self.parser,
            &self.seeker,
            &self.panes,
            self.permissions,
            home_dir,
        );
        if pending_renames.is_empty() {
            return;
        }
        for (tab_id, new_name) in &pending_renames {
            validation::log(format!("queue-rename tab_id={tab_id} new_name={new_name}"));
            post_message_to(PluginMessage {
                name: "queue-rename".to_string(),
                payload: serde_json::to_string(&(tab_id, new_name)).unwrap_or_default(),
                worker_name: Some(RENAME_WORKER.to_string()),
            });
        }
        post_message_to(PluginMessage {
            name: "execute-renames".to_string(),
            payload: String::new(),
            worker_name: Some(RENAME_WORKER.to_string()),
        });
    }
}
