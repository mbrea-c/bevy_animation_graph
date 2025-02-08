use bevy::{
    prelude::{Entity, GlobalTransform, Transform},
    utils::HashMap,
};

use crate::core::{id::BoneId, skeleton::Skeleton};

use super::SystemResources;

/// Provides logic for getting a "fallback" bone transform when a pose does not
/// have information for the particular bone.
// Implements Copy because it's just immutable references
#[derive(Clone, Copy)]
pub struct PoseFallbackContext<'a> {
    pub entity_map: &'a HashMap<BoneId, Entity>,
    pub resources: &'a SystemResources<'a, 'a>,
    /// Whether queries for transforms should fallback to the identity transform
    /// if a matching entity cannot be found for the bone.
    /// Useful when using this e.g. draw a pose directly, without spawning a whole scene.
    pub fallback_to_identity: bool,
}

impl PoseFallbackContext<'_> {
    pub fn local_transform(&self, bone_id: BoneId) -> Option<Transform> {
        if let Some(entity) = self.entity_map.get(&bone_id) {
            Some(*self.resources.transform_query.get(*entity).unwrap().0)
        } else {
            if self.fallback_to_identity {
                Some(Transform::IDENTITY)
            } else {
                None
            }
        }
    }

    /// Global transform of the root bone, or fallback if enabled.
    ///
    /// Note that there's no equivalent method for non-root bones, as we only rely
    /// on the global transform of the root bone.
    pub fn root_global_transform(&self, skeleton: &Skeleton) -> Option<GlobalTransform> {
        if let Some(entity) = self.entity_map.get(&skeleton.root()) {
            Some(*self.resources.transform_query.get(*entity).unwrap().1)
        } else {
            if self.fallback_to_identity {
                Some(GlobalTransform::IDENTITY)
            } else {
                None
            }
        }
    }
}
