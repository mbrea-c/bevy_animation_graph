use bevy::{asset::AssetPath, reflect::Reflect};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SkeletonSerial {
    /// Path to animated scene source
    pub source: SkeletonSource,
}

#[derive(Clone, Reflect, Serialize, Deserialize)]
pub enum SkeletonSource {
    Gltf {
        source: AssetPath<'static>,
        label: String,
    },
}
