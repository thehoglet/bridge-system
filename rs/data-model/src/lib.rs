use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Open {
    pub open: String,
    pub meaning: String,
    pub notes: Option<Vec<String>>,
    pub pass: Option<BTreeMap<String, Continuation>>,
    pub fourth: Option<String>,
}

impl Default for Open {
    fn default() -> Open {
        Open {
            open: "".into(),
            meaning: "".into(),
            notes: None,
            pass: None,
            fourth: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Continuation {
    pub meaning: String,
    pub notes: Option<Vec<String>>,
    pub rebid: Option<String>,
    pub pass: Option<BTreeMap<String, Continuation>>,
}

impl Default for Continuation {
    fn default() -> Continuation {
        Continuation {
            meaning: "".into(),
            notes: None,
            rebid: None,
            pass: None,
        }
    }
}
