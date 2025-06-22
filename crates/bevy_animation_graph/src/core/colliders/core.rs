use bevy::{
    asset::{Asset, Handle},
    math::{
        Isometry3d,
        primitives::{Capsule3d, Cuboid, Sphere},
    },
    platform::collections::HashMap,
    reflect::Reflect,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::{id::BoneId, skeleton::Skeleton};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub struct SkeletonColliderId(Uuid);

impl SkeletonColliderId {
    pub fn generate() -> Self {
        SkeletonColliderId(Uuid::new_v4())
    }

    pub fn placeholder() -> Self {
        SkeletonColliderId(Uuid::nil())
    }
}

#[derive(Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
pub enum ColliderShape {
    Sphere(Sphere),
    Capsule(Capsule3d),
    Cuboid(Cuboid),
}

#[derive(Debug, Clone, Reflect)]
pub struct ColliderConfig {
    pub id: SkeletonColliderId,
    pub shape: ColliderShape,
    pub layers: u32,
    pub attached_to: BoneId,
    pub offset: Isometry3d,
}

impl Default for ColliderConfig {
    fn default() -> Self {
        Self {
            id: SkeletonColliderId::placeholder(),
            shape: ColliderShape::Cuboid(Cuboid::new(1., 1., 1.)),
            layers: 0,
            attached_to: BoneId::default(),
            offset: Isometry3d::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Reflect, Asset)]
pub struct SkeletonColliders {
    colliders: HashMap<BoneId, Vec<ColliderConfig>>,
    /// Skeleton colliders only make sense in reference to a skeleton. Users may want
    /// to use different collider setups depending on the situation, hence why we store them as a
    /// separate asset rather than making them part of a skeleton.
    pub skeleton: Handle<Skeleton>,
}

impl SkeletonColliders {
    pub fn get_colliders(&self, bone_id: BoneId) -> Option<&Vec<ColliderConfig>> {
        self.colliders.get(&bone_id)
    }

    pub fn get_colliders_mut(&mut self, bone_id: BoneId) -> Option<&mut Vec<ColliderConfig>> {
        self.colliders.get_mut(&bone_id)
    }

    pub fn add_collider(&mut self, config: ColliderConfig) {
        if let Some(existing) = self.colliders.get_mut(&config.attached_to) {
            existing.push(config);
        } else {
            self.colliders.insert(config.attached_to, vec![config]);
        }
    }

    pub fn delete_collider(&mut self, bone_id: BoneId, collider_id: SkeletonColliderId) {
        self.colliders
            .get_mut(&bone_id)
            .map(|colls| colls.retain(|cfg| cfg.id != collider_id));
    }

    pub fn collider_count(&self) -> usize {
        self.colliders.values().map(|c| c.len()).sum()
    }

    pub fn iter_colliders(&self) -> impl Iterator<Item = &ColliderConfig> {
        self.colliders.values().flatten()
    }
}
