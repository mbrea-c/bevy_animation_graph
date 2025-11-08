use bevy::{
    asset::Handle,
    ecs::hierarchy::ChildSpawnerCommands,
    image::Image,
    light::PointLight,
    math::Vec3,
    prelude::{Transform, World},
};
use bevy_animation_graph::prelude::{
    AnimatedScene, AnimatedSceneHandle, AnimatedSceneInstance, AnimationGraphPlayer,
};
use egui_dock::egui;

use crate::ui::{
    PartOfSubScene, PreviewScene, SubSceneConfig, SubSceneSyncAction,
    global_state::{ClearGlobalState, active_scene::ActiveScene, get_global_state},
    native_windows::{EditorWindowContext, NativeEditorWindowExtension},
    utils::orbit_camera_scene_show,
};

#[derive(Debug, Default)]
pub struct ScenePreviewWindow;

impl NativeEditorWindowExtension for ScenePreviewWindow {
    fn ui(&self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        let Some(active_scene) = get_global_state::<ActiveScene>(world) else {
            return;
        };

        let config = ScenePreviewConfig {
            animated_scene: active_scene.handle.clone(),
        };

        let ui_texture_id = ui.id().with("Scene preview texture");
        let mut query = world.query::<(&AnimatedSceneInstance, &PreviewScene, &PartOfSubScene)>();
        if let Some((instance, _, _)) = query
            .iter(world)
            .find(|(_, _, PartOfSubScene(uid))| *uid == ui_texture_id)
        {
            // Scene playback control will only be shown once the scene is created
            // (so from the second frame onwards)
            let entity = instance.player_entity();
            let mut query = world.query::<&mut AnimationGraphPlayer>();
            let Ok(mut player) = query.get_mut(world, entity) else {
                return;
            };

            ui.horizontal(|ui| {
                if ui.button("X").on_hover_text("Close preview").clicked() {
                    ctx.trigger(ClearGlobalState::<ActiveScene>::default());
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

        orbit_camera_scene_show(&config, ui, world, ui_texture_id);
    }

    fn display_name(&self) -> String {
        "Scene Preview".to_string()
    }
}

#[derive(Clone, PartialEq)]
pub struct ScenePreviewConfig {
    pub animated_scene: Handle<AnimatedScene>,
}

impl SubSceneConfig for ScenePreviewConfig {
    fn spawn(&self, builder: &mut ChildSpawnerCommands, _: Handle<Image>) {
        builder.spawn((
            PointLight::default(),
            Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        ));

        builder.spawn((
            AnimatedSceneHandle::new(self.animated_scene.clone()),
            PreviewScene,
        ));
    }

    fn sync_action(&self, new_config: &Self) -> SubSceneSyncAction {
        match self.animated_scene == new_config.animated_scene {
            true => SubSceneSyncAction::Nothing,
            false => SubSceneSyncAction::Respawn,
        }
    }

    fn update(&self, _id: egui::Id, _world: &mut World) {}
}
