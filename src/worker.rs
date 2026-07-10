use zellij_tile::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

pub const RENAME_WORKER: &str = "rename";

#[derive(Default, Serialize, Deserialize)]
pub struct RenameWorker {
    pending: BTreeMap<u64, String>,
}

impl ZellijWorker<'_> for RenameWorker {
    fn on_message(&mut self, message: String, payload: String) {
        match message.as_str() {
            "queue-rename" => {
                eprintln!("[worker] queue-rename payload={payload}");
                if let Ok((tab_id, new_name)) =
                    serde_json::from_str::<(u64, String)>(&payload)
                {
                    self.pending.insert(tab_id, new_name);
                    eprintln!("[worker] queued, {} pending", self.pending.len());
                } else {
                    eprintln!("[worker] failed to parse payload");
                }
            }
            "execute-renames" => {
                let pending = std::mem::take(&mut self.pending);
                eprintln!("[worker] execute-renames: {} tabs", pending.len());
                let payload = serde_json::to_string(&pending).unwrap_or_default();
                post_message_to_plugin(PluginMessage {
                    name: "execute-renames".to_string(),
                    payload,
                    worker_name: None,
                });
            }
            _ => {
                eprintln!("[worker] unknown message: {message}");
            }
        }
    }
}