use std::fs::read_to_string;

#[derive(Clone, Debug)]
pub struct JudgerResult {
    pub memory: i32,
    pub time: f32,
    pub exit_code: i32,
    pub exit_signal: i32,
    pub report_path: String,
    pub output_path: String
}

impl Default for JudgerResult {
    fn default() -> Self {
        Self {
            memory: -1,
            time: -1.0,
            exit_code: -1,
            exit_signal: -1,
            report_path: "".to_string(),
            output_path: "".to_string()
        }
    }
}

impl JudgerResult {
    pub fn from_file(path: &String, output: &String) -> Self {
        let report = read_to_string(path);
        match report {
            Ok(report) => {
                let lines = report
                    .split_ascii_whitespace()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>();
                if lines.len() == 15 {
                    return Self {
                        memory: lines[2].parse::<i32>().unwrap_or(-1),
                        time: lines[8].parse::<f32>().unwrap_or(-1.0),
                        exit_code: lines[11].parse::<i32>().unwrap_or(-1),
                        exit_signal: lines[14].parse::<i32>().unwrap_or(-1),
                        report_path: path.clone(),
                        output_path: output.clone(),
                    };
                }
                Self::default()
            }
            Err(_) => Self::default()
        }
    }
}