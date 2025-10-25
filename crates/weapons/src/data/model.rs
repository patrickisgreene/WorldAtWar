use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::data::Transformation;

#[derive(Serialize, Deserialize)]
pub struct Model {
    pub file: PathBuf,
    #[serde(default)]
    pub transformation: Transformation,
}