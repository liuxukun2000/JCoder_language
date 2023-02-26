use std::collections::HashMap;
use crate::nsjail_judger::judger::NsjailTask;

impl NsjailTask {
    pub fn to_args(&self, rootfs: &String) -> Vec<String> {
        let mut result = vec![
            "-Mo".to_string(),
            "--chroot".to_string(),
            rootfs.clone().to_string(),
        ];
        for (key, value) in &self.config {
            result.push(format!("--{}", key));
            result.push(value.clone());
        }

        for (key, value) in &self.envs {
            result.push("--env".to_string());
            result.push(format!("{}={}", &key, &value));
        }

        for mount in &self.mount {
            result.push(mount.clone());
        }

        result.push("--cwd".to_string());
        result.push(self.cwd.clone());


        result.push("--".to_string());
        result.push(self.exec.clone());

        for arg in &self.args {
            result.push(arg.clone());
        }
        result
    }
}

impl Default for NsjailTask {
    fn default() -> Self {
        Self {
            config: Default::default(),
            meta_data: Default::default(),
            mount: vec![],
            exec: "".to_string(),
            args: vec![],
            cwd: "/".to_string(),
            envs: HashMap::from([("PATH".to_string(), "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string())]),
        }
    }
}