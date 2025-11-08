use super::{
    animation_clip::EntityPath, animation_graph::TimeUpdate,
    animation_graph_player::AnimationGraphPlayer, pose::BoneId, prelude::PlaybackState,
};
use crate::prelude::SystemResources;
use bevy::{
    ecs::prelude::*, gizmos::gizmos::Gizmos, log::info_span, mesh::morph::MorphWeights,
    platform::collections::HashMap, time::prelude::*, transform::prelude::*,
};
use std::collections::VecDeque;

fn build_entity_map(root_entity: Entity, resources: &SystemResources) -> HashMap<BoneId, Entity> {
    let mut entity_map = HashMap::default();

    let root_name = resources.names_query.get(root_entity).unwrap();
    let root_path = EntityPath {
        parts: vec![root_name.clone()],
    };

    entity_map.insert(root_path.id(), root_entity);

    let root_children = resources.children_query.get(root_entity).unwrap();

    let mut queue: VecDeque<(Entity, EntityPath)> = VecDeque::new();

    for child in root_children {
        queue.push_back((*child, root_path.clone()));
    }

    while !queue.is_empty() {
        let (entity, parent_path) = queue.pop_front().unwrap();
        let Ok(name) = resources.names_query.get(entity) else {
            continue;
        };
        let path = parent_path.child(name.clone());
        entity_map.insert(path.id(), entity);

        if let Ok(children) = resources.children_query.get(entity) {
            for child in children {
                queue.push_back((*child, path.clone()));
            }
        }
    }

    entity_map
}

/// System that will play all animations, using any entity with a [`AnimationGraphPlayer`]
/// and a [`Handle<AnimationClip>`] as an animation root
#[allow(clippy::too_many_arguments)]
pub fn animation_player(
    time: Res<Time>,
    mut animation_players: Query<(Entity, &mut AnimationGraphPlayer)>,
    sysres: SystemResources,
) {
    animation_players.par_iter_mut().for_each(|(root, player)| {
        run_animation_player(root, player, &time, &sysres);
    });
    animation_players.par_iter_mut().for_each(|(root, player)| {
        debug_draw_animation_players(player, root, &sysres);
    });
}

/// System that will draw deferred gizmo commands called during graph evaluation
#[allow(clippy::too_many_arguments)]
pub fn animation_player_deferred_gizmos(
    mut animation_players: Query<&mut AnimationGraphPlayer>,
    mut gizmos: Gizmos,
) {
    for mut player in &mut animation_players {
        player.deferred_gizmos.apply(&mut gizmos);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn run_animation_player(
    root: Entity,
    mut player: Mut<AnimationGraphPlayer>,
    time: &Time,
    system_resources: &SystemResources,
) {
    let _run_animation_player_span = info_span!("run_animation_player").entered();

    // The entity map is updated regardless of animation type.
    // Important as the entity map is relied upon for e.g. debug bone drawing
    {
        let _entity_map_span = info_span!("build_entity_map").entered();
        player.entity_map = build_entity_map(root, system_resources);
    }

    if !player.animation.is_graph() {
        return;
    }

    if !player.is_paused() {
        player.queue_time_update(TimeUpdate::Delta(time.delta_secs()));
    }

    {
        let _update_span = info_span!("player_update").entered();
        player.update(system_resources, root);
    }

    if matches!(player.playback_state(), PlaybackState::PlayOneFrame) {
        player.pause();
    }
}

pub fn apply_animation_to_targets(
    mut animation_targets: Query<(
        &mut Transform,
        Option<&mut MorphWeights>,
        &bevy::animation::AnimationTarget,
    )>,
    graph_players: Query<&AnimationGraphPlayer>,
) {
    for (mut target_transform, target_morphs, target) in &mut animation_targets {
        let target_bone_id = BoneId::from(target.id);
        let Ok(player) = graph_players.get(target.player) else {
            continue;
        };
        let Some(pose) = player.get_default_output_pose() else {
            continue;
        };

        let Some(bone_index) = pose.paths.get(&target_bone_id) else {
            continue;
        };

        let bone_pose = &pose.bones[*bone_index];
        if let Some(rotation) = bone_pose.rotation {
            target_transform.rotation = rotation;
        }
        if let Some(translation) = bone_pose.translation {
            target_transform.translation = translation;
        }
        if let Some(scale) = bone_pose.scale {
            target_transform.scale = scale;
        }
        if let Some(weights) = &bone_pose.weights
            && let Some(mut morphs) = target_morphs
        {
            apply_morph_weights(morphs.weights_mut(), weights);
        }
    }
}

pub fn debug_draw_animation_players(
    mut player: Mut<AnimationGraphPlayer>,
    root_entity: Entity,
    system_resources: &SystemResources,
) {
    player.debug_draw_bones(system_resources, root_entity);
}

/// Update `weights` based on weights in `keyframe` with a linear interpolation
/// on `key_lerp`.
fn apply_morph_weights(weights: &mut [f32], new_weights: &[f32]) {
    let zipped = weights.iter_mut().zip(new_weights);
    for (morph_weight, keyframe) in zipped {
        *morph_weight = *keyframe;
    }
}
