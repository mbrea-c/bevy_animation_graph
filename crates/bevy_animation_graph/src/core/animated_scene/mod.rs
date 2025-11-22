pub mod loader;

use bevy::{
    animation::AnimationTarget,
    asset::{Asset, Assets, Handle, ReflectAsset},
    camera::visibility::Visibility,
    ecs::{
        component::Component,
        entity::Entity,
        hierarchy::Children,
        name::Name,
        observer::On,
        query::{With, Without},
        reflect::AppTypeRegistry,
        system::{Commands, Query, Res, ResMut},
    },
    platform::collections::HashMap,
    reflect::Reflect,
    scene::{Scene, SceneInstanceReady, SceneRoot},
    transform::components::Transform,
};

use super::{animation_clip::EntityPath, errors::AssetLoaderError, id::BoneId, skeleton::Skeleton};
use crate::core::{
    animation_graph::AnimationGraph,
    animation_graph_player::{AnimationGraphPlayer, AnimationSource},
    ragdoll::{bone_mapping::RagdollBoneMap, definition::Ragdoll},
};

#[derive(Clone, Asset, Reflect)]
#[reflect(Asset)]
pub struct AnimatedScene {
    pub source: Handle<Scene>,
    pub processed_scene: Option<Handle<Scene>>,
    pub animation_graph: Handle<AnimationGraph>,
    pub retargeting: Option<Retargeting>,
    /// Skeleton of the animations we want to play on the source scene.
    ///
    /// Usually this will be the source scene's skeleton, but it may differ if we're applying
    /// retargeting.
    pub skeleton: Handle<Skeleton>,
    pub ragdoll: Option<Handle<Ragdoll>>,
    pub ragdoll_bone_map: Option<Handle<RagdollBoneMap>>,
}

/// Configuration needed to apply animation retargeting
#[derive(Clone, Reflect)]
pub struct Retargeting {
    /// *Actual* skeleton of the source scene.
    pub source_skeleton: Handle<Skeleton>,
    /// Allows renaming of individual components of bone paths.
    ///
    /// For example using an override `"bone_a": "bone_b"` will map a path `["parent_bone",
    /// "bone_a", "child_bone"]` to `["parent_bone", "bone_b", "child_bone"]`.
    pub bone_path_overrides: HashMap<String, String>,
}

#[derive(Component)]
pub struct AnimatedSceneInstance {
    player_entity: Entity,
}

impl AnimatedSceneInstance {
    pub fn player_entity(&self) -> Entity {
        self.player_entity
    }
}

#[derive(Component, Default)]
#[require(Transform, Visibility)]
pub struct AnimatedSceneHandle {
    pub handle: Handle<AnimatedScene>,
    pub override_source: Option<AnimationSource>,
}

impl AnimatedSceneHandle {
    pub fn new(handle: Handle<AnimatedScene>) -> Self {
        Self {
            handle,
            override_source: None,
        }
    }
}

/// Processed animated scenes are "cached".
pub(crate) fn spawn_animated_scenes(
    mut commands: Commands,
    unloaded_scenes: Query<(Entity, &AnimatedSceneHandle), Without<SceneRoot>>,
    mut animated_scene_assets: ResMut<Assets<AnimatedScene>>,
    mut scenes: ResMut<Assets<Scene>>,
    skeletons: Res<Assets<Skeleton>>,
    app_type_registry: Res<AppTypeRegistry>,
) {
    for (entity, animscn_handle) in &unloaded_scenes {
        let Some(animscn) = animated_scene_assets.get_mut(&animscn_handle.handle) else {
            continue;
        };

        let processed_scene = if animscn.processed_scene.is_some() {
            animscn.processed_scene.as_ref().unwrap()
        } else if is_scene_ready_to_process(animscn, &scenes, &skeletons) {
            let Some(scene) = scenes
                .get(&animscn.source)
                .and_then(|scn| scn.clone_with(&app_type_registry).ok())
            else {
                continue;
            };

            let scene = process_scene_into_animscn(
                scene,
                animscn.skeleton.clone(),
                animscn.ragdoll.clone(),
                animscn.ragdoll_bone_map.clone(),
                animscn.animation_graph.clone(),
                &skeletons,
                animscn.retargeting.as_ref(),
            )
            .unwrap();

            animscn.processed_scene = Some(scenes.add(scene));
            animscn.processed_scene.as_ref().unwrap()
        } else {
            continue;
        };

        commands
            .entity(entity)
            .insert(SceneRoot(processed_scene.clone()));
    }
}

