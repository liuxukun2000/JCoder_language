use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use libc::{c_char, c_int, dlopen, dlerror, dlsym, dlclose, CS};
use serde::{Deserialize, Serialize};
use crate::dir_utils::traits::BaseDir;
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

type MainFn = unsafe extern "C" fn(argc: c_int, argv: *const *const c_char) -> c_int;

lazy_static!{
    static ref NSJAIL: MainFn = {
        let lib_path = Path::new("./libnsjail.so");
        let lib_name = CString::new(lib_path.as_os_str().as_bytes()).unwrap();
        let lib_handle = unsafe { dlopen(lib_name.as_ptr(), libc::RTLD_NOW|libc::RTLD_LOCAL) };
        if lib_handle.is_null() {
            let err = unsafe { dlerror() };
            let msg = unsafe { CString::from_raw(err as *mut c_char) };
            panic!("Failed to load shared library: {}", msg.to_string_lossy());
        }

        // Find the main function
        let main_name = CString::new("main").unwrap();
        let main_func = unsafe { dlsym(lib_handle, main_name.as_ptr()) };
        if main_func.is_null() {
            let err = unsafe { dlerror() };
            let msg = unsafe { CString::from_raw(err as *mut c_char) };
            unsafe { dlclose(lib_handle) };
            panic!("Failed to find main function: {}", msg.to_string_lossy());
        }

        // Call the main function
        let main_fn: MainFn = unsafe { std::mem::transmute(main_func) };
        main_fn
    };
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct NsjailTask {
    pub config: HashMap<String, String>,
    pub meta_data: HashMap<String, String>,
    pub mount: Vec<String>,
    pub exec: String,
    pub args: Vec<String>,
}

impl NsjailTask {
    pub fn to_args(&self, rootfs: &String) -> Vec<CString> {
        let mut result = vec![
            CString::new("nsjail").unwrap(),
            CString::new("-Mo").unwrap(),
            CString::new("--chroot").unwrap(),
            CString::new(rootfs.clone()).unwrap(),
        ];
        for (key, value) in &self.config {
            result.push(CString::new(format!("--{}", key)).unwrap());
            result.push(CString::new(value.clone()).unwrap());
        }

        for mount in &self.mount {
            result.push(CString::new(mount.clone()).unwrap());
        }

        result.push(CString::new("--").unwrap());
        result.push(CString::new(self.exec.clone()).unwrap());
        for arg in &self.args {
            result.push(CString::new(arg.clone()).unwrap());
        }
        result
    }
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct NsjailConfig {
    pub rootfs_path: String,
    pub task_config: Vec<NsjailTask>,
}

pub struct NsjailJudger {
    pub config: NsjailConfig,
    pub dirs: Vec<Arc<Mutex<dyn BaseDir>>>,
    pub cursor: usize
}

impl NsjailJudger {
    pub fn new(config: &NsjailConfig) -> Self {
        NsjailJudger {
            config: config.clone(),
            dirs: vec![],
            cursor: 0
        }
    }
    pub fn mount_all(&mut self, dir: Arc<Mutex<dyn BaseDir>>, target: &String) {
        self.dirs.push(dir.clone());
        self.config.task_config.iter_mut().for_each(|task| {
            task.mount.push(dir.lock().unwrap().get_bind_string(target));
        });
    }

    pub fn run(&mut self, cursor: i32) -> i32 {
        let argv = self.config.task_config[cursor as usize].to_args(&self.config.rootfs_path);
        let argc = argv.len() as c_int;
        let args = argv.iter().map(|x| x.as_c_str().as_ptr()).collect::<Vec<_>>();
        let ret = unsafe { NSJAIL(argc, args.as_ptr()) };
        ret as i32
    }

    pub fn run_step(&mut self) -> i32 {
        let argv = self.config.task_config[self.cursor].to_args(&self.config.rootfs_path);
        println!("{:?}", &argv);
        let argc = argv.len() as c_int;
        let mut args = argv.iter().map(|x| x.as_c_str().as_ptr()).collect::<Vec<_>>();
        args.push(std::ptr::null());
        let ret = unsafe { NSJAIL(argc, args.as_ptr()) };
        std::mem::forget(argc);
        std::mem::forget(argv);
        std::mem::forget(args);
        self.cursor += 1;
        ret as i32
    }

    pub fn run_all(&mut self) -> i32 {
        let mut ret = 0;
        self.cursor = 0;
        while self.cursor < self.config.task_config.len() {
            ret = self.run_step();
            if ret != 0 {
                break;
            }
        }
        ret
    }

}