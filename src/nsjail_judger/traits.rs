// use std::collections::HashMap;
// use toml::Table;
// use crate::nsjail_judger::judger::NsjailConfig;
//
// impl NsjailConfig {
//     pub fn new(chroot_path: &String) -> NsjailConfig {
//         NsjailConfig {
//             rootfs_path: chroot_path.clone(),
//             task_config: vec![],
//         }
//     }
//
//     fn from_string(config: &String) -> Result<Self, String> {
//         let mut config = config.parse::<Table>().unwrap();
//         let tasks = config["tasks"].as_array();
//         let rootfs= config["rootfs"].as_str();
//         if rootfs.is_none() {
//             return Err("rootfs not found".to_string());
//         }
//         if tasks.is_none() {
//             return Err("tasks not found".to_string());
//         }
//         let mut result = NsjailConfig::new(&rootfs.unwrap().to_string());
//         for task in tasks.unwrap() {
//             let task = task.as_table();
//             if task.is_none() {
//                 return Err("task is not a table".to_string());
//             }
//             let task = task.unwrap();
//             let mut task_config = HashMap::new();
//             for (key, value) in task {
//                 let value = value.as_str();
//                 if value.is_none() {
//                     return Err("task value is not a string".to_string());
//                 }
//                 task_config.insert(key.to_string(), value.unwrap().to_string());
//             }
//             result.task_config.push(task_config);
//             let mut meta_data = HashMap::new();
//             for (key, value) in task {
//                 let value = value.as_str();
//                 if value.is_none() {
//                     return Err("task value is not a string".to_string());
//                 }
//                 meta_data.insert(key.to_string(), value.unwrap().to_string());
//             }
//             result.meta_data.push(meta_data);
//         }
//         Ok(result)
//     }
// }
//
