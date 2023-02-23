use std::fs::Permissions;
use std::os::unix::prelude::PermissionsExt;
use std::path::PathBuf;
use crate::dir_utils::local::{Local, LocalConfig};
use crate::dir_utils::tmpfs::{Tmpfs, TmpfsConfig};

pub trait BaseDir {
    fn create(&mut self) -> Result<(), String>;
    fn clean(&mut self) -> Result<(), String>;
    fn get_src(&self) -> String;
    fn read_only(&self) -> bool;
    fn get_bind_string(&mut self, target: &String) -> String;
}

impl Default for TmpfsConfig {
    fn default() -> Self {
        Self {
            size: 64,
            inodes: 1024,
            path: String::default(),
            flag: 0,
            permissions: Permissions::from_mode(777),
            read_only: false,
        }
    }
}

impl TmpfsConfig {
    pub fn new(path: String) -> Self {
        Self {
            path,
            ..Self::default()
        }
    }
}

impl From<&TmpfsConfig> for Tmpfs {
    fn from(config: &TmpfsConfig) -> Self {
        Self {
            config: config.clone(),
            src: PathBuf::from(&config.path),
            target: None,
            created: false,
            mounted: false,
        }
    }
}

impl From<&LocalConfig> for Local {
    fn from(config: &LocalConfig) -> Self {
        Self {
            config: config.clone(),
            src: PathBuf::from(&config.path),
            target: None,
            created: false,
            mounted: false,
        }
    }
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            create_if_not_exist: false,
            create_recursively: true,
            clean: false,
            path: "".to_string(),
            permissions: None,
            read_only: true,
        }
    }
}

impl LocalConfig {
    pub fn new(path: String) -> Self {
        Self {
            path,
            ..Self::default()
        }
    }
}