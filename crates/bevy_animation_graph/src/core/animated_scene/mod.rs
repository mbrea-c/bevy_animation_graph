pub mod loader;

use super::{animation_clip::EntityPath, errors::AssetLoaderError, id::BoneId, skeleton::Skeleton};
use crate::prelude::{AnimationGraph, AnimationGraphPlayer};
use bevy::{
    animation::AnimationTarget,
    asset::{Asset, Handle, ReflectAsset},
    ecs::{entity::Entity, query::Without},
    prelude::*,
    reflect::Reflect,
    render::view::Visibility,
    scene::{Scene, SceneInstance, SceneInstanceReady},
    transform::components::Transform,
    utils::HashMap,
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
pub struct AnimatedSceneHandle(pub Handle<AnimatedScene>);

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
        let Some(animscn) = animated_scene_assets.get_mut(&animscn_handle.0) else {
            continue;
        };

        let processed_scene = if animscn.processed_scene.is_some() {
            animscn.processed_scene.as_ref().unwrap()
        } else {
            let Some(scene) = scenes
                .get(&animscn.source)
                .and_then(|scn| scn.clone_with(&app_type_registry).ok())
            else {
                continue;
            };

            let scene = process_scene_into_animscn(
                scene,
                animscn.skeleton.clone(),
                animscn.animation_graph.clone(),
                &skeletons,
                animscn.retargeting.as_ref(),
            )
            .unwrap();

            animscn.processed_scene = Some(scenes.add(scene));
            animscn.processed_scene.as_ref().unwrap()
        };

        commands
            .entity(entity)
            .insert(SceneRoot(processed_scene.clone()));
    }
}

/// This function finds the [`bevy::animation::AnimationPlayer`] and replaces it with our own.
///
/// It also applies retargeting if necessary.
#[allow(clippy::result_large_err)]
fn process_scene_into_animscn(
    mut scene: Scene,
    skeleton_handle: Handle<Skeleton>,
    graph: Handle<AnimationGraph>,
    skeletons: &Assets<Skeleton>,
    retargeting: Option<&Retargeting>,
) -> Result<Scene, AssetLoaderError> {
    let mut query = scene
        .world
        .query_filtered::<Entity, With<bevy::animation::AnimationPlayer>>();

    let Ok(animation_player) = query.get_single(&scene.world) else {
        return Err(AssetLoaderError::AnimatedSceneMissingRoot);
    };

    let mut entity_mut = scene.world.entity_mut(animation_player);

    entity_mut.remove::<bevy::animation::AnimationPlayer>();
    entity_mut.insert(AnimationGraphPlayer::new(skeleton_handle).with_graph(graph));

    if let Some(retargeting) = retargeting {
        if let Some(skeleton) = skeletons.get(&retargeting.source_skeleton) {
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
    trigger: Trigger<SceneInstanceReady>,
    scene_spawner: Res<SceneSpawner>,
    root_query: Query<&SceneInstance, With<AnimatedSceneHandle>>,
    player_query: Query<(), With<AnimationGraphPlayer>>,
    mut commands: Commands,
) {
    let root_entity = trigger.entity();
    let Ok(scene_instance) = root_query.get(root_entity) else {
        return;
    };

    for child in scene_spawner.iter_instance_entities(**scene_instance) {
        let Ok(_) = player_query.get(child) else {
            continue;
        };

        commands.entity(root_entity).insert(AnimatedSceneInstance {
            player_entity: child,
        });
        return;
    }
}
