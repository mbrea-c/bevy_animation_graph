use std::rc::Rc;

use crate::asset_saving::{SaveFsm, SaveGraph};
use bevy::color::palettes::css::WHITE;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

use bevy::render::view::RenderLayers;
use bevy::utils::HashMap;
use bevy_animation_graph::core::animated_scene::AnimatedSceneInstance;
use bevy_animation_graph::core::animation_graph_player::AnimationGraphPlayer;
use bevy_animation_graph::core::pose::Pose;
use bevy_animation_graph::core::space_conversion::SpaceConversionContext;
use bevy_animation_graph::prelude::{
    AnimationSource, DeferredGizmos, DeferredGizmosContext, PoseFallbackContext, SystemResources,
};
use bevy_inspector_egui::bevy_egui;
use egui_dock::egui;

use super::UiState;

#[derive(Component)]
pub struct PreviewScene;

pub fn asset_save_event_system(
    mut ui_state: ResMut<UiState>,
    mut evw_save_graph: EventWriter<SaveGraph>,
    mut evw_save_fsm: EventWriter<SaveFsm>,
) {
    for save_event in ui_state.graph_save_events.drain(..) {
        evw_save_graph.send(save_event);
    }
    for save_event in ui_state.fsm_save_events.drain(..) {
        evw_save_fsm.send(save_event);
    }
}

pub fn graph_debug_draw_bone_system(
    ui_state: Res<UiState>,
    scene_instance_query: Query<&AnimatedSceneInstance, With<PreviewScene>>,
    mut player_query: Query<&mut AnimationGraphPlayer>,
) {
    let Some(path) = ui_state.selection.entity_path.as_ref() else {
        return;
    };
    if ui_state.selection.scene.is_none() {
        return;
    };
    let Ok(instance) = scene_instance_query.get_single() else {
        return;
    };
    let entity = instance.player_entity();
    let Ok(mut player) = player_query.get_mut(entity) else {
        return;
    };

    player.gizmo_for_bones(vec![path.clone().id()])
}

// Pose gizmo rendering

#[derive(Component)]
#[require(Transform, Visibility, Name)]
pub struct PoseGizmoRender {
    pub pose: Pose,
}

pub fn render_pose_gizmos(
    pose_queries: Query<&PoseGizmoRender>,
    resources: SystemResources,
    mut gizmos: Gizmos,
) {
    let mut deferred_gizmos = DeferredGizmos::default();
    for pose_gizmo_render in &pose_queries {
        let entity_map = HashMap::new();

        let mut ctx = DeferredGizmosContext {
            gizmos: &mut deferred_gizmos,
            resources: &resources,
            entity_map: &entity_map,
            space_conversion: SpaceConversionContext {
                pose_fallback: PoseFallbackContext {
                    entity_map: &entity_map,
                    resources: &resources,
                    fallback_to_identity: true,
                },
            },
        };

        ctx.pose_bone_gizmos(WHITE.into(), &pose_gizmo_render.pose);
    }

    deferred_gizmos.apply(&mut gizmos);
}

#[derive(Component)]
pub struct OverrideSceneAnimation(pub Pose);

#[allow(clippy::type_complexity)]
pub fn override_scene_animations(
    scene_query: Query<
        (&AnimatedSceneInstance, &OverrideSceneAnimation),
        Or<(
            Changed<AnimatedSceneInstance>,
            Changed<OverrideSceneAnimation>,
        )>,
    >,
    mut player_query: Query<&mut AnimationGraphPlayer>,
) {
    for (instance, pose_override) in &scene_query {
        let Ok(mut player) = player_query.get_mut(instance.player_entity()) else {
            continue;
        };
        player.set_animation(AnimationSource::Pose(pose_override.0.clone()));
    }
}

// Below here is the generic textured render tooling

pub fn provide_texture_for_scene<T: SubSceneConfig>(
    world: &mut World,
    id: egui::Id,
    config: T,
) -> Handle<Image> {
    if !world.contains_resource::<SubScenes<T>>() {
        world.insert_resource(SubScenes::<T>::default());
    }

    if !world.contains_resource::<SubSceneLayerManager>() {
        world.insert_resource(SubSceneLayerManager::default());
    }

    let new_config = Rc::new(config);

    let action = world
        .run_system_cached_with(check_sync_action::<T>, (id, new_config.clone()))
        .unwrap();

    match action {
        SubSceneSyncAction::Nothing => {}
        SubSceneSyncAction::Respawn => {
            world
                .run_system_cached_with(cleanup_render_layer::<T>, id)
                .unwrap();

            world
                .run_system_cached_with(setup_textured_render, (id, new_config.clone()))
                .unwrap();

            world.flush();
        }
        SubSceneSyncAction::Update => {
            new_config.update(id, world);
            world
                .run_system_cached_with(update_config, (id, new_config.clone()))
                .unwrap();
        }
    }

    world
        .run_system_cached_with(get_image_handle::<T>, id)
        .unwrap()
}

/// Keeps track of "subscenes" spawned in order to render
/// in a texture shown in the UI.
#[derive(Resource)]
pub struct SubScenes<T> {
    /// The map is from renderlayer to scene data
    scenes: HashMap<egui::Id, SubSceneData<T>>,
    layers: HashMap<egui::Id, usize>,
}

