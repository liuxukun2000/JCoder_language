use difference::Changeset;
use crate::compare::compare::CompareConfig;
use crate::compare::results::{BaseResult, BaseStatus};

impl Default for CompareConfig {
    fn default() -> Self {
        Self {
            ignore_end_of_text_enters: Some(1),
            ignore_end_of_line_space: true,
            fast_compare: false,
            force_fast_compare: 30000,
        }
    }
}

impl Default for BaseResult {
    fn default() -> Self {
        Self {
            status: BaseStatus::PENDING,
            changeset: None,
            time: 0,
            memory: 0,
            info: None,
        }
    }
}

pub trait ResultTrait {
    fn update(&mut self, changeset: &Changeset);
}