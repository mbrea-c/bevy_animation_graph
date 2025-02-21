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
    AnimatedScene, AnimatedSceneHandle, AnimatedSceneInstance, AnimationGraphPlayer,
};
use egui_dock::egui;

use crate::ui::{
    core::EditorWindowExtension,
    utils::{orbit_camera_scene_show, orbit_camera_transform, orbit_camera_update, OrbitView},
    PreviewScene, SubSceneConfig, SubSceneSyncAction,
};

#[derive(Debug, Default)]
pub struct ScenePreviewWindow {
    pub orbit_view: OrbitView,
}

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
            view: self.orbit_view.clone(),
        };

        let mut query = world.query::<(&AnimatedSceneInstance, &PreviewScene)>();
        if let Ok((instance, _)) = query.get_single(world) {
            // Scene playback control will only be shown once the scene is created
            // (so from the second frame onwards)
            let entity = instance.player_entity();
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
        }

        orbit_camera_scene_show(&config, &mut self.orbit_view, ui, world, ui.id());
    }

    fn display_name(&self) -> String {
        "Scene Preview".to_string()
    }
}

#[derive(Clone, PartialEq)]
pub struct ScenePreviewConfig {
    animated_scene: Handle<AnimatedScene>,
    view: OrbitView,
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
            orbit_camera_transform(&self.view),
        ));

        builder.spawn((
            AnimatedSceneHandle(self.animated_scene.clone()),
            PreviewScene,
        ));
    }

    fn sync_action(&self, new_config: &Self) -> SubSceneSyncAction {
        match (
            self.animated_scene == new_config.animated_scene,
            self.view == new_config.view,
        ) {
            (true, true) => SubSceneSyncAction::Nothing,
            (true, false) => SubSceneSyncAction::Update,
            (false, _) => SubSceneSyncAction::Respawn,
        }
    }

    fn update(&self, id: egui::Id, world: &mut World) {
        world
            .run_system_cached_with(orbit_camera_update, (id, self.view.clone()))
            .unwrap();
    }
}
