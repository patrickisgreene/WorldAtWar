use serde::{Serialize, Deserialize};

use crate::data::Transformation;

#[derive(Debug, Serialize, Deserialize)]
pub struct Radar {
    pub strength: f32,
    pub shape: RadarShape,
    #[serde(default)]
    pub transformation: Transformation,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RadarShape {
    Sphere(f64),
    Cone { radius: f64, length: f64 },
}