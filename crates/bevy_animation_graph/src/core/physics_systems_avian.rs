use avian3d::prelude::{AngularVelocity, LinearVelocity, Position, RigidBody, Rotation};
use bevy::asset::Assets;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::query::With;
use bevy::ecs::system::{Commands, Res};
use bevy::ecs::{query::Without, system::Query};
use bevy::math::{Isometry3d, Vec3};
use bevy::time::Time;
use bevy::transform::components::GlobalTransform;

use crate::core::animation_graph::DEFAULT_OUTPUT_RAGDOLL_CONFIG;
use crate::core::ragdoll::bone_mapping::RagdollBoneMap;
use crate::core::ragdoll::definition::{BodyMode, Ragdoll};
use crate::core::ragdoll::read_pose_avian::read_pose;
use crate::core::ragdoll::relative_kinematic_body::{
    RelativeKinematicBody, RelativeKinematicBodyPositionBased,
};
use crate::core::ragdoll::spawning::spawn_ragdoll_avian;
use crate::core::ragdoll::write_pose::write_pose_to_ragdoll;
use crate::prelude::{AnimationGraphPlayer, PoseFallbackContext, SystemResources};

use super::context::RootOffsetResult;

pub fn update_relative_kinematic_body_velocities(
    mut relative_kinematic_query: Query<(
        &RelativeKinematicBody,
        &RigidBody,
        &mut LinearVelocity,
        &mut AngularVelocity,
    )>,
    relative_to_query: Query<(&LinearVelocity, &AngularVelocity), Without<RelativeKinematicBody>>,
) {
    for (relative_kinematic_body, rigid_body, mut linvel, mut angvel) in
        &mut relative_kinematic_query
    {
        if !rigid_body.is_kinematic() {
            continue;
        }
        let (base_linvel, base_angvel) = relative_kinematic_body
            .relative_to
            .and_then(|relative_to| relative_to_query.get(relative_to).ok())
            .map(|(linvel, angvel)| (linvel.0, angvel.0))
            .unwrap_or((Vec3::ZERO, Vec3::ZERO));

        let composed_linvel = base_linvel + relative_kinematic_body.kinematic_linear_velocity;
        let composed_angvel = base_angvel + relative_kinematic_body.kinematic_angular_velocity;

        linvel.0 = composed_linvel;
        angvel.0 = composed_angvel;
    }
}

pub fn update_relative_kinematic_position_based_body_velocities(
    mut relative_kinematic_query: Query<(
        &RelativeKinematicBodyPositionBased,
        &Position,
        &Rotation,
        &mut RelativeKinematicBody,
    )>,
    relative_to_query: Query<(&Position, &Rotation), Without<RelativeKinematicBodyPositionBased>>,
    time: Res<Time>,
) {
    for (rel_kinbod_pos, pos, rot, mut rel_kinbod) in &mut relative_kinematic_query {
        // First we need to compute the current relative position
        let cur_isometry = if let Some((base_pos, base_rot)) = rel_kinbod_pos
            .relative_to
            .and_then(|e| relative_to_query.get(e).ok())
        {
            let cur_global_isometry = Isometry3d::new(pos.0, rot.0);
            let cur_base_isometry = Isometry3d::new(base_pos.0, base_rot.0);

            cur_base_isometry.inverse() * cur_global_isometry
        } else {
            Isometry3d::new(pos.0, rot.0)
        };

        let linvel = (rel_kinbod_pos.relative_target.translation - cur_isometry.translation)
            / time.delta_secs();

        let start = cur_isometry.rotation;
        let mut end = rel_kinbod_pos.relative_target.rotation;
        if start.dot(end) < 0.0 {
            end = -end;
        }
        let quat_diff = (end * start.conjugate()).normalize();
        let axis_angle_diff = quat_diff.to_scaled_axis();
        let angvel = axis_angle_diff / time.delta_secs();

        rel_kinbod.kinematic_linear_velocity = linvel.into();
        rel_kinbod.kinematic_angular_velocity = angvel;
        rel_kinbod.relative_to = rel_kinbod_pos.relative_to;
    }
}

pub fn spawn_missing_ragdolls_avian(
    mut commands: Commands,
    mut animation_players: Query<(Entity, &mut AnimationGraphPlayer, &GlobalTransform)>,
    parent_query: Query<&ChildOf>,
    velocity_check: Query<(), (With<LinearVelocity>, With<AngularVelocity>)>,
    ragdoll_assets: Res<Assets<Ragdoll>>,
) {
    for (entity, mut player, global_transform) in &mut animation_players {
        if let Some(ragdoll_asset_id) = player.ragdoll.as_ref().map(|h| h.id())
            && player.spawned_ragdoll.is_none()
            && let Some(ragdoll) = ragdoll_assets.get(ragdoll_asset_id)
        {
            // We need to find the nearest simulated "ancestor". This will be the origin for
            // relative kinematic bodies in the ragdoll
            let simulated_parent = parent_query
                .iter_ancestors(entity)
                .find(|ancestor| velocity_check.contains(*ancestor));

            let spawned_ragdoll = spawn_ragdoll_avian(
                ragdoll,
                global_transform.to_isometry(),
                simulated_parent,
                &mut commands,
            );
            player.spawned_ragdoll = Some(spawned_ragdoll);
        }
    }
}

