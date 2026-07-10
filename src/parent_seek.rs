use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use zellij_tile::prelude::*;

use crate::formatter;

pub struct ParentSeeker {
    pub git_cache: BTreeMap<PathBuf, Option<PathBuf>>,
}

impl Default for ParentSeeker {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentSeeker {
    pub fn new() -> Self {
        Self {
            git_cache: BTreeMap::new(),
        }
    }

    pub fn get_git_ws(
        &self,
        path: &Path,
        permissions: Option<PermissionStatus>,
    ) -> Option<PathBuf> {
        if let Some(metadata) = self.git_cache.get(path) {
            return metadata.clone();
        }

        if let Some(PermissionStatus::Granted) = permissions {
            let mut context = BTreeMap::new();
            context.insert(String::from("plugin"), String::from("tiptab"));
            context.insert(String::from("fn"), String::from("get_git_ws"));
            context.insert(String::from("path"), path.to_string_lossy().to_string());
            run_command_with_env_variables_and_cwd(
                &["git", "rev-parse", "--show-toplevel"],
                BTreeMap::new(),
                path.to_path_buf(),
                context,
            );
        }

        None
    }

    pub fn git_name(&self, cwd: &Path, permissions: Option<PermissionStatus>) -> Option<String> {
        let root = self.get_git_ws(cwd, permissions)?;
        let name = formatter::fmt_git(cwd, &root);
        if name.is_empty() { None } else { Some(name) }
    }

    pub fn folder_name(cwd: &Path, home_dir: Option<&str>) -> String {
        if let Some(home) = home_dir {
            let home = home.trim_end_matches('/');
            if cwd.as_os_str() == home {
                return "~".to_string();
            }
        }
        formatter::fmt_folder(cwd)
    }
}
