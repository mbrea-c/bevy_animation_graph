use bevy::{
    asset::{Assets, Handle},
    ecs::{
        system::{In, ResMut},
        world::World,
    },
    transform::components::Transform,
};
use bevy_animation_graph::core::{
    ragdoll::{
        bone_mapping::{BodyMapping, BoneMapping, RagdollBoneMap},
        definition::{Body, Collider, Joint, Ragdoll},
    },
    skeleton::Skeleton,
};

use crate::ui::actions::{DynamicAction, run_handler, saving::DirtyAssets};

pub struct EditRagdollBody {
    pub ragdoll: Handle<Ragdoll>,
    pub body: Body,
}

impl DynamicAction for EditRagdollBody {
    fn handle(self: Box<Self>, world: &mut World) {
        run_handler(world, "Could not edit body")(Self::system, *self)
    }
}

impl EditRagdollBody {
    pub fn system(
        In(input): In<Self>,
        mut ragdoll_assets: ResMut<Assets<Ragdoll>>,
        mut dirty_assets: ResMut<DirtyAssets>,
    ) {
        let Some(ragdoll) = ragdoll_assets.get_mut(&input.ragdoll) else {
            return;
        };

        dirty_assets.add(input.ragdoll);

        if let Some(body) = ragdoll.get_body_mut(input.body.id) {
            *body = input.body.clone()
        }
    }
}

pub struct CreateRagdollBody {
    pub ragdoll: Handle<Ragdoll>,
    pub body: Body,
}

impl DynamicAction for CreateRagdollBody {
    fn handle(self: Box<Self>, world: &mut World) {
        run_handler(world, "Could not create body")(Self::system, *self)
    }
}

impl CreateRagdollBody {
    pub fn system(
        In(input): In<Self>,
        mut ragdoll_assets: ResMut<Assets<Ragdoll>>,
        mut dirty_assets: ResMut<DirtyAssets>,
    ) {
        let Some(ragdoll) = ragdoll_assets.get_mut(&input.ragdoll) else {
            return;
        };

        dirty_assets.add(input.ragdoll);

        ragdoll.add_body(input.body);
    }
}

pub struct EditRagdollCollider {
    pub ragdoll: Handle<Ragdoll>,
    pub collider: Collider,
}

impl DynamicAction for EditRagdollCollider {
    fn handle(self: Box<Self>, world: &mut World) {
        run_handler(world, "Could not edit collider")(Self::system, *self)
    }
}

impl EditRagdollCollider {
    pub fn system(
        In(input): In<Self>,
        mut ragdoll_assets: ResMut<Assets<Ragdoll>>,
        mut dirty_assets: ResMut<DirtyAssets>,
    ) {
        let Some(ragdoll) = ragdoll_assets.get_mut(&input.ragdoll) else {
            return;
        };

        dirty_assets.add(input.ragdoll);

        if let Some(collider) = ragdoll.get_collider_mut(input.collider.id) {
            *collider = input.collider.clone()
        }
    }
}

pub struct EditRagdollJoint {
    pub ragdoll: Handle<Ragdoll>,
    pub joint: Joint,
}

impl DynamicAction for EditRagdollJoint {
    fn handle(self: Box<Self>, world: &mut World) {
        run_handler(world, "Could not edit joint")(Self::system, *self)
    }
}

impl EditRagdollJoint {
    pub fn system(
        In(input): In<Self>,
        mut ragdoll_assets: ResMut<Assets<Ragdoll>>,
        mut dirty_assets: ResMut<DirtyAssets>,
    ) {
        let Some(ragdoll) = ragdoll_assets.get_mut(&input.ragdoll) else {
            return;
        };

        dirty_assets.add(input.ragdoll);

        if let Some(joint) = ragdoll.get_joint_mut(input.joint.id) {
            *joint = input.joint.clone()
        }
    }
}

pub struct CreateOrEditBodyMapping {
    pub ragdoll_bone_map: Handle<RagdollBoneMap>,
    pub body_mapping: BodyMapping,
}

impl DynamicAction for CreateOrEditBodyMapping {
    fn handle(self: Box<Self>, world: &mut World) {
        run_handler(world, "Could not edit joint")(Self::system, *self)
    }
}

