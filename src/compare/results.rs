use std::sync::Arc;

use difference::{Changeset, Difference};

#[derive(PartialEq)]
pub enum BaseStatus {
    AC,
    WA,
    TLE,
    MLE,
    OLE,
    CLE,
    UKE,
    PENDING,
}

pub struct BaseResult {
    pub status: BaseStatus,
    pub changeset: Option<Changeset>,
    pub time: i32,
    pub memory: i32,
    pub info: Option<String>,
}

pub fn clone_changeset(changeset: &Changeset) -> Changeset {
    let tmp = changeset.diffs.iter().map(|x| {
        match x {
            Difference::Same(x) => Difference::Same(x.clone()),
            Difference::Add(x) => Difference::Add(x.clone()),
            Difference::Rem(x) => Difference::Rem(x.clone()),
        }
    }).collect();
    Changeset {
        diffs: tmp,
        split: changeset.split.clone(),
        distance: changeset.distance,
    }
}

impl BaseResult {
    pub fn update(&mut self, changeset: &Changeset) {
        if self.status != BaseStatus::PENDING {
            return;
        }
        if changeset.distance != 0 {
            self.status = BaseStatus::WA;
        } else {
            self.status = BaseStatus::AC;
        }
        self.changeset = Some(clone_changeset(changeset));
    }
}


