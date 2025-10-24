use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum MovementSettings {
    FollowHeightMap,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Movement {
    pub speed: f64,
    #[serde(default)]
    pub altitude: f64,
    #[serde(default)]
    pub settings: Vec<MovementSettings>,
}