/// Updates the rigidbody modes of bodies in a ragdoll based on configuration
pub fn update_ragdoll_rigidbodies(
    animation_players: Query<&AnimationGraphPlayer>,
    ragdoll_assets: Res<Assets<Ragdoll>>,
    rigid_body_query: Query<&RigidBody>,
    mut commands: Commands,
) {
    for player in &animation_players {
        if let Some(ragdoll_asset_id) = player.ragdoll.as_ref().map(|h| h.id())
            && let Some(ragdoll) = ragdoll_assets.get(ragdoll_asset_id)
            && let Some(spawned_ragdoll) = &player.spawned_ragdoll
        {
            let config = player
                .get_outputs()
                .get(DEFAULT_OUTPUT_RAGDOLL_CONFIG)
                .and_then(|v| v.as_ragdoll_config().ok())
                .cloned()
                .unwrap_or_default();

            for body in ragdoll.bodies.values() {
                let Some(body_entity) = spawned_ragdoll.bodies.get(&body.id) else {
                    continue;
                };
                let Ok(rigid_body) = rigid_body_query.get(*body_entity) else {
                    continue;
                };

                let body_mode = config.body_mode(body.id).unwrap_or(body.default_mode);

                let target_mode = match body_mode {
                    BodyMode::Kinematic => RigidBody::Kinematic,
                    BodyMode::Dynamic => RigidBody::Dynamic,
                };

                if *rigid_body != target_mode {
                    commands.entity(*body_entity).insert(target_mode);
                }
            }
        }
    }
}

/// Updates the target positions of bodies in a ragdoll based on the animated pose
pub fn update_ragdolls_avian(
    animation_players: Query<&AnimationGraphPlayer>,
    ragdoll_assets: Res<Assets<Ragdoll>>,
    bone_map_assets: Res<Assets<RagdollBoneMap>>,
    mut relative_kinematic_body_query: Query<&mut RelativeKinematicBodyPositionBased>,
    system_resources: SystemResources,
) {
    for player in &animation_players {
        if let Some(ragdoll_asset_id) = player.ragdoll.as_ref().map(|h| h.id())
            && let Some(ragdoll) = ragdoll_assets.get(ragdoll_asset_id)
            && let Some(spawned_ragdoll) = &player.spawned_ragdoll
            && let Some(bone_map_handle) = &player.ragdoll_bone_map
            && let Some(bone_map) = bone_map_assets.get(bone_map_handle)
            && let Some(pose) = player.get_default_output_pose()
            && let Some(skeleton) = system_resources.skeleton_assets.get(&pose.skeleton)
        {
            let pose_fallback = PoseFallbackContext {
                entity_map: &player.entity_map,
                resources: &system_resources,
                fallback_to_identity: true,
            };
            let rb_targets =
                write_pose_to_ragdoll(pose, skeleton, ragdoll, bone_map, pose_fallback);
            let root_transform = pose_fallback.compute_root_global_transform_to_rigidbody(skeleton);

            for body_target in rb_targets.bodies {
                let Some(body_entity) = spawned_ragdoll.bodies.get(&body_target.body_id) else {
                    continue;
                };

                let Ok(mut relative_kinematic_body) =
                    relative_kinematic_body_query.get_mut(*body_entity)
                else {
                    continue;
                };

                let target = body_target.character_space_isometry;

                match root_transform {
                    RootOffsetResult::FromRigidbody(entity, transform) => {
                        relative_kinematic_body.relative_to = Some(entity);
                        relative_kinematic_body.relative_target = transform.to_isometry() * target;
                    }
                    RootOffsetResult::FromRoot(transform) => {
                        relative_kinematic_body.relative_to = None;
                        relative_kinematic_body.relative_target = transform.to_isometry() * target;
                    }
                    RootOffsetResult::Failed => continue,
                }
            }
        }
    }
}

pub fn read_back_poses_avian(
    mut animation_players: Query<&mut AnimationGraphPlayer>,
    bone_map_assets: Res<Assets<RagdollBoneMap>>,
    pos_query: Query<(&Position, &Rotation)>,
    system_resources: SystemResources,
) {
    for mut player in &mut animation_players {
        if let Some(spawned_ragdoll) = &player.spawned_ragdoll
            && let Some(bone_map_handle) = &player.ragdoll_bone_map
            && let Some(bone_map) = bone_map_assets.get(bone_map_handle)
            && let Some(pose) = player.get_default_output_pose()
            && let Some(skeleton) = system_resources.skeleton_assets.get(&pose.skeleton)
        {
            let config = player
                .get_outputs()
                .get(DEFAULT_OUTPUT_RAGDOLL_CONFIG)
                .and_then(|v| v.as_ragdoll_config().ok())
                .cloned()
                .unwrap_or_default();

            let pose_fallback = PoseFallbackContext {
                entity_map: &player.entity_map,
                resources: &system_resources,
                fallback_to_identity: true,
            };

            let updated_pose = read_pose(
                spawned_ragdoll,
                bone_map,
                skeleton,
                &pos_query,
                pose_fallback,
                pose,
                &config,
            );

            player.set_default_output_pose(updated_pose);
        }
    }
}
