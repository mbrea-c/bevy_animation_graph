use super::{
    animation_clip::EntityPath,
    animation_graph::{TimeUpdate, UpdateTime},
    animation_graph_player::AnimationGraphPlayer,
    pose::{BoneId, Pose},
};
use crate::prelude::SystemResources;
use bevy::{
    core::prelude::*, ecs::prelude::*, gizmos::gizmos::Gizmos, hierarchy::prelude::*,
    log::prelude::*, render::mesh::morph::MorphWeights, time::prelude::*, transform::prelude::*,
    utils::HashMap,
};
use std::{collections::VecDeque, ops::Deref};

fn entity_from_path(
    root: Entity,
    path: &EntityPath,
    children: &Query<&Children>,
    names: &Query<&Name>,
) -> Option<Entity> {
    // PERF: finding the target entity can be optimised
    let mut current_entity = root;

    let mut parts = path.parts.iter().enumerate();

    // check the first name is the root node which we already have
    let Some((_, root_name)) = parts.next() else {
        return None;
    };
    if names.get(current_entity) != Ok(root_name) {
        return None;
    }

    for (_idx, part) in parts {
        let mut found = false;
        let children = children.get(current_entity).ok()?;
        if !found {
            for child in children.deref() {
                if let Ok(name) = names.get(*child) {
                    if name == part {
                        // Found a children with the right name, continue to the next part
                        current_entity = *child;
                        found = true;
                        break;
                    }
                }
            }
        }
        if !found {
            warn!("Entity not found for path {:?} on part {:?}", path, part);
            return None;
        }
    }
    Some(current_entity)
}

/// Verify that there are no ancestors of a given entity that have an [`AnimationPlayer`].
fn verify_no_ancestor_player(
    player_parent: Option<&Parent>,
    parents: &Query<(Has<AnimationGraphPlayer>, Option<&Parent>)>,
) -> bool {
    let Some(mut current) = player_parent.map(Parent::get) else {
        return true;
    };
    loop {
        let Ok((has_player, parent)) = parents.get(current) else {
            return true;
        };
        if has_player {
            return false;
        }
        if let Some(parent) = parent {
            current = parent.get();
        } else {
            return true;
        }
    }
}

fn build_entity_map(root_entity: Entity, resources: &SystemResources) -> HashMap<BoneId, Entity> {
    let mut entity_map = HashMap::default();

    let root_name = resources.names_query.get(root_entity).unwrap();
    let root_path: BoneId = EntityPath {
        parts: vec![root_name.clone()],
    };

    entity_map.insert(root_path.clone(), root_entity);

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
        entity_map.insert(path.clone(), entity);

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
    morphs: Query<&mut MorphWeights>,
    parents: Query<(Has<AnimationGraphPlayer>, Option<&Parent>)>,
    mut animation_players: Query<(Entity, Option<&Parent>, &mut AnimationGraphPlayer)>,
    sysres: SystemResources,
) {
    animation_players
        .par_iter_mut()
        .for_each(|(root, maybe_parent, player)| {
            run_animation_player(
                root,
                player,
                &time,
                &morphs,
                maybe_parent,
                &parents,
                &sysres,
            );
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
    morphs: &Query<&mut MorphWeights>,
    maybe_parent: Option<&Parent>,
    parents: &Query<(Has<AnimationGraphPlayer>, Option<&Parent>)>,
    system_resources: &SystemResources,
) {
    // Continue if paused unless the `AnimationPlayer` was changed
    // This allow the animation to still be updated if the player.elapsed field was manually updated in pause
    if player.paused || player.animation.is_none() {
        return;
    }

    player.elapsed = player
        .elapsed
        .update(TimeUpdate::Delta(time.delta_seconds()))
        .update(player.pending_update);
    player.pending_update = None;

    let entity_map = build_entity_map(root, system_resources);

    let Some(out_pose) = player.query(system_resources, root, &entity_map) else {
        return;
    };

    // Apply the main animation
    apply_pose(
        &out_pose,
        root,
        &system_resources.names_query,
        &system_resources.transform_query,
        morphs,
        maybe_parent,
        parents,
        &system_resources.children_query,
    );
}

/// Update `weights` based on weights in `keyframe` with a linear interpolation
/// on `key_lerp`.
fn apply_morph_weights(weights: &mut [f32], new_weights: &[f32]) {
    let zipped = weights.iter_mut().zip(new_weights);
    for (morph_weight, keyframe) in zipped {
        *morph_weight = *keyframe;
    }
}

/// Extract a keyframe from a list of keyframes by index.
///
/// # Panics
///
/// When `key_index * target_count` is larger than `keyframes`
///
/// This happens when `keyframes` is not formatted as described in
/// [`Keyframes::Weights`]. A possible cause is [`AnimationClip`] not being
/// meant to be used for the [`MorphWeights`] of the entity it's being applied to.
pub(crate) fn get_keyframe(target_count: usize, keyframes: &[f32], key_index: usize) -> &[f32] {
    let start = target_count * key_index;
    let end = target_count * (key_index + 1);
    &keyframes[start..end]
}

#[allow(clippy::too_many_arguments)]
fn apply_pose(
    animation_pose: &Pose,
    root: Entity,
    names: &Query<&Name>,
    transforms: &Query<(&mut Transform, &GlobalTransform)>,
    morphs: &Query<&mut MorphWeights>,
    maybe_parent: Option<&Parent>,
    parents: &Query<(Has<AnimationGraphPlayer>, Option<&Parent>)>,
    children: &Query<&Children>,
) {
    if !verify_no_ancestor_player(maybe_parent, parents) {
        warn!("Animation player on {:?} has a conflicting animation player on an ancestor. Cannot safely animate.", root);
        return;
    }

    let mut any_path_found = false;
    for (path, bone_id) in &animation_pose.paths {
        let Some(target) = entity_from_path(root, path, children, names) else {
            continue;
        };
        any_path_found = true;
        // SAFETY: The verify_no_ancestor_player check above ensures that two animation players cannot alias
        // any of their descendant Transforms.
        //
        // The system scheduler prevents any other system from mutating Transforms at the same time,
        // so the only way this fetch can alias is if two AnimationPlayers are targeting the same bone.
        // This can only happen if there are two or more AnimationPlayers are ancestors to the same
        // entities. By verifying that there is no other AnimationPlayer in the ancestors of a
        // running AnimationPlayer before animating any entity, this fetch cannot alias.
        //
        // This means only the AnimationPlayers closest to the root of the hierarchy will be able
        // to run their animation. Any players in the children or descendants will log a warning
        // and do nothing.
        let Ok((mut transform, _)) = (unsafe { transforms.get_unchecked(target) }) else {
            continue;
        };

        let pose = &animation_pose.bones[*bone_id];
        let mut morphs = unsafe { morphs.get_unchecked(target) };
        if let Some(rotation) = pose.rotation {
            transform.rotation = rotation;
        }
        if let Some(translation) = pose.translation {
            transform.translation = translation;
        }
        if let Some(scale) = pose.scale {
            transform.scale = scale;
        }
        if let Some(weights) = &pose.weights {
            if let Ok(morphs) = &mut morphs {
                apply_morph_weights(morphs.weights_mut(), weights);
            }
        }
    }

    if !any_path_found {
        warn!("Animation player on {root:?} did not match any entity paths.");
    }
}
