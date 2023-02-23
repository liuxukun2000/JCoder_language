#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;


use core::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use difference::{Changeset, Difference};
use fs_extra::{copy_items, dir};
use fs_extra::dir::{copy, CopyOptions};
use crate::compare::compare::{compare_string, CompareConfig, remove_end_of_text_enters, compare_file};
use crate::compare::results::{BaseResult, BaseStatus};
use crate::dir_utils::local::{Local, LocalConfig};
use crate::dir_utils::tmpfs::{Tmpfs, TmpfsConfig};
use crate::dir_utils::traits::BaseDir;
use crate::nsjail_judger::judger::{NsjailConfig, NsjailJudger, NsjailTask};

lazy_static! {
    static ref USEDDIRS: Mutex<Vec<Arc<Mutex<dyn BaseDir + Send + Sync>>>> = Mutex::new(Vec::new());
    static ref LOGS: Mutex<Vec<String>> = Mutex::new(Vec::new());
    static ref ERRORS: Mutex<Vec<String>> = Mutex::new(Vec::new());
    static ref STATUS: Mutex<BaseStatus> = Mutex::new(BaseStatus::PENDING);
    static ref COMEPARERESULT: Mutex<Vec<Arc<BaseResult>>> = Mutex::new(Vec::new());
}

mod dir_utils;
mod compare;
mod nsjail_judger;


macro_rules! Eval {
    (clean) => {
        for dir in USEDDIRS.lock().unwrap().iter_mut() {
            let _tmp = dir.deref().lock().unwrap().get_src();
            match dir.lock().unwrap().clean() {
                Ok(_) => LOGS.lock().unwrap().push(
                    format!("Successfully Deleted Directory: {}", &_tmp)
                ),
                Err(e) => ERRORS.lock().unwrap().push(e.to_string()),
            };
        }
    };
    (use tmpfs $path: expr, as $name: ident) => {
        Eval!(create tmpfs with TmpfsConfig::new($path.to_string()), as $name);
    };
    (use localfs $path: expr, as $name: ident) => {
        Eval!(create localfs with LocalConfig::new($path.to_string()), as $name);
    };
    (create tmpfs with $args: expr, as $name: ident) => {
        let mut $name = Arc::new(Mutex::new(Tmpfs::from(&$args)));
        let _tmp = $name.lock().unwrap().get_src();
        match $name.lock().unwrap().create() {
            Ok(_) => {
                USEDDIRS.lock().unwrap().push($name.clone());
                LOGS.lock().unwrap().push(
                    format!("Successfully Created Directory: {}", &_tmp)
                );
            },
            Err(e) => ERRORS.lock().unwrap().push(e.to_string()),
        };
    };
    (create localfs with $args: expr, as $name: ident) => {
        let mut $name = Arc::new(Mutex::new(Local::from(&$args)));
        let _tmp = $name.lock().unwrap().get_src();
        match $name.lock().unwrap().create() {
            Ok(_) => {
                USEDDIRS.lock().unwrap().push($name.clone());
                LOGS.lock().unwrap().push(
                    format!("Successfully Created Directory: {}", &_tmp)
                );
            },
            Err(e) => ERRORS.lock().unwrap().push(e.to_string()),
        };
    };
    (copydir $from: ident to $to: ident, with $config: ident) => {
        let f = $from.lock().unwrap().get_src();
        let t = $to.lock().unwrap().get_src();
        match copy(&f, &t, &$config) {
          Ok(num) => {
              LOGS.lock().unwrap().push(
                    format!(
                        "Successfully Copied {} Items From {} to {}",
                        num, &f, &t
                    )
                );
          },
            Err(e) => {
              ERRORS.lock().unwrap().push(e.to_string());
          }
        };
    };
    (copydir $from: ident to $to: ident) => {
        let _config = CopyOptions {
                    overwrite: true,
                    skip_exist: false,
                    buffer_size: 0,
                    copy_inside: false,
                    content_only: true,
                    depth: 0,
                };
        Eval!(copydir $from to $to, with _config);
    };
    (compare string $ans: ident to $output: ident, with $args: ident, as $name: ident) => {
        let mut $name = compare_string($ans, $output, &$args);
        LOGS.lock().unwrap().push(
            format!(
                "Successfully Compared String {} to {}",
                $ans, $output
            )
        );
    };
    (compare string $ans: ident to $output: ident, as $name: ident) => {
        let mut $name = compare_string($ans, $output, &CompareConfig::default());
        LOGS.lock().unwrap().push(
            format!(
                "Successfully Compared String {} to {}",
                $ans, $output
            )
        );
    };
    (compare file $ans: ident to $output: ident, with $args: ident, as $name: ident) => {
        let mut $name = compare_file(&$ans, &$output, &$args);
        if let Err(ref e) = $name {
            ERRORS.lock().unwrap().push(e.clone());
            *STATUS.lock().unwrap() = BaseStatus::UKE;
        } else {
            LOGS.lock().unwrap().push(
                format!(
                    "Successfully Compared File {} to {}",
                    &$ans, &$output
                )
            );
        }
    };
    (compare file $ans: ident to $output: ident, as $name: ident) => {
        let mut $name = compare_file(&$ans, &$output, &CompareConfig::default());
        if let Err(ref e) = $name {
            ERRORS.lock().unwrap().push(e.clone());
            *STATUS.lock().unwrap() = BaseStatus::UKE;
        } else {
            LOGS.lock().unwrap().push(
                format!(
                    "Successfully Compared File {} to {}",
                    &$ans, &$output
                )
            );
        }
    };
    (update result $result: ident by changeset {$changeset: expr}) => {
        $result.update(&$changeset);
        *STATUS.lock().unwrap() = $result.status;
    };
    (push result $result: ident) => {
        COMEPARERESULT.lock().unwrap().push(Arc::new($result));
    };
    (update all;) => {

    };
    (use nsjail with $args: expr, as $name: ident) => {
        let mut $name = NsjailJudger::new(&$args);
    };
    (mount $fs: ident to $judger: ident at $target: literal) => {
        $judger.mount_all($fs, &$target.to_string());
    };
    (run all tasks in $judger: ident) => {
        $judger.run_all();
    };
    (run tasks $index: literal in $judger: ident) => {
        $judger.run($index);
    };
}

