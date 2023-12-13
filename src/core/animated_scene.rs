use crate::{
    prelude::{AnimationGraph, AnimationGraphPlayer},
    utils::asset_loader_error::AssetLoaderError,
};
use bevy::{
    asset::{io::Reader, Asset, AssetLoader, AsyncReadExt, Handle, LoadContext},
    core::Name,
    ecs::{bundle::Bundle, entity::Entity, query::Without},
    hierarchy::Children,
    prelude::*,
    reflect::Reflect,
    render::view::{InheritedVisibility, ViewVisibility, Visibility},
    scene::{Scene, SceneInstance},
    transform::components::{GlobalTransform, Transform},
    utils::BoxedFuture,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct AnimatedSceneSerial {
    source: String,
    path_to_player: Vec<String>,
    animation_graph: String,
}

#[derive(Clone, Asset, Reflect)]
pub struct AnimatedScene {
    source: Handle<Scene>,
    path_to_player: Vec<String>,
    animation_graph: Handle<AnimationGraph>,
}

#[derive(Component)]
pub struct AnimatedSceneInstance;

#[derive(Component)]
pub struct AnimatedSceneFailed;

#[derive(Bundle, Default)]
pub struct AnimatedSceneBundle {
    pub animated_scene: Handle<AnimatedScene>,
    /// Transform of the scene root entity.
    pub transform: Transform,
    /// Global transform of the scene root entity.
    pub global_transform: GlobalTransform,
    /// User-driven visibility of the scene root entity.
    pub visibility: Visibility,
    /// Inherited visibility of the scene root entity.
    pub inherited_visibility: InheritedVisibility,
    /// Algorithmically-computed visibility of the scene root entity for rendering.
    pub view_visibility: ViewVisibility,
}

#[derive(Default)]
pub struct AnimatedSceneLoader;

impl AssetLoader for AnimatedSceneLoader {
    type Asset = AnimatedScene;
    type Settings = ();
    type Error = AssetLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = vec![];
            reader.read_to_end(&mut bytes).await?;
            let serial: AnimatedSceneSerial = ron::de::from_bytes(&bytes)?;

            let animation_graph: Handle<AnimationGraph> = load_context.load(serial.animation_graph);
            let source: Handle<Scene> = load_context.load(serial.source);

            Ok(AnimatedScene {
                source,
                path_to_player: serial.path_to_player,
                animation_graph,
            })

            // let source_path = AssetPath::from(serial.source.clone());
            // let asset = load_context.load_direct(serial.source.clone()).await?;

            // let mut source_scene: Scene = if let Some(label) = source_path.label_cow() {
            //     let scene: &Scene = asset.get_labeled(label).unwrap().get().unwrap();
            //     // Doesn't seem like I can `.take()` a labeled asset :(
            //     let my_scene: Scene = unsafe { std::mem::transmute_copy(scene) };
            //     my_scene
            // } else {
            //     asset.take().unwrap()
            // };
            // let mut query = source_scene
            //     .world
            //     .query_filtered::<(Entity, &Name), Without<Parent>>();

            // let mut remaining_path = serial.path_to_player.clone();

            // let root_name = remaining_path.remove(0);
            // let mut root_entity: Option<Entity> = None;

            // for (entity, name) in query.iter(&source_scene.world) {
            //     if name.to_string() == root_name {
            //         root_entity = Some(entity);
            //         break;
            //     }
            // }

            // if root_entity.is_none() {
            //     return Err(AssetLoaderError::AnimatedSceneMissingName(
            //         serial.path_to_player.join("/"),
            //     ));
            // }

            // let root_entity = root_entity.unwrap();

            // let mut query = source_scene.world.query::<(Entity, &Children, &Name)>();
            // let mut name_query = source_scene.world.query::<&Name>();

            // let mut next_entity = root_entity;

            // while !remaining_path.is_empty() {
            //     let name = remaining_path.remove(0);
            //     let Ok((_, children, _)) = query.get(&source_scene.world, root_entity) else {
            //         return Err(AssetLoaderError::AnimatedSceneMissingName(
            //             serial.path_to_player.join("/"),
            //         ));
            //     };
            //     let mut found = false;
            //     for child in children.iter() {
            //         let Ok(child_name) = name_query.get(&source_scene.world, *child) else {
            //             return Err(AssetLoaderError::AnimatedSceneMissingName(
            //                 serial.path_to_player.join("/"),
            //             ));
            //         };

            //         if child_name.to_string() == name {
            //             next_entity = *child;
            //             found = true;
            //             break;
            //         }
            //     }
            //     if !found {
            //         return Err(AssetLoaderError::AnimatedSceneMissingName(
            //             serial.path_to_player.join("/"),
            //         ));
            //     }
            // }

            // source_scene
            //     .world
            //     .entity_mut(next_entity)
            //     .insert(AnimationGraphPlayer {
            //         animation: Some(animation_graph),
            //         ..Default::default()
            //     });

            // Ok(source_scene)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["animscn.ron"]
    }
}

