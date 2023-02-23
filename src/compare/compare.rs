use difference::{Changeset, Difference};

pub struct CompareConfig {
    pub ignore_end_of_text_enters: Option<i32>,
    pub ignore_end_of_line_space: bool,
    pub fast_compare: bool,
    pub force_fast_compare: usize,
}

// pub fn fuck_crlf()
pub fn remove_end_of_text_enters(target: &mut String, mut limit: i32) -> usize {
    let mut bound = 0usize;
    if limit < 0 { limit = target.len() as i32; }
    let limit = limit as usize;
    for i in target.chars().rev() {
        if i == '\n' { bound += 1; } else { break; }
        if bound >= limit { break; }
    }
    target.truncate(target.len() - bound);
    target.shrink_to_fit();
    bound
}

pub fn compare_string(mut ans: String, mut output: String, config: &CompareConfig) -> Changeset {
    if let Some(num) = config.ignore_end_of_text_enters {
        remove_end_of_text_enters(&mut ans, num);
        remove_end_of_text_enters(&mut output, num);
    }
    if config.ignore_end_of_line_space {
        ans = ans.replace(" \n", "\n");
        output = output.replace(" \n", "\n");
    }
    if config.fast_compare ||
        ans.len() > config.force_fast_compare ||
        output.len() > config.force_fast_compare {
        let result = ans == output;
        return Changeset {
            diffs: vec![],
            split: "".to_string(),
            distance: 1 - result as i32,
        };
    }
    Changeset::new(&ans, &output, "\n")
}

pub fn compare_file(ans_path: &String, output_path: &String, config: &CompareConfig) -> Result<Changeset, String> {
    let mut ans = std::fs::read_to_string(ans_path);

    let mut output = std::fs::read_to_string(output_path);
    if ans.is_err() {
        return Err(format!(
            "Error occur when read answer file: {} :{}",
            &ans_path, &ans.as_ref().err().unwrap().to_string()
        ));
    }
    if output.is_err() {
        return Err(format!(
            "Error occur when read output file: {} :{}",
            &output_path, &output.as_ref().err().unwrap().to_string()
        ));
    }
    Ok(compare_string(ans.unwrap(), output.unwrap(), config))
}