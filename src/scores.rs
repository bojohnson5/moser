use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Default)]
pub struct ScoreData {
    pub lessons: HashMap<String, Vec<u32>>, // lesson number -> list of scores (%)
}
