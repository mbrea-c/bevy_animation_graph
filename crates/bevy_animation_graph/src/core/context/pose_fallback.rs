use bevy::{
    platform::collections::HashMap,
    prelude::{Entity, GlobalTransform, Transform},
};

use crate::core::{context::system_resources::SystemResources, id::BoneId, skeleton::Skeleton};

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
        } else if self.fallback_to_identity {
            Some(Transform::IDENTITY)
        } else {
            None
        }
    }

    /// Global transform of the root bone, or fallback if enabled.
    ///
    /// Note that there's no equivalent method for non-root bones, as we only rely
    /// on the global transform of the root bone.
    pub fn root_global_transform(&self, skeleton: &Skeleton) -> Option<GlobalTransform> {
        if let Some(entity) = self.entity_map.get(&skeleton.root()) {
            Some(*self.resources.transform_query.get(*entity).unwrap().1)
        } else if self.fallback_to_identity {
            Some(GlobalTransform::IDENTITY)
        } else {
            None
        }
    }

    #[cfg(feature = "physics_avian")]
    /// Finds the offset from the root bone of the skeleton to either the root of the hierarchy (a.k.a.
    /// global position) or the closest rigidbody in the hierarchy
    pub fn compute_root_global_transform_to_rigidbody(
        &self,
        skeleton: &Skeleton,
    ) -> RootOffsetResult {
        let Some(start) = self.entity_map.get(&skeleton.root()) else {
            return RootOffsetResult::Failed;
        };
        let mut current = Some(*start);
        let mut cumulative_transform = Transform::IDENTITY;

        while let Some(entity) = current {
            let Ok((transform, _)) = self.resources.transform_query.get(entity) else {
                return RootOffsetResult::Failed;
            };
            cumulative_transform = *transform * cumulative_transform;
            current = self
                .resources
                .parent_query
                .get(entity)
                .map(|child_of| child_of.0)
                .ok();

            if let Some(parent) = current
                && self.resources.rigidbody_query.contains(parent)
            {
                return RootOffsetResult::FromRigidbody(parent, cumulative_transform);
            }
        }

        RootOffsetResult::FromRoot(cumulative_transform)
    }
}

pub enum RootOffsetResult {
    FromRigidbody(Entity, Transform),
    FromRoot(Transform),
    Failed,
}
