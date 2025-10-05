use bevy::{
    asset::{Assets, Handle},
    ecs::{
        system::{In, ResMut},
        world::World,
    },
    math::Isometry3d,
    platform::collections::HashMap,
    transform::components::Transform,
};
use bevy_animation_graph::{
    core::{
        ragdoll::{
            bone_mapping::{BodyMapping, BoneMapping, RagdollBoneMap},
            definition::{Body, Collider, ColliderId, Joint, Ragdoll, SymmetrySuffixes},
        },
        skeleton::Skeleton,
    },
    prelude::config::SymmertryMode,
};

use crate::ui::actions::{ActionContext, DynamicAction, run_handler, saving::DirtyAssets};

pub struct EditRagdollBody {
    pub ragdoll: Handle<Ragdoll>,
    pub body: Body,
}

impl DynamicAction for EditRagdollBody {
    fn handle(self: Box<Self>, world: &mut World, ctx: &mut ActionContext) {
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
    fn handle(self: Box<Self>, world: &mut World, _: &mut ActionContext) {
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
    fn handle(self: Box<Self>, world: &mut World, _: &mut ActionContext) {
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
    fn handle(self: Box<Self>, world: &mut World, _: &mut ActionContext) {
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
    fn handle(self: Box<Self>, world: &mut World, _: &mut ActionContext) {
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
    fn handle(self: Box<Self>, world: &mut World, _: &mut ActionContext) {
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
    fn handle(self: Box<Self>, world: &mut World, _: &mut ActionContext) {
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

pub struct RecomputeRagdollSymmetry {
    pub ragdoll_bone_map: Handle<RagdollBoneMap>,
}

impl DynamicAction for RecomputeRagdollSymmetry {
    fn handle(self: Box<Self>, world: &mut World, _: &mut ActionContext) {
        run_handler(world, "Could not edit joint")(Self::system, *self)
    }
}

impl RecomputeRagdollSymmetry {
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

        let suffixes = ragdoll.suffixes.clone();

        dirty_assets.add(input.ragdoll_bone_map);

        // Things to symmetrize:
        // - [ ] Bodies
        // - [ ] Colliders
        // - [ ] Joints
        // - [ ] Body to bone mappings
        // - [ ] Bone to body mappings

        // Maps bodies to their image under symmetry, if requested and already created
        let mut reverse_body_index = HashMap::new();

        for body in ragdoll.iter_bodies() {
            if let Some(source_body_id) = body.created_from {
                reverse_body_index.insert(source_body_id, body.id);
            }
        }

        // Maps colliders to their image under symmetry, if requested and already created
        let mut reverse_collider_index = HashMap::new();

        for collider in ragdoll.iter_colliders() {
            if let Some(source_collider_id) = collider.created_from {
                reverse_collider_index.insert(source_collider_id, collider.id);
            }
        }

        let current_body_ids = ragdoll.iter_body_ids().collect::<Vec<_>>();

        for body_id in current_body_ids {
            let Some(body) = ragdoll.get_body_mut(body_id) else {
                continue;
            };

            let should_use_symmetry = body.use_symmetry && body.created_from.is_none();

            if should_use_symmetry {
                if !body.label.ends_with(&suffixes.original) {
                    body.label = format!("{}{}", body.label, suffixes.original);
                }

                if !reverse_body_index.contains_key(&body.id) {
                    // Create new
                    let mirrored_body = Body::new();
                    reverse_body_index.insert(body_id, mirrored_body.id);
                    ragdoll.bodies.insert(mirrored_body.id, mirrored_body);
                }

                let Some(mirrored_body_id) = reverse_body_index.get(&body_id) else {
                    continue; // should never be reached
                };

                let [Some(original_body), Some(mirrored_body)] =
                    ragdoll.bodies.get_many_mut([&body_id, &mirrored_body_id])
                else {
                    continue; // should never be reached
                };

                mirror_body_properties(
                    original_body,
                    mirrored_body,
                    &suffixes,
                    &SymmertryMode::MirrorX,
                );

                let original_collider_ids = original_body.colliders.clone();
                let mirrored_collider_ids = original_collider_ids
                    .into_iter()
                    .map(|cid| {
                        mirror_collider(cid, &mut reverse_collider_index, ragdoll)
                            .expect("Should always succeed")
                    })
                    .collect();

                let Some(mirrored_body) = ragdoll.get_body_mut(*mirrored_body_id) else {
                    continue;
                };

                mirrored_body.colliders = mirrored_collider_ids;
            }
        }
    }
}

fn mirror_body_properties(
    this: &Body,
    target: &mut Body,
    suffixes: &SymmetrySuffixes,
    mode: &SymmertryMode,
) {
    let original_label = this
        .label
        .strip_suffix(&suffixes.original)
        .unwrap_or(&this.label)
        .to_owned();

    let mirrored_label = format!("{}{}", original_label, suffixes.mirror);
    target.label = mirrored_label;

    let mirrored_isometry = Isometry3d {
        rotation: mode.apply_quat(this.isometry.rotation),
        translation: mode.apply_position(this.isometry.translation.into()).into(),
    };
    target.isometry = mirrored_isometry;

    target.default_mode = this.default_mode.clone();
    target.use_symmetry = false;
    target.created_from = Some(this.id);
}

fn mirror_collider(
    original_collider_id: ColliderId,
    reverse_index: &mut HashMap<ColliderId, ColliderId>,
    ragdoll: &mut Ragdoll,
) -> Option<ColliderId> {
    if !reverse_index.contains_key(&original_collider_id) {
        // Create new
        let mirrored_collider = Collider::new();
        reverse_index.insert(original_collider_id, mirrored_collider.id);
        ragdoll
            .colliders
            .insert(mirrored_collider.id, mirrored_collider);
    }

    let mirrored_collider_id = reverse_index.get(&original_collider_id)?;

    let [Some(original_collider), Some(mirrored_collider)] = ragdoll
        .colliders
        .get_many_mut([&original_collider_id, mirrored_collider_id])
    else {
        return None;
    };

    mirrored_collider.local_offset = original_collider.local_offset;
    mirrored_collider.shape = original_collider.shape.clone();
    mirrored_collider.layer_membership = original_collider.layer_membership;
    mirrored_collider.layer_filter = original_collider.layer_filter;
    mirrored_collider.override_layers = original_collider.override_layers;
    mirrored_collider.label = original_collider.label.clone();
    mirrored_collider.created_from = Some(original_collider_id);

    Some(*mirrored_collider_id)
}
