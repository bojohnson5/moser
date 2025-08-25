use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Default)]
pub struct ScoreData {
    pub lessons: HashMap<u8, Vec<u32>>, // lesson number -> list of scores (%)
}
