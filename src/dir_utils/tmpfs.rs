use std::ffi::CString;
use std::fmt::format;
use std::fs;
use std::fs::Permissions;
use std::os::unix::prelude::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use errno::errno;
use libc::{mount, umount};
use crate::dir_utils::traits::BaseDir;

#[derive(Clone)]
pub struct TmpfsConfig {
    pub size: usize,
    pub inodes: usize,
    pub path: String,
    pub flag: u64,
    pub permissions: Permissions,
    pub read_only: bool
}

#[derive(Default, Clone)]
pub struct Tmpfs {
    pub config: TmpfsConfig,
    pub src: PathBuf,
    pub target: Option<PathBuf>,
    pub created: bool,
    pub mounted: bool
}

impl BaseDir for Tmpfs {
    fn create(&mut self) -> Result<(), String> {
        if self.created == true {
            return Ok(());
        }
        if !self.src.is_dir() {
            if fs::create_dir(&self.src).is_err() { return Err(errno().to_string()); }
            if fs::set_permissions(&self.src, self.config.permissions.clone()).is_err() {
                return Err(errno().to_string());
            }
        }
        let data = format!("size={}M,nr_inodes={}", self.config.size, self.config.inodes);
        unsafe {
            let result = mount(
                CString::new("tmpfs").unwrap().as_ptr(),
                CString::new(self.config.path.as_bytes()).unwrap().as_ptr(),
                CString::new("tmpfs").unwrap().as_ptr(),
                self.config.flag,
                CString::new(data.as_bytes()).unwrap().as_ptr().cast(),
            );
            if result == -1 {
                return Err(errno().to_string());
            }
        }
        self.created = true;
        Ok(())
    }

    fn clean(&mut self) -> Result<(), String> {
        if self.created == false {
            return Ok(());
        }
        unsafe {
            let result = umount(CString::new(self.config.path.as_bytes()).unwrap().as_ptr());
            if result == -1 {
                return Err(errno().to_string());
            }
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