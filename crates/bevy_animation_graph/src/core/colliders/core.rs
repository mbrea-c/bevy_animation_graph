use bevy::{
    asset::{Asset, Handle},
    math::{
        Isometry3d, Vec3,
        primitives::{Capsule3d, Cuboid, Sphere},
    },
    platform::collections::HashMap,
    reflect::Reflect,
    transform::components::Transform,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    core::{
        id::BoneId,
        skeleton::{DefaultBoneTransform, Skeleton},
    },
    prelude::config::SymmetryConfig,
};

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

#[derive(Debug, Default, Clone, Copy, Reflect, PartialEq, Serialize, Deserialize, Hash)]
pub enum ColliderOffsetMode {
    #[default]
    Local,
    Global,
}

#[derive(Debug, Clone, Reflect)]
pub struct ColliderConfig {
    pub id: SkeletonColliderId,
    pub shape: ColliderShape,
    pub layers: u32,
    pub attached_to: BoneId,
    pub offset: Isometry3d,
    pub offset_mode: ColliderOffsetMode,
}

impl ColliderConfig {
    pub fn local_transform(&self, default_transforms: &DefaultBoneTransform) -> Transform {
        match self.offset_mode {
            ColliderOffsetMode::Local => Transform::from_isometry(self.offset),
            ColliderOffsetMode::Global => Transform {
                translation: default_transforms.global.rotation.inverse()
                    * Vec3::from(self.offset.translation),
                rotation: default_transforms.global.rotation.inverse() * self.offset.rotation,
                scale: Vec3::ONE,
            },
        }
    }
}

impl Default for ColliderConfig {
    fn default() -> Self {
        Self {
            id: SkeletonColliderId::placeholder(),
            shape: ColliderShape::Cuboid(Cuboid::new(1., 1., 1.)),
            layers: 0,
            attached_to: BoneId::default(),
            offset: Isometry3d::default(),
            offset_mode: ColliderOffsetMode::default(),
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
    pub symmetry: SymmetryConfig,
    pub symmetry_enabled: bool,
    /// Default physics layer memberships if not overriden
    pub default_layer_membership: u32,
    /// Default physics layer filters if not overriden
    pub default_layer_filter: u32,
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
        if let Some(colls) = self.colliders.get_mut(&bone_id) {
            colls.retain(|cfg| cfg.id != collider_id)
        }
    }

    pub fn collider_count(&self) -> usize {
        self.colliders.values().map(|c| c.len()).sum()
    }

    pub fn iter_colliders(&self) -> impl Iterator<Item = &ColliderConfig> {
        self.colliders.values().flatten()
    }

    pub fn iter_bones(&self) -> impl Iterator<Item = BoneId> {
        self.colliders.keys().copied()
    }
}