/// Keeps track of layer assignments
#[derive(Resource, Default)]
pub struct SubSceneLayerManager {
    next_available_layer: usize,
}

impl SubSceneLayerManager {
    pub fn assign_layer(&mut self) -> usize {
        let layer = self.next_available_layer;
        self.next_available_layer += 1;

        layer
    }
}

impl<T> Default for SubScenes<T> {
    fn default() -> Self {
        Self {
            scenes: HashMap::default(),
            layers: HashMap::default(),
        }
    }
}

impl<T> SubScenes<T> {
    pub fn add(
        &mut self,
        id: egui::Id,
        data: SubSceneData<T>,
        layer_manager: &mut SubSceneLayerManager,
    ) -> usize {
        let layer = layer_manager.assign_layer();

        self.scenes.insert(id, data);
        self.layers.insert(id, layer);

        layer
    }

    pub fn get_data(&self, id: egui::Id) -> Option<&SubSceneData<T>> {
        self.scenes.get(&id)
    }

    pub fn get_data_mut(&mut self, id: egui::Id) -> Option<&mut SubSceneData<T>> {
        self.scenes.get_mut(&id)
    }

    pub fn remove(&mut self, id: egui::Id) -> Option<SubSceneData<T>> {
        self.layers.remove(&id);
        self.scenes.remove(&id)
    }
}

pub struct SubSceneData<T> {
    config: T,
    image: Handle<Image>,
}

/// Indicates that this entity is part of a subscene with the given render layer
#[derive(Component, Clone)]
pub struct PartOfSubScene(pub egui::Id);

#[derive(Debug, Clone, Copy)]
pub enum SubSceneSyncAction {
    Nothing,
    Respawn,
    Update,
}

pub trait SubSceneConfig: Clone + PartialEq + Send + Sync + 'static {
    fn spawn(&self, builder: &mut ChildBuilder, render_target: Handle<Image>);
    fn sync_action(&self, new_config: &Self) -> SubSceneSyncAction;
    fn update(&self, id: egui::Id, world: &mut World);
}

pub fn setup_textured_render<T: SubSceneConfig>(
    In((widget_id, config)): In<(egui::Id, Rc<T>)>,
    mut egui_user_textures: ResMut<bevy_egui::EguiUserTextures>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut subscenes: ResMut<SubScenes<T>>,
    mut layer_manager: ResMut<SubSceneLayerManager>,
) -> Handle<Image> {
    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    image.resize(size);

    let image_handle = images.add(image);

    egui_user_textures.add_image(image_handle.clone());

    let root = commands
        .spawn((
            Transform::default(),
            Visibility::default(),
            Name::new("Subscene"),
        ))
        .with_children(|c| {
            config.spawn(c, image_handle.clone());
        })
        .id();

    let layer = subscenes.add(
        widget_id,
        SubSceneData {
            config: config.as_ref().clone(),
            image: image_handle.clone(),
        },
        &mut layer_manager,
    );

    commands.entity(root).insert((
        RenderLayers::from_layers(&[layer]),
        PartOfSubScene(widget_id),
    ));

    image_handle
}

pub fn update_config<T: SubSceneConfig>(
    In((widget_id, config)): In<(egui::Id, Rc<T>)>,
    mut subscenes: ResMut<SubScenes<T>>,
) {
    if let Some(data) = subscenes.get_data_mut(widget_id) {
        data.config = config.as_ref().clone();
    }
}

pub fn cleanup_render_layer<T: SubSceneConfig>(
    In(widget_id): In<egui::Id>,
    mut commands: Commands,
    query: Query<(Entity, &PartOfSubScene)>,
    mut egui_user_textures: ResMut<bevy_egui::EguiUserTextures>,
    mut subscenes: ResMut<SubScenes<T>>,
) {
    for (entity, &PartOfSubScene(id)) in &query {
        if widget_id == id {
            commands.entity(entity).despawn_recursive();
        }
    }

    if let Some(config) = subscenes.remove(widget_id) {
        egui_user_textures.remove_image(&config.image);
    }
}

pub fn check_sync_action<T: SubSceneConfig>(
    In((widget_id, config)): In<(egui::Id, Rc<T>)>,
    subscenes: Res<SubScenes<T>>,
) -> SubSceneSyncAction {
    if let Some(data) = subscenes.get_data(widget_id) {
        data.config.sync_action(&config)
    } else {
        SubSceneSyncAction::Respawn
    }
}

pub fn get_image_handle<T: SubSceneConfig>(
    In(widget_id): In<egui::Id>,
    subscenes: Res<SubScenes<T>>,
) -> Handle<Image> {
    subscenes.get_data(widget_id).unwrap().image.clone()
}

#[allow(clippy::type_complexity)]
pub fn propagate_layers(
    layers_query: Query<(Entity, &RenderLayers, &PartOfSubScene), Or<(Changed<Children>,)>>,
    children_query: Query<&Children>,
    mut commands: Commands,
) {
    for (entity, render_layers, part_of_sub_scene) in &layers_query {
        for child in children_query.iter_descendants(entity) {
            commands
                .entity(child)
                .insert((render_layers.clone(), part_of_sub_scene.clone()));
        }
    }
}
