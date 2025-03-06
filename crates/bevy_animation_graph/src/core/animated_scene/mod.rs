pub mod loader;

use super::{errors::AssetLoaderError, skeleton::Skeleton};
use crate::prelude::{AnimationGraph, AnimationGraphPlayer};
use bevy::{
    asset::{Asset, Handle, ReflectAsset},
    ecs::{entity::Entity, query::Without},
    prelude::*,
    reflect::Reflect,
    render::view::Visibility,
    scene::{Scene, SceneInstance, SceneInstanceReady},
    transform::components::Transform,
};

#[derive(Clone, Asset, Reflect)]
#[reflect(Asset)]
pub struct AnimatedScene {
    pub(crate) source: Handle<Scene>,
    pub(crate) processed_scene: Option<Handle<Scene>>,
    pub(crate) path_to_player: Vec<String>,
    pub(crate) animation_graph: Handle<AnimationGraph>,
    pub(crate) skeleton: Handle<Skeleton>,
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
#[allow(clippy::result_large_err)]
fn process_scene_into_animscn(
    mut scene: Scene,
    skeleton: Handle<Skeleton>,
    graph: Handle<AnimationGraph>,
) -> Result<Scene, AssetLoaderError> {
    let mut query = scene
        .world
        .query_filtered::<Entity, With<bevy::animation::AnimationPlayer>>();

    let Ok(animation_player) = query.get_single(&scene.world) else {
        return Err(AssetLoaderError::AnimatedSceneMissingRoot);
    };

    let mut entity_mut = scene.world.entity_mut(animation_player);

    entity_mut.remove::<bevy::animation::AnimationPlayer>();
    entity_mut.insert(AnimationGraphPlayer::new(skeleton).with_graph(graph));

    Ok(scene)
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
