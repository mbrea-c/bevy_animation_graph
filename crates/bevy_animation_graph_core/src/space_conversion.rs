use bevy::transform::components::Transform;

use super::{
    pose::{BoneId, BonePose, Pose},
    skeleton::Skeleton,
};
use crate::context::pose_fallback::PoseFallbackContext;

// Implements Copy because it's just immutable references
#[derive(Clone, Copy)]
pub struct SpaceConversionContext<'a> {
    pub pose_fallback: PoseFallbackContext<'a>,
}

impl SpaceConversionContext<'_> {
    pub fn change_bone_space_down(
        &self,
        transform: Transform,
        data: &Pose, // Should be in bone space
        skeleton: &Skeleton,
        source: BoneId,
        target: BoneId,
    ) -> Transform {
        let mut curr_bone_id = target;
        let mut curr_transform = Transform::IDENTITY;

        while curr_bone_id != source {
            let bone_frame = if data.paths.contains_key(&curr_bone_id) {
                let bone_id = data.paths.get(&curr_bone_id).unwrap();
                data.bones[*bone_id].clone()
            } else {
                BonePose::default()
            };
            let curr_local_transform = self.pose_fallback.local_transform(curr_bone_id).unwrap();
            let merged_local_transform = bone_frame.to_transform_with_base(curr_local_transform);

            curr_transform = merged_local_transform * curr_transform;
            curr_bone_id = skeleton.parent(&curr_bone_id).unwrap();
        }

        Transform::from_matrix(curr_transform.to_matrix().inverse()) * transform
    }

    pub fn change_bone_space_up(
        &self,
        transform: Transform,
        data: &Pose, // Should be in bone space
        skeleton: &Skeleton,
        source: BoneId,
        target: BoneId,
    ) -> Transform {
        let mut curr_bone_id = source;
        let mut curr_transform = Transform::IDENTITY;

        while curr_bone_id != target {
            let bone_pose: BonePose = if data.paths.contains_key(&curr_bone_id) {
                let bone_id = data.paths.get(&curr_bone_id).unwrap();
                data.bones[*bone_id].clone()
            } else {
                BonePose::default()
            };
            let curr_local_transform = self.pose_fallback.local_transform(curr_bone_id).unwrap();
            let merged_local_transform = bone_pose.to_transform_with_base(curr_local_transform);

            curr_transform = merged_local_transform * curr_transform;
            curr_bone_id = skeleton.parent(&curr_bone_id).unwrap();
        }

        curr_transform * transform
    }

    pub fn root_to_bone_space(
        &self,
        transform: Transform,
        pose: &Pose, // Should be in bone space
        skeleton: &Skeleton,
        target: BoneId,
    ) -> Transform {
        self.change_bone_space_down(transform, pose, skeleton, skeleton.root(), target)
    }

    pub fn global_to_bone_space(
        &self,
        transform: Transform,
        pose: &Pose, // Should be in bone space
        skeleton: &Skeleton,
        target: BoneId,
    ) -> Transform {
        let character_transform = self.transform_global_to_character(transform, skeleton);
        self.root_to_bone_space(character_transform, pose, skeleton, target)
    }

    pub fn transform_global_to_character(
        &self,
        transform: Transform,
        skeleton: &Skeleton,
    ) -> Transform {
        let root_global_transform = self.pose_fallback.root_global_transform(skeleton).unwrap();
        let inverse_global_transform =
            Transform::from_matrix(root_global_transform.to_matrix().inverse());
        inverse_global_transform * transform
    }

    pub fn character_transform_of_bone(
        &self,
        pose: &Pose,
        skeleton: &Skeleton,
        target: BoneId,
    ) -> Transform {
        self.change_bone_space_up(Transform::IDENTITY, pose, skeleton, target, skeleton.root())
    }

    pub fn global_transform_of_bone(
        &self,
        pose: &Pose,
        skeleton: &Skeleton,
        target: BoneId,
    ) -> Transform {
        let root_transform_global = self.pose_fallback.root_global_transform(skeleton).unwrap();
        root_transform_global.compute_transform()
            * self.character_transform_of_bone(pose, skeleton, target)
    }
}
