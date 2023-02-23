use std::fs;
use std::fs::Permissions;
use std::path::PathBuf;
use errno::errno;

use crate::dir_utils::traits::BaseDir;

#[derive(Clone)]
pub struct LocalConfig {
    pub create_if_not_exist: bool,
    pub create_recursively: bool,
    pub clean: bool,
    pub path: String,
    pub permissions: Option<Permissions>,
    pub read_only: bool,
}

#[derive(Default, Clone)]
pub struct Local {
    pub config: LocalConfig,
    pub src: PathBuf,
    pub target: Option<PathBuf>,
    pub created: bool,
    pub mounted: bool,
}

impl BaseDir for Local {
    fn create(&mut self) -> Result<(), String> {
        if self.created == true {
            return Ok(());
        }
        if !self.src.is_dir() {
            if self.config.create_if_not_exist {
                if self.config.create_recursively {
                    if fs::create_dir_all(&self.src).is_err() { return Err(errno().to_string()); }
                } else {
                    if fs::create_dir(&self.src).is_err() { return Err(errno().to_string()); }
                }
            }
            match &self.config.permissions {
                Some(permissions) => {
                    if fs::set_permissions(&self.src, permissions.clone()).is_err() {
                        return Err(errno().to_string());
                    }
                },
                None => ()
            };
        }
        self.created = true;
        Ok(())
    }

    fn clean(&mut self) -> Result<(), String> {
        if self.created == false || self.config.clean == false {
            return Ok(());
        }
        match fs::remove_dir_all(&self.src) {
            Ok(_) => { self.created = false; Ok(()) },
            Err(e) => Err(e.to_string())
        }
    }

    fn get_src(&self) -> String {
        self.src.clone().to_str().unwrap().to_string()
    }

    fn read_only(&self) -> bool {
        self.config.read_only
    }

    fn get_bind_string(&mut self, target: &String) -> String  {
        self.target = Some(PathBuf::from(target));
        self.mounted = true;
        if self.read_only() {
            return format!("--bindmount_ro={}:{}", self.get_src(), target);
        }
        format!("--bindmount={}:{}", self.get_src(), target)
    }
}