impl CreateOrEditBodyMapping {
    pub fn system(
        In(input): In<Self>,
        mut ragdoll_bone_map_assets: ResMut<Assets<RagdollBoneMap>>,
        mut dirty_assets: ResMut<DirtyAssets>,
    ) {
        let Some(ragdoll_bone_map) = ragdoll_bone_map_assets.get_mut(&input.ragdoll_bone_map)
        else {
            return;
        };

        dirty_assets.add(input.ragdoll_bone_map);

        ragdoll_bone_map
            .bodies_from_bones
            .insert(input.body_mapping.body_id, input.body_mapping);
    }
}

pub struct CreateOrEditBoneMapping {
    pub ragdoll_bone_map: Handle<RagdollBoneMap>,
    pub bone_mapping: BoneMapping,
}

impl DynamicAction for CreateOrEditBoneMapping {
    fn handle(self: Box<Self>, world: &mut World) {
        run_handler(world, "Could not edit joint")(Self::system, *self)
    }
}

impl CreateOrEditBoneMapping {
    pub fn system(
        In(input): In<Self>,
        mut ragdoll_bone_map_assets: ResMut<Assets<RagdollBoneMap>>,
        mut dirty_assets: ResMut<DirtyAssets>,
    ) {
        let Some(ragdoll_bone_map) = ragdoll_bone_map_assets.get_mut(&input.ragdoll_bone_map)
        else {
            return;
        };

        dirty_assets.add(input.ragdoll_bone_map);

        ragdoll_bone_map
            .bones_from_bodies
            .insert(input.bone_mapping.bone_id.clone(), input.bone_mapping);
    }
}

pub struct RecomputeMappingOffsets {
    pub ragdoll_bone_map: Handle<RagdollBoneMap>,
}

impl DynamicAction for RecomputeMappingOffsets {
    fn handle(self: Box<Self>, world: &mut World) {
        run_handler(world, "Could not edit joint")(Self::system, *self)
    }
}

impl RecomputeMappingOffsets {
    pub fn system(
        In(input): In<Self>,
        mut ragdoll_assets: ResMut<Assets<Ragdoll>>,
        mut skeleton_assets: ResMut<Assets<Skeleton>>,
        mut ragdoll_bone_map_assets: ResMut<Assets<RagdollBoneMap>>,
        mut dirty_assets: ResMut<DirtyAssets>,
    ) {
        let Some(ragdoll_bone_map) = ragdoll_bone_map_assets.get_mut(&input.ragdoll_bone_map)
        else {
            return;
        };
        let Some(ragdoll) = ragdoll_assets.get_mut(&ragdoll_bone_map.ragdoll) else {
            return;
        };
        let Some(skeleton) = skeleton_assets.get_mut(&ragdoll_bone_map.skeleton) else {
            return;
        };

        dirty_assets.add(input.ragdoll_bone_map);

        for body_mapping in ragdoll_bone_map.bodies_from_bones.values_mut() {
            if body_mapping.bone.override_offset {
                continue;
            }
            let Some(body) = ragdoll.get_body(body_mapping.body_id) else {
                continue;
            };

            let Some(bone_default_transform) =
                skeleton.default_transforms(body_mapping.bone.bone.id())
            else {
                continue;
            };

            // Need to get a transform that takes us from bone to body
            // bone * x = body
            // bone.inverse() * bone * x = bone.inverse() * body
            // x = bone.inverse() * body
            let offset =
                Transform::from_matrix(bone_default_transform.character.compute_matrix().inverse())
                    * Transform::from_isometry(body.isometry);

            body_mapping.bone.offset = offset.to_isometry();
        }

        for bone_mapping in ragdoll_bone_map.bones_from_bodies.values_mut() {
            let Some(bone_default_transform) =
                skeleton.default_transforms(bone_mapping.bone_id.id())
            else {
                continue;
            };

            for body_weight in &mut bone_mapping.bodies {
                if body_weight.override_offset {
                    continue;
                }
                let Some(body) = ragdoll.get_body(body_weight.body) else {
                    continue;
                };

                // Need to get a transform that takes us from body to bone
                // body * x = bone
                // body.inverse() * body * x = body.inverse() * bone
                // x = body.inverse() * bone
                let offset =
                    body.isometry.inverse() * bone_default_transform.character.to_isometry();

                body_weight.offset = offset;
            }
        }
    }
}
