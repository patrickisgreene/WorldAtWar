use bevy_math::prelude::*;
use bevy_transform::prelude::*;
use serde::{Serialize, Deserialize};

mod vec3_serde {
    use bevy_math::Vec3;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(vec: &Vec3, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        [vec.x, vec.y, vec.z].serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec3, D::Error>
    where
        D: Deserializer<'de>,
    {
        let arr = <[f32; 3]>::deserialize(deserializer)?;
        Ok(Vec3::new(arr[0], arr[1], arr[2]))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transformation {
    pub scale: f32,
    #[serde(default, with = "vec3_serde")]
    pub translation: Vec3,
    #[serde(default, with = "vec3_serde")]
    pub rotation: Vec3,
}

impl Default for Transformation {
    fn default() -> Self {
        Self {
            scale: 1.0,
            translation: Default::default(),
            rotation: Default::default(),
        }
    }
}

impl Transformation {
    pub fn transform(&self, translation: Vec3) -> Transform {
        Transform::from_translation(self.translation + translation)
            .with_rotation(Quat::from_euler(
                bevy_math::EulerRot::XYZ,
                self.rotation.x,
                self.rotation.y,
                self.rotation.z,
            ))
            .with_scale(Vec3::splat(self.scale))
    }
}