use bevy::{
    asset::{Assets, Handle},
    ecs::{
        system::{In, InMut, ResMut},
        world::World,
    },
    math::Isometry3d,
    platform::collections::HashMap,
    transform::components::Transform,
};
use bevy_animation_graph::{
    core::{
        animation_clip::EntityPath,
        ragdoll::{
            bone_mapping::{BodyMapping, BodyWeight, BoneMapping, RagdollBoneMap},
            definition::{
                AngleLimit, Body, BodyId, Collider, ColliderId, Joint, JointId, JointVariant,
                Ragdoll, SymmetrySuffixes,
            },
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
    fn handle(self: Box<Self>, world: &mut World, _ctx: &mut ActionContext) {
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

pub struct CreateRagdollCollider {
    pub ragdoll: Handle<Ragdoll>,
    pub collider: Collider,
    pub attach_to: BodyId,
}

impl DynamicAction for CreateRagdollCollider {
    fn handle(self: Box<Self>, world: &mut World, _: &mut ActionContext) {
        run_handler(world, "Could not create body")(Self::system, *self)
    }
}

impl CreateRagdollCollider {
    pub fn system(
        In(input): In<Self>,
        mut ragdoll_assets: ResMut<Assets<Ragdoll>>,
        mut dirty_assets: ResMut<DirtyAssets>,
    ) {
        let Some(ragdoll) = ragdoll_assets.get_mut(&input.ragdoll) else {
            return;
        };

        dirty_assets.add(input.ragdoll);

        let Some(body) = ragdoll.get_body_mut(input.attach_to) else {
            return;
        };

        body.colliders.push(input.collider.id);

        ragdoll.add_collider(input.collider);
    }
}

pub struct CreateRagdollJoint {
    pub ragdoll: Handle<Ragdoll>,
    pub joint: Joint,
}

impl DynamicAction for CreateRagdollJoint {
    fn handle(self: Box<Self>, world: &mut World, _: &mut ActionContext) {
        run_handler(world, "Could not create body")(Self::system, *self)
    }
}

impl CreateRagdollJoint {
    pub fn system(
        In(input): In<Self>,
        mut ragdoll_assets: ResMut<Assets<Ragdoll>>,
        mut dirty_assets: ResMut<DirtyAssets>,
    ) {
        let Some(ragdoll) = ragdoll_assets.get_mut(&input.ragdoll) else {
            return;
        };

        dirty_assets.add(input.ragdoll);

        ragdoll.add_joint(input.joint);
    }
}

pub struct DeleteRagdollBody {
    pub ragdoll: Handle<Ragdoll>,
    pub body_id: BodyId,
}

impl DynamicAction for DeleteRagdollBody {
    fn handle(self: Box<Self>, world: &mut World, _: &mut ActionContext) {
        run_handler(world, "Could not create body")(Self::system, *self)
    }
}

impl DeleteRagdollBody {
    pub fn system(
        In(input): In<Self>,
        mut ragdoll_assets: ResMut<Assets<Ragdoll>>,
        mut dirty_assets: ResMut<DirtyAssets>,
    ) {
        let Some(ragdoll) = ragdoll_assets.get_mut(&input.ragdoll) else {
            return;
        };

        dirty_assets.add(input.ragdoll);

        ragdoll.delete_body(input.body_id);
    }
}

pub struct DeleteRagdollCollider {
    pub ragdoll: Handle<Ragdoll>,
    pub collider_id: ColliderId,
}

impl DynamicAction for DeleteRagdollCollider {
    fn handle(self: Box<Self>, world: &mut World, _: &mut ActionContext) {
        run_handler(world, "Could not create body")(Self::system, *self)
    }
}

impl DeleteRagdollCollider {
    pub fn system(
        In(input): In<Self>,
        mut ragdoll_assets: ResMut<Assets<Ragdoll>>,
        mut dirty_assets: ResMut<DirtyAssets>,
    ) {
        let Some(ragdoll) = ragdoll_assets.get_mut(&input.ragdoll) else {
            return;
        };

        dirty_assets.add(input.ragdoll);

        ragdoll.delete_collider(input.collider_id);
    }
}

pub struct DeleteRagdollJoint {
    pub ragdoll: Handle<Ragdoll>,
    pub joint_id: JointId,
}

impl DynamicAction for DeleteRagdollJoint {
    fn handle(self: Box<Self>, world: &mut World, _: &mut ActionContext) {
        run_handler(world, "Could not create body")(Self::system, *self)
    }
}

impl DeleteRagdollJoint {
    pub fn system(
        In(input): In<Self>,
        mut ragdoll_assets: ResMut<Assets<Ragdoll>>,
        mut dirty_assets: ResMut<DirtyAssets>,
    ) {
        let Some(ragdoll) = ragdoll_assets.get_mut(&input.ragdoll) else {
            return;
        };

        dirty_assets.add(input.ragdoll);

        ragdoll.delete_joint(input.joint_id);
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
                Transform::from_matrix(bone_default_transform.character.to_matrix().inverse())
                    * Transform::from_translation(body.offset);

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
                let offset = Isometry3d::from_translation(body.offset).inverse()
                    * bone_default_transform.character.to_isometry();

                body_weight.offset = offset;
            }
        }
    }
}

pub struct RecomputeRagdollSymmetry {
    pub ragdoll_bone_map: Handle<RagdollBoneMap>,
}

impl DynamicAction for RecomputeRagdollSymmetry {
    fn handle(self: Box<Self>, world: &mut World, ctx: &mut ActionContext) {
        run_handler(world, "Could not edit joint")(Self::system, (*self, ctx))
    }
}

impl RecomputeRagdollSymmetry {
    pub fn system(
        (In(input), InMut(ctx)): (In<Self>, InMut<ActionContext>),
        mut ragdoll_assets: ResMut<Assets<Ragdoll>>,
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

        let suffixes = ragdoll.suffixes.clone();

        dirty_assets.add(ragdoll_bone_map.ragdoll.clone());
        dirty_assets.add(input.ragdoll_bone_map.clone());

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

        // Maps joints to their image under symmetry, if requested and already created
        let mut reverse_joint_index = HashMap::new();

        for joint in ragdoll.iter_joints() {
            if let Some(source_joint_id) = joint.created_from {
                reverse_joint_index.insert(source_joint_id, joint.id);
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
                    ragdoll.bodies.get_many_mut([&body_id, mirrored_body_id])
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

        let current_joint_ids = ragdoll.iter_joint_ids().collect::<Vec<_>>();

        for joint_id in current_joint_ids {
            let Some(joint) = ragdoll.get_joint(joint_id) else {
                continue;
            };

            if !joint.use_symmetry || joint.created_from.is_some() {
                continue;
            }

            mirror_joint(
                joint_id,
                &mut reverse_body_index,
                &mut reverse_joint_index,
                ragdoll,
            )
            .expect("Failed to mirror joint");
        }

        let mapping_body_ids: Vec<BodyId> =
            ragdoll_bone_map.bodies_from_bones.keys().copied().collect();
        for body_id in mapping_body_ids {
            let Some(body) = ragdoll.get_body(body_id) else {
                continue;
            };
            if !body.use_symmetry || body.created_from.is_some() {
                continue;
            }

            mirror_body_mapping(body_id, &mut reverse_body_index, ragdoll_bone_map)
                .expect("Failed to mirror body mapping");
        }

        let mapping_bone_paths: Vec<EntityPath> =
            ragdoll_bone_map.bones_from_bodies.keys().cloned().collect();

        for bone_path in mapping_bone_paths {
            let Some(mapping) = ragdoll_bone_map.bones_from_bodies.get(&bone_path) else {
                continue;
            };
            if mapping.created_from.is_some() {
                continue;
            };
            mirror_bone_mapping(bone_path, &mut reverse_body_index, ragdoll_bone_map);
        }

        // Cleanup section
        let current_body_ids = ragdoll.iter_body_ids().collect::<Vec<_>>();
        for body_id in current_body_ids {
            let Some(body) = ragdoll.get_body(body_id) else {
                continue;
            };

            let Some(source_body_id) = body.created_from else {
                continue;
            };

            let Some(source_body) = ragdoll.get_body(source_body_id) else {
                for collider_id in body.colliders.clone() {
                    ragdoll.colliders.remove(&collider_id);
                }
                ragdoll.bodies.remove(&body_id);
                continue;
            };

            if !source_body.use_symmetry {
                for collider_id in body.colliders.clone() {
                    ragdoll.colliders.remove(&collider_id);
                }
                ragdoll.bodies.remove(&body_id);
            }
        }

        let current_joint_ids = ragdoll.iter_joint_ids().collect::<Vec<_>>();
        for joint_id in current_joint_ids {
            let Some(joint) = ragdoll.get_joint(joint_id) else {
                continue;
            };

            let Some(source_joint_id) = joint.created_from else {
                continue;
            };

            let Some(source_joint) = ragdoll.get_joint(source_joint_id) else {
                ragdoll.joints.remove(&joint_id);
                continue;
            };

            if !source_joint.use_symmetry {
                ragdoll.joints.remove(&joint_id);
            }
        }

        let mapping_body_ids: Vec<BodyId> =
            ragdoll_bone_map.bodies_from_bones.keys().copied().collect();
        for body_id in mapping_body_ids {
            let Some(mapping) = ragdoll_bone_map.bodies_from_bones.get(&body_id) else {
                continue;
            };

            let Some(source_body_id) = mapping.created_from else {
                continue;
            };

            let Some(source_body) = ragdoll.get_body(source_body_id) else {
                ragdoll_bone_map.bodies_from_bones.remove(&body_id);
                continue;
            };

            if !source_body.use_symmetry {
                ragdoll_bone_map.bodies_from_bones.remove(&body_id);
            }
        }

        ctx.actions.dynamic(RecomputeMappingOffsets {
            ragdoll_bone_map: input.ragdoll_bone_map.clone(),
        });
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

    let mirror_label = format!("{}{}", original_label, suffixes.mirror);
    target.label = mirror_label;

    let mirror_offset = mode.apply_position(this.offset);
    target.offset = mirror_offset;

    target.default_mode = this.default_mode;
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

fn mirror_joint(
    original_joint_id: JointId,
    reverse_body_index: &mut HashMap<BodyId, BodyId>,
    reverse_joint_index: &mut HashMap<JointId, JointId>,
    ragdoll: &mut Ragdoll,
) -> Option<JointId> {
    if !reverse_joint_index.contains_key(&original_joint_id) {
        // Create new
        let mirrored_joint = Joint::new();
        reverse_joint_index.insert(original_joint_id, mirrored_joint.id);
        ragdoll.joints.insert(mirrored_joint.id, mirrored_joint);
    }

    let mirrored_joint_id = reverse_joint_index.get(&original_joint_id)?;

    let suffixes = ragdoll.suffixes.clone();

    let [Some(original_joint), Some(mirrored_joint)] = ragdoll
        .joints
        .get_many_mut([&original_joint_id, mirrored_joint_id])
    else {
        return None;
    };

    let original_label = original_joint
        .label
        .strip_suffix(&suffixes.original)
        .unwrap_or(&original_joint.label)
        .to_owned();

    let label = format!("{}{}", original_label, suffixes.original);
    let mirrored_label = format!("{}{}", original_label, suffixes.mirror);

    original_joint.label = label;
    mirrored_joint.label = mirrored_label;

    match &original_joint.variant {
        JointVariant::Spherical(inner_joint) => {
            let mut mirror_spherical_joint = inner_joint.clone();

            let mirrored_body1 = reverse_body_index
                .get(&inner_joint.body1)
                .copied()
                .unwrap_or(inner_joint.body1);
            let mirrored_body2 = reverse_body_index
                .get(&inner_joint.body2)
                .copied()
                .unwrap_or(inner_joint.body2);

            mirror_spherical_joint.body1 = mirrored_body1;
            mirror_spherical_joint.body2 = mirrored_body2;

            mirror_spherical_joint.twist_limit =
                inner_joint.twist_limit.as_ref().map(|l| AngleLimit {
                    min: -l.max,
                    max: -l.min,
                });

            mirror_spherical_joint.position =
                SymmertryMode::MirrorX.apply_position(inner_joint.position);

            mirror_spherical_joint.twist_axis =
                SymmertryMode::MirrorX.apply_position(inner_joint.twist_axis);

            mirrored_joint.variant = JointVariant::Spherical(mirror_spherical_joint);
        }
        JointVariant::Revolute(inner_joint) => {
            let mut mirror_inner_joint = inner_joint.clone();

            let mirrored_body1 = reverse_body_index
                .get(&inner_joint.body1)
                .copied()
                .unwrap_or(inner_joint.body1);
            let mirrored_body2 = reverse_body_index
                .get(&inner_joint.body2)
                .copied()
                .unwrap_or(inner_joint.body2);

            mirror_inner_joint.body1 = mirrored_body1;
            mirror_inner_joint.body2 = mirrored_body2;

            mirror_inner_joint.angle_limit = inner_joint.angle_limit.as_ref().map(|l| AngleLimit {
                min: -l.max,
                max: -l.min,
            });

            mirror_inner_joint.position =
                SymmertryMode::MirrorX.apply_position(inner_joint.position);

            mirror_inner_joint.hinge_axis =
                SymmertryMode::MirrorX.apply_position(inner_joint.hinge_axis);

            mirrored_joint.variant = JointVariant::Revolute(mirror_inner_joint);
        }
    }

    mirrored_joint.use_symmetry = false;
    mirrored_joint.created_from = Some(original_joint_id);

    Some(*mirrored_joint_id)
}

fn mirror_body_mapping(
    original_target: BodyId,
    reverse_body_index: &mut HashMap<BodyId, BodyId>,
    ragdoll_bone_map: &mut RagdollBoneMap,
) -> Option<()> {
    let mirror_target = reverse_body_index.get(&original_target)?;

    if !ragdoll_bone_map
        .bodies_from_bones
        .contains_key(mirror_target)
    {
        // Create new
        let mirror_mapping = BodyMapping::default();

        ragdoll_bone_map
            .bodies_from_bones
            .insert(*mirror_target, mirror_mapping);
    }

    let [Some(original_mapping), Some(mirror_mapping)] = ragdoll_bone_map
        .bodies_from_bones
        .get_many_mut([&original_target, mirror_target])
    else {
        return None;
    };

    let mirror_bone_path = ragdoll_bone_map
        .skeleton_symmetry
        .name_mapper
        .flip(&original_mapping.bone.bone);

    mirror_mapping.body_id = *mirror_target;
    mirror_mapping.bone = original_mapping.bone.clone();
    mirror_mapping.bone.bone = mirror_bone_path;
    mirror_mapping.created_from = Some(original_target);

    Some(())
}

fn mirror_bone_mapping(
    original_target: EntityPath,
    reverse_body_index: &mut HashMap<BodyId, BodyId>,
    ragdoll_bone_map: &mut RagdollBoneMap,
) -> Option<()> {
    let mirror_target = ragdoll_bone_map
        .skeleton_symmetry
        .name_mapper
        .flip(&original_target);
    if mirror_target == original_target {
        // Nothing to do
        return None;
    }

    if !ragdoll_bone_map
        .bones_from_bodies
        .contains_key(&mirror_target)
    {
        // Create new
        let mirror_mapping = BoneMapping::default();
        ragdoll_bone_map
            .bones_from_bodies
            .insert(mirror_target.clone(), mirror_mapping);
    }

    let [Some(original_mapping), Some(mirror_mapping)] = ragdoll_bone_map
        .bones_from_bodies
        .get_many_mut([&original_target, &mirror_target])
    else {
        return None;
    };

    mirror_mapping.bone_id = mirror_target;
    let mut bodies = Vec::new();

    for body_weight in &original_mapping.bodies {
        let mirror_body_weight = BodyWeight {
            body: reverse_body_index
                .get(&body_weight.body)
                .copied()
                .unwrap_or(body_weight.body),
            weight: body_weight.weight,
            offset: SymmertryMode::MirrorX.apply_isometry_3d(body_weight.offset),
            override_offset: body_weight.override_offset,
        };

        bodies.push(mirror_body_weight);
    }

    mirror_mapping.bodies = bodies;
    mirror_mapping.created_from = Some(original_target);

    Some(())
}
