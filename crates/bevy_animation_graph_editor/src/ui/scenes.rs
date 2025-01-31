use crate::asset_saving::{SaveFsm, SaveGraph};
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy_animation_graph::core::animated_scene::{AnimatedSceneBundle, AnimatedSceneInstance};
use bevy_animation_graph::core::animation_graph_player::AnimationGraphPlayer;
use bevy_animation_graph::prelude::AnimatedSceneHandle;
use bevy_inspector_egui::bevy_egui;

use super::UiState;

#[derive(Component)]
pub struct PreviewScene;

pub fn scene_spawner_system(
    mut commands: Commands,
    mut query: Query<(Entity, &AnimatedSceneHandle), With<PreviewScene>>,
    mut ui_state: ResMut<UiState>,
) {
    if let Ok((entity, AnimatedSceneHandle(scene_handle))) = query.get_single_mut() {
        if let Some(scene_selection) = &mut ui_state.selection.scene {
            if scene_selection.respawn || &scene_selection.scene != scene_handle {
                commands.entity(entity).despawn_recursive();
                commands
                    .spawn(AnimatedSceneBundle {
                        animated_scene: AnimatedSceneHandle(scene_selection.scene.clone()),
                        ..default()
                    })
                    .insert(PreviewScene);
                scene_selection.respawn = false;
            }
        } else {
            commands.entity(entity).despawn_recursive();
        }
    } else if let Some(scene_selection) = &mut ui_state.selection.scene {
        commands
            .spawn(AnimatedSceneBundle {
                animated_scene: AnimatedSceneHandle(scene_selection.scene.clone()),
                ..default()
            })
            .insert(PreviewScene);
        scene_selection.respawn = false;
    }
}

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
    let entity = instance.player_entity;
    let Ok(mut player) = player_query.get_mut(entity) else {
        return;
    };

    player.gizmo_for_bones(vec![path.clone().id()])
}

pub fn setup_system(
    mut egui_user_textures: ResMut<bevy_egui::EguiUserTextures>,
    mut ui_state: ResMut<UiState>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
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
    ui_state.preview_image = image_handle.clone();

    // Light
    // NOTE: Currently lights are shared between passes - see https://github.com/bevyengine/bevy/issues/3462
    commands.spawn((
        PointLight::default(),
        Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
    ));

    commands.spawn((
        Camera3d::default(),
        Camera {
            // render before the "main pass" camera
            order: -1,
            clear_color: ClearColorConfig::Custom(Color::from(LinearRgba::new(1.0, 1.0, 1.0, 0.0))),
            target: RenderTarget::Image(image_handle),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 2.0, 3.0)).looking_at(Vec3::Y, Vec3::Y),
    ));
}
