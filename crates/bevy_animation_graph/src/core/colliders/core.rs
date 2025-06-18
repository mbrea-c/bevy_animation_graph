use bevy::{
    asset::{Asset, Handle},
    math::{
        Isometry3d,
        primitives::{Capsule3d, Cuboid, Sphere},
    },
    reflect::Reflect,
};
use serde::{Deserialize, Serialize};

use crate::core::{id::BoneId, skeleton::Skeleton};

#[derive(Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
pub enum ColliderShape {
    Sphere(Sphere),
    Capsule(Capsule3d),
    Cuboid(Cuboid),
}

#[derive(Debug, Clone, Reflect)]
pub struct ColliderConfig {
    pub shape: ColliderShape,
    pub layers: u32,
    pub attached_to: BoneId,
    pub offset: Isometry3d,
}

#[derive(Debug, Clone, Default, Reflect, Asset)]
pub struct SkeletonColliders {
    pub colliders: Vec<ColliderConfig>,
    /// Skeleton colliders only make sense in reference to a skeleton. Users may want
    /// to use different collider setups depending on the situation, hence why we store them as a
    /// separate asset rather than making them part of a skeleton.
    pub skeleton: Handle<Skeleton>,
}
