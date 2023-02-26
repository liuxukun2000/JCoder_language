use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use crate::dir_utils::traits::BaseDir;
use std::process::{Command};
use std::path::Path;
use random_string::generate;
use crate::nsjail_judger::result::JudgerResult;

#[derive(Deserialize, Serialize, Clone)]
pub struct NsjailTask {
    pub config: HashMap<String, String>,
    pub meta_data: HashMap<String, String>,
    pub mount: Vec<String>,
    pub exec: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub envs: HashMap<String, String>
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct NsjailConfig {
    pub rootfs_path: String,
    pub task_config: Vec<NsjailTask>,
}

pub struct NsjailJudger {
    pub config: NsjailConfig,
    pub dirs: Vec<Arc<Mutex<dyn BaseDir>>>,
    pub user_dir: i32,
    pub output_dir: i32,
    pub cursor: usize,
    pub prefix: String
}

impl NsjailJudger {
    pub fn new(config: &NsjailConfig) -> Self {
        NsjailJudger {
            config: config.clone(),
            dirs: vec![],
            user_dir: -1,
            output_dir: -1,
            cursor: 0,
            prefix: generate(6, "abcdefghigklmnopqrstuvwxyz")
        }
    }
    pub fn mount_all(&mut self, dir: Arc<Mutex<dyn BaseDir>>, target: &String) {
        self.dirs.push(dir.clone());
        self.config.task_config.iter_mut().for_each(|task| {
            task.mount.push(dir.lock().unwrap().get_bind_string(target));
        });
    }

    pub fn convert_task(dir: &String, task: &mut NsjailTask, prefix: &String, cursor: usize) -> (String, String) {
        let mut rep_path = "".to_string();
        let mut out_path = "".to_string();
        match task.config.get("log") {
            Some(x) => {
                task.config.insert("log".to_string(), format!("{}/{}", dir, x));
            },
            None => {
                task.config.insert("log".to_string(), format!("{}/{}.log", dir, prefix));
            }
        };
        match task.config.get("stdout") {
            Some(x) => {
                out_path = format!("{}/{}", dir, x);
                task.config.insert("stdout".to_string(), out_path.clone());
            },
            None => {
                out_path = format!("{}/{}_{}.out", dir, prefix, cursor);
                task.config.insert("stdout".to_string(), out_path.clone());
            }
        };
        match task.config.get("report") {
            Some(x) => {
                rep_path = format!("{}/{}", dir, x);
                task.config.insert("report".to_string(), rep_path.clone());
            },
            None => {
                rep_path = format!("{}/{}_{}.rep", dir, prefix, cursor);
                task.config.insert("report".to_string(), rep_path.clone());
            }
        };
        (out_path, rep_path)
    }

    pub fn run(&mut self, cursor: i32) -> JudgerResult {
        let mut task = &mut self.config.task_config[cursor as usize];
        let dir = ".".to_string();
        let (out_path, rep_path) = NsjailJudger::convert_task(&dir, task, &self.prefix, self.cursor);

        let result = Command::new("./nsjail")
            .args(&task.to_args(&self.config.rootfs_path))
            .status();
        if result.is_ok() {
            return JudgerResult::from_file(&rep_path, &out_path);
        }
        JudgerResult::default()
    }

    #[cfg(online)]
    pub fn run(&mut self, cursor: i32) -> JudgerResult {
        let mut task = &mut self.config.task_config[cursor as usize];
        let out = self.output_dir;
        if out < 0 {
            return JudgerResult::default();
        }
        let dir = self.dirs[out as usize].lock().unwrap().get_src();
        let (out_path, rep_path) = NsjailJudger::convert_task(&dir, task, &self.prefix, self.cursor);
        let result = Command::new("./nsjail")
            .args(&task.to_args(&self.config.rootfs_path))
            .status();
        if result.is_ok() {
            return JudgerResult::from_file(&rep_path, &out_path);
        }
        JudgerResult::default()
    }

    pub fn run_step(&mut self) -> JudgerResult {
        let result = self.run(self.cursor as i32);
        self.cursor += 1;
        result
    }

    pub fn run_all(&mut self) -> Vec<JudgerResult> {
        let mut ret = vec![];
        self.cursor = 0;
        while self.cursor < self.config.task_config.len() {
            let tmp = self.run_step();
            if tmp.exit_code != 0 || tmp.exit_signal != 0 {
                ret.push(tmp);
                break;
            }
            ret.push(tmp);
        }
        ret
    }

}