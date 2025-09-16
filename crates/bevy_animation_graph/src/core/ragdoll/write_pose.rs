//! Utilities for mapping a skeleton to a desired ragdoll configuration

use bevy::math::Isometry3d;

use crate::{
    core::{
        pose::Pose,
        ragdoll::{
            bone_mapping::RagdollBoneMap,
            definition::{BodyId, Ragdoll},
        },
        skeleton::Skeleton,
        space_conversion::SpaceConversionContext,
    },
    prelude::PoseFallbackContext,
};

pub struct BodyDesire {
    pub body_id: BodyId,
    /// Isometry relative to the character root transform
    pub character_space_isometry: Isometry3d,
}

pub struct RagdollDesires {
    pub bodies: Vec<BodyDesire>,
}

pub fn write_pose_to_ragdoll(
    pose: &Pose,
    skeleton: &Skeleton,
    ragdoll: &Ragdoll,
    mapping: &RagdollBoneMap,
    pose_fallback_ctx: PoseFallbackContext,
) -> RagdollDesires {
    let mut desires = RagdollDesires { bodies: Vec::new() };
    let convert = SpaceConversionContext {
        pose_fallback: pose_fallback_ctx,
    };

    for body in &ragdoll.bodies {
        let Some(bone_weight) = mapping.bodies_from_bones.get(&body.id) else {
            continue;
        };

        // TODO: Quaternion interpolation for more than 1 bone weights
        let character_transform =
            convert.character_transform_of_bone(pose, skeleton, bone_weight.bone.id());

        desires.bodies.push(BodyDesire {
            body_id: body.id,
            character_space_isometry: character_transform.to_isometry(),
        })
    }

    desires
}
