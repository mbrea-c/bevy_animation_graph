use avian3d::prelude::{AngularVelocity, LinearVelocity};
use bevy::asset::Assets;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::query::With;
use bevy::ecs::system::{Commands, Res};
use bevy::ecs::{query::Without, system::Query};
use bevy::math::Vec3;
use bevy::transform::components::GlobalTransform;

use crate::core::ragdoll::definition::Ragdoll;
use crate::core::ragdoll::relative_kinematic_body::RelativeKinematicBody;
use crate::core::ragdoll::spawning::spawn_ragdoll_avian;
use crate::prelude::AnimationGraphPlayer;

pub fn update_relative_kinematic_body_velocities(
    mut relative_kinematic_query: Query<(
        &RelativeKinematicBody,
        &mut LinearVelocity,
        &mut AngularVelocity,
    )>,
    relative_to_query: Query<(&LinearVelocity, &AngularVelocity), Without<RelativeKinematicBody>>,
) {
    for (relative_kinematic_body, mut linvel, mut angvel) in &mut relative_kinematic_query {
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

pub fn update_ragdolls_avian(
    mut animation_players: Query<(&mut AnimationGraphPlayer, &GlobalTransform)>,
    ragdoll_assets: Res<Assets<Ragdoll>>,
) {
    for (mut player, global_transform) in &mut animation_players {
        if let Some(ragdoll_asset_id) = player.ragdoll.as_ref().map(|h| h.id())
            && let Some(ragdoll) = ragdoll_assets.get(ragdoll_asset_id)
            && let Some(spawned_ragdoll) = &player.spawned_ragdoll
            && let Some(pose) = player.get_default_output_pose()
        {
            // let spawned_ragdoll =
            //     spawn_ragdoll_avian(ragdoll, global_transform.to_isometry(), &mut commands);
            // player.spawned_ragdoll = Some(spawned_ragdoll);
        }
    }
}
