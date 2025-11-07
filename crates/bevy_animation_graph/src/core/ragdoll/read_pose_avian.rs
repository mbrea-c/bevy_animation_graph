use avian3d::prelude::{Position, Rotation};
use bevy::{ecs::system::Query, math::Isometry3d, transform::components::Transform};

use crate::{
    core::{
        pose::{BonePose, Pose},
        ragdoll::{
            bone_mapping::RagdollBoneMap, configuration::RagdollConfig, spawning::SpawnedRagdoll,
        },
        skeleton::Skeleton,
        space_conversion::SpaceConversionContext,
    },
    prelude::PoseFallbackContext,
};

pub fn read_pose(
    spawned_ragdoll: &SpawnedRagdoll,
    bone_map: &RagdollBoneMap,
    skeleton: &Skeleton,
    query: &Query<(&Position, &Rotation)>,
    pose_fallback_context: PoseFallbackContext,
    base_pose: &Pose,
    config: &RagdollConfig,
) -> Pose {
    let mut pose = base_pose.clone();

    let conversions = SpaceConversionContext {
        pose_fallback: pose_fallback_context,
    };

    let mut bones_from_bodies = bone_map.bones_from_bodies.iter().collect::<Vec<_>>();
    bones_from_bodies.sort_by_key(|(path, _)| path.parts.len());

    for (bone_path, bone_mapping) in bones_from_bodies {
        let should_readback = config.should_readback(bone_path.id()).unwrap_or(true);
        if !should_readback {
            continue;
        }
        let parent_bone_transform = bone_path
            .parent()
            .map(|parent_bone_path| {
                conversions.global_transform_of_bone(&pose, skeleton, parent_bone_path.id())
            })
            .unwrap_or_default();

        let inverse_parent_transform =
            Transform::from_matrix(parent_bone_transform.compute_affine().inverse().into());

        let Some(mut weighted_transforms) = bone_mapping
            .bodies
            .iter()
            .filter_map(|body_weight| {
                let body_entity = spawned_ragdoll.bodies.get(&body_weight.body)?;
                let (pos, rot) = query.get(*body_entity).ok()?;

                let global_isometry = Isometry3d::new(pos.0, rot.0) * body_weight.offset;
                let global_transform = Transform::from_isometry(global_isometry);

                let local_transform = inverse_parent_transform * global_transform;

                Some(Transform {
                    translation: local_transform.translation * body_weight.weight,
                    rotation: local_transform.rotation * body_weight.weight,
                    scale: local_transform.scale * body_weight.weight,
                })
            })
            .reduce(|left, right| Transform {
                translation: left.translation + right.translation,
                rotation: left.rotation + right.rotation,
                scale: left.scale + right.scale,
            })
        else {
            continue;
        };

        weighted_transforms.rotation = weighted_transforms.rotation.normalize();

        let bone_pose = BonePose::from_transform(weighted_transforms);
        pose.add_bone(bone_pose, bone_path.id());
    }

    pose
}