pub(crate) fn spawn_animated_scenes(
    mut commands: Commands,
    unloaded_scenes: Query<(Entity, &Handle<AnimatedScene>), Without<Handle<Scene>>>,
    animated_scene_assets: Res<Assets<AnimatedScene>>,
) {
    for (entity, animscn_handle) in &unloaded_scenes {
        let Some(animscn) = animated_scene_assets.get(animscn_handle) else {
            continue;
        };

        commands.entity(entity).insert(animscn.source.clone());
    }
}

pub(crate) fn process_animated_scenes(
    mut commands: Commands,
    unloaded_scenes: Query<
        (Entity, &Handle<AnimatedScene>, &SceneInstance),
        (Without<AnimatedSceneInstance>, Without<AnimatedSceneFailed>),
    >,
    scene_spawner: Res<SceneSpawner>,
    animated_scene_assets: Res<Assets<AnimatedScene>>,
    parent_query: Query<&Parent>,
    query: Query<(Entity, &Children, Option<&Name>)>,
) {
    'outer: for (animscn_entity, animscn_handle, scn_instance) in &unloaded_scenes {
        if !scene_spawner.instance_is_ready(**scn_instance) {
            continue;
        }

        let Some(animscn) = animated_scene_assets.get(animscn_handle) else {
            continue;
        };

        let mut remaining_path = animscn.path_to_player.clone();

        let mut root_entity: Option<Entity> = None;
        // The root entity is the only child of the current entity that is part of the scene
        // instance
        for entity in scene_spawner.iter_instance_entities(**scn_instance) {
            if let Ok(parent) = parent_query.get(entity) {
                if parent.get() == animscn_entity {
                    root_entity = Some(entity);
                }
            }
        }

        let Some(root_entity) = root_entity else {
            error!("Animated scene missing root entity");
            commands.entity(animscn_entity).insert(AnimatedSceneFailed);
            continue;
        };

        let mut next_entity = root_entity;
        while !remaining_path.is_empty() {
            let name = remaining_path.remove(0);

            let Ok((_, children, _)) = query.get(next_entity) else {
                error!(
                    "Animated scene missing entity at path: {:?}",
                    animscn.path_to_player
                );
                commands.entity(animscn_entity).insert(AnimatedSceneFailed);
                continue 'outer;
            };

            let mut found = false;
            for child in children.iter() {
                let Ok((_, _, Some(child_name))) = query.get(*child) else {
                    error!(
                        "Animated scene missing entity at path: {:?}",
                        animscn.path_to_player
                    );
                    commands.entity(animscn_entity).insert(AnimatedSceneFailed);
                    continue 'outer;
                };

                if child_name.to_string() == name {
                    next_entity = *child;
                    found = true;
                    break;
                }
            }
            if !found {
                error!(
                    "Animated scene missing entity at path: {:?}",
                    animscn.path_to_player
                );
                commands.entity(animscn_entity).insert(AnimatedSceneFailed);
                continue 'outer;
            }
        }

        commands
            .entity(next_entity)
            .remove::<AnimationPlayer>()
            .insert(AnimationGraphPlayer {
                animation: Some(animscn.animation_graph.clone()),
                ..Default::default()
            });
        commands
            .entity(animscn_entity)
            .insert(AnimatedSceneInstance);
    }
}
