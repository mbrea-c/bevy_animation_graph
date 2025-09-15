use bevy::{asset::Asset, math::Isometry3d, platform::collections::HashMap, reflect::Reflect};
use serde::{Deserialize, Serialize};

use crate::core::{animation_clip::EntityPath, ragdoll::definition::BodyId};

// TODO: Replace EntityPath with bone ids, and add a serial proxy that allows the loader to map

#[derive(Asset, Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct RagdollBoneMap {
    /// We assume the invariant that the sum of all body weights for a bone is always 1.
    bones_from_bodies: HashMap<EntityPath, Vec<BodyWeight>>,
    /// We assume the invariant that the sum of all body weights for a bone is always 1.
    bodies_from_bones: HashMap<BodyId, Vec<BoneWeight>>,
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct BodyWeight {
    body: BodyId,
    weight: f32,
    offset: Isometry3d,
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct BoneWeight {
    bone: EntityPath,
    weight: f32,
    offset: Isometry3d,
}