fn main() {

    let mut config = NsjailConfig::default();
    let mut task = NsjailTask::default();
    // task.config.insert("time_limit".to_string(), "1000".to_string());
    // task.config.insert("cgroup_mem_max".to_string(), "902400".to_string());
    // task.config.insert("log".to_string(), "test.log".to_string());
    // task.config.insert("stdout".to_string(), "test.out".to_string());
    // task.config.insert("report".to_string(), "test.rep".to_string());
    task.exec = "/bin/ls".to_string();
    task.args = vec!["/".to_string()];

    let mut task1 = NsjailTask::default();
    // task1.config.insert("time_limit".to_string(), "1000".to_string());
    // task1.config.insert("cgroup_mem_max".to_string(), "902400".to_string());
    // task1.config.insert("log".to_string(), "test.log".to_string());
    // task1.config.insert("stdout".to_string(), "test.out".to_string());
    // task1.config.insert("report".to_string(), "test.rep".to_string());
    task1.exec = "/bin/ls".to_string();
    task1.args = vec!["/".to_string()];

    for i in 0..1 {
        config.task_config.push(task.clone());
    }
    config.task_config.push(task1.clone());
    config.rootfs_path = "/home/satan/language/rootfs".to_string();
    // println!("{:?}", &task.to_args(&"/".to_string()));

    Eval!(use nsjail with config, as jail);
    // Eval!(use tmpfs "/tmp/test1", as x);
    // Eval!(use localfs "/home/satan/language/test", as y);
    // Eval!(copydir y to x);
    // Eval!(mount x to jail at "/test");
    Eval!(run all tasks in jail);
    // Eval!(run all tasks in jail);
    // Eval!(use localfs "/home/satan/language/test", as y;);
    // Eval!(copydir y to x, with options;);
    Eval!(clean);
    println!("{:?}", LOGS.lock().unwrap());
    println!("{:?}", ERRORS.lock().unwrap());
}

