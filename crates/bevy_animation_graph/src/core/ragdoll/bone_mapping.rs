use bevy::{
    asset::{Asset, Handle},
    math::Isometry3d,
    platform::collections::HashMap,
    reflect::Reflect,
};
use serde::{Deserialize, Serialize};

use crate::core::{
    animation_clip::EntityPath,
    ragdoll::definition::{BodyId, Ragdoll},
    skeleton::Skeleton,
};

// TODO: Replace EntityPath with bone ids, and add a serial proxy that allows the loader to map

#[derive(Asset, Debug, Clone, Reflect)]
pub struct RagdollBoneMap {
    /// We assume the invariant that the sum of all body weights for a bone is always 1.
    pub bones_from_bodies: HashMap<EntityPath, BoneMapping>,
    /// We assume the invariant that the sum of all body weights for a bone is always 1.
    pub bodies_from_bones: HashMap<BodyId, BodyMapping>,
    pub skeleton: Handle<Skeleton>,
    pub ragdoll: Handle<Ragdoll>,
}

/// How to get a bone's position from rigidbodies
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct BoneMapping {
    pub bone_id: EntityPath,
    pub bodies: Vec<BodyWeight>,
}

/// How to get a rigibody's position from bones
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct BodyMapping {
    pub body_id: BodyId,
    pub bone: BoneWeight,
}

#[derive(Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct BodyWeight {
    pub body: BodyId,
    pub weight: f32,
    pub offset: Isometry3d,
    #[serde(default)]
    pub override_offset: bool,
}

#[derive(Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct BoneWeight {
    pub bone: EntityPath,
    pub offset: Isometry3d,
    #[serde(default)]
    pub override_offset: bool,
}