/// Checks whether the scene can be processed
fn is_scene_ready_to_process(
    animscn: &AnimatedScene,
    scenes: &Assets<Scene>,
    skeletons: &Assets<Skeleton>,
) -> bool {
    scenes.contains(&animscn.source) && skeletons.contains(&animscn.skeleton)
}

/// This function finds the [`bevy::animation::AnimationPlayer`] and replaces it with our own.
///
/// It also applies retargeting if necessary.
#[allow(clippy::result_large_err, clippy::too_many_arguments)]
fn process_scene_into_animscn(
    mut scene: Scene,
    skeleton_handle: Handle<Skeleton>,
    ragdoll_handle: Option<Handle<Ragdoll>>,
    ragdoll_bone_map_handle: Option<Handle<RagdollBoneMap>>,
    graph: Handle<AnimationGraph>,
    skeletons: &Assets<Skeleton>,
    retargeting: Option<&Retargeting>,
) -> Result<Scene, AssetLoaderError> {
    let mut query = scene
        .world
        .query_filtered::<Entity, With<bevy::animation::AnimationPlayer>>();

    let Ok(animation_player) = query.single(&scene.world) else {
        return Err(AssetLoaderError::AnimatedSceneMissingRoot);
    };

    let mut entity_mut = scene.world.entity_mut(animation_player);
    let mut player = AnimationGraphPlayer::new(skeleton_handle.clone()).with_graph(graph);
    player.ragdoll = ragdoll_handle;
    player.ragdoll_bone_map = ragdoll_bone_map_handle;
    entity_mut.remove::<bevy::animation::AnimationPlayer>();
    entity_mut.insert(player);

    if let Some(retargeting) = retargeting
        && let Some(skeleton) = skeletons.get(&retargeting.source_skeleton)
    {
        let player_entity_id = entity_mut.id();

        let mut query = scene.world.query::<&mut AnimationTarget>();

        for mut target in query.iter_mut(&mut scene.world) {
            if player_entity_id != target.player {
                continue;
            }

            let bone_id = BoneId::from(target.id);
            let Some(mapped_bone_id) =
                apply_bone_path_overrides(bone_id, skeleton, &retargeting.bone_path_overrides)
            else {
                continue;
            };
            *target = AnimationTarget {
                id: bevy::animation::AnimationTargetId(mapped_bone_id.id()),
                player: target.player,
            }
        }
    }

    Ok(scene)
}

fn apply_bone_path_overrides(
    bone_id: BoneId,
    skeleton: &Skeleton,
    mappings: &HashMap<String, String>,
) -> Option<BoneId> {
    let path = EntityPath {
        parts: skeleton
            .id_to_path(bone_id)?
            .parts
            .into_iter()
            .map(|p| {
                if let Some(s) = mappings.get(p.as_str()) {
                    Name::new(s.clone())
                } else {
                    p
                }
            })
            .collect(),
    };

    Some(path.id())
}

/// Adds an `AnimatedSceneInstance` pointing to the animation graph player when the scene is
/// spawned
pub(crate) fn locate_animated_scene_player(
    trigger: On<SceneInstanceReady>,
    handle_query: Query<&AnimatedSceneHandle>,
    mut player_query: Query<&mut AnimationGraphPlayer>,
    children_query: Query<&Children>,
    mut commands: Commands,
) {
    let root_entity = trigger.entity;

    let Ok(animscn_handle) = handle_query.get(root_entity) else {
        return;
    };

    for child in children_query.iter_descendants(root_entity) {
        let Ok(mut player) = player_query.get_mut(child) else {
            continue;
        };

        if let Some(override_source) = animscn_handle.override_source.clone() {
            player.set_animation(override_source);
        }

        commands.entity(root_entity).insert(AnimatedSceneInstance {
            player_entity: child,
        });
        return;
    }
}
