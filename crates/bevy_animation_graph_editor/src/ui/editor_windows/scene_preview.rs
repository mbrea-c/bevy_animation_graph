use bevy::{
    asset::Handle,
    color::{Color, LinearRgba},
    image::Image,
    math::Vec3,
    pbr::PointLight,
    prelude::{Camera, Camera3d, ChildBuild, ChildBuilder, ClearColorConfig, Transform, World},
    render::camera::RenderTarget,
    utils::default,
};
use bevy_animation_graph::prelude::{
    AnimatedScene, AnimatedSceneBundle, AnimatedSceneHandle, AnimatedSceneInstance,
    AnimationGraphPlayer,
};
use egui_dock::egui;

use crate::ui::{
    core::EditorWindowExtension, provide_texture_for_scene, utils::render_image, PreviewScene,
    SubSceneConfig, SubSceneSyncAction,
};

#[derive(Debug)]
pub struct ScenePreviewWindow;

impl EditorWindowExtension for ScenePreviewWindow {
    fn ui(
        &mut self,
        ui: &mut egui_dock::egui::Ui,
        world: &mut bevy::prelude::World,
        ctx: &mut crate::ui::core::EditorContext,
    ) {
        let Some(scene_selection) = &ctx.selection.scene else {
            return;
        };

        let config = ScenePreviewConfig {
            animated_scene: scene_selection.scene.clone(),
        };

        // First get the texture to make sure the scene is spawned if needed
        let texture = provide_texture_for_scene(world, ui.id(), config);

        // Now we handle button presses and such
        let mut query = world.query::<(&AnimatedSceneInstance, &PreviewScene)>();
        let Ok((instance, _)) = query.get_single(world) else {
            return;
        };
        let entity = instance.player_entity;
        let mut query = world.query::<&mut AnimationGraphPlayer>();
        let Ok(mut player) = query.get_mut(world, entity) else {
            return;
        };

        ui.horizontal(|ui| {
            if ui.button("X").on_hover_text("Close preview").clicked() {
                ctx.selection.scene = None;
            }

            if ui.button("||").on_hover_text("Pause").clicked() {
                player.pause()
            }

            if ui.button(">").on_hover_text("Play").clicked() {
                player.resume()
            }

            if ui.button("||>").on_hover_text("Play one frame").clicked() {
                player.play_one_frame()
            }
        });

        // Finally render the actual scene texture
        render_image(ui, world, &texture);
    }

    fn display_name(&self) -> String {
        "Scene Preview".to_string()
    }
}

#[derive(Clone, PartialEq)]
pub struct ScenePreviewConfig {
    animated_scene: Handle<AnimatedScene>,
}

impl SubSceneConfig for ScenePreviewConfig {
    fn spawn(&self, builder: &mut ChildBuilder, render_target: Handle<Image>) {
        builder.spawn((
            PointLight::default(),
            Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        ));

        builder.spawn((
            Camera3d::default(),
            Camera {
                // render before the "main pass" camera
                order: -1,
                clear_color: ClearColorConfig::Custom(Color::from(LinearRgba::new(
                    1.0, 1.0, 1.0, 0.0,
                ))),
                target: RenderTarget::Image(render_target),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 2.0, 3.0)).looking_at(Vec3::Y, Vec3::Y),
        ));

        builder.spawn((
            AnimatedSceneBundle {
                animated_scene: AnimatedSceneHandle(self.animated_scene.clone()),
                ..default()
            },
            PreviewScene,
        ));
    }

    fn sync_action(&self, new_config: &Self) -> SubSceneSyncAction {
        if self.animated_scene != new_config.animated_scene {
            SubSceneSyncAction::Respawn
        } else {
            SubSceneSyncAction::Nothing
        }
    }

    fn update(&self, _id: egui::Id, _world: &mut World) {
        unreachable!()
    }
}
