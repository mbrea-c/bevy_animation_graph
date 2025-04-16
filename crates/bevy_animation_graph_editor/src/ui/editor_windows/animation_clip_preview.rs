use bevy::{asset::Handle, prelude::World};
use bevy_animation_graph::{
    nodes::EventMarkupNode,
    prelude::{AnimatedScene, AnimatedSceneInstance, AnimationGraphPlayer},
};
use egui_dock::egui;

use crate::ui::{
    actions::{
        clip_preview::{
            ClipPreviewScenes, CreateClipPreview, CreateTrackNodePreview, NodePreviewKey,
            NodePreviewScenes,
        },
        window::DynWindowAction,
    },
    core::{EditorWindowContext, EditorWindowExtension},
    reflect_widgets::wrap_ui::using_wrap_ui,
    utils::{orbit_camera_scene_show, OrbitView},
    PartOfSubScene, PreviewScene,
};

use super::{event_track_editor::TargetTracks, scene_preview::ScenePreviewConfig};

#[derive(Debug, Default)]
pub struct ClipPreviewWindow {
    pub orbit_view: OrbitView,
    pub target: Option<TargetTracks>,
    pub base_scene: Option<Handle<AnimatedScene>>,
    pub last_order: Option<TimingOrder>,
    /// Updating this has no effect, this is just here as a means of "publishing"
    /// the value for other windows to see
    pub current_time: Option<f32>,
}

#[derive(Debug)]
pub enum TimingOrder {
    Seek { time: f32 },
}

#[derive(Debug)]
pub enum ClipPreviewAction {
    TimingOrder(TimingOrder),
    SelectTarget(Option<TargetTracks>),
    SelectBaseScene(Handle<AnimatedScene>),
}

impl EditorWindowExtension for ClipPreviewWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        let timeline_height = 30.;

        egui::TopBottomPanel::top("Clip preview base scene selector")
            .resizable(false)
            .exact_height(timeline_height)
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| {
                self.draw_base_scene_selector(ui, world, ctx);
            });

        let Some(target) = &self.target else {
            ui.centered_and_justified(|ui| {
                ui.label("No target selected");
            });
            return;
        };
        let Some(base_scene) = &self.base_scene else {
            ui.centered_and_justified(|ui| {
                ui.label("No base scene selected");
            });
            return;
        };

        let preview_scene = match target {
            TargetTracks::Clip(handle) => {
                let Some(preview_scene) =
                    world.resource_scope::<ClipPreviewScenes, _>(|_, preview| {
                        preview.previews.get(&handle.id()).cloned()
                    })
                else {
                    ctx.editor_actions.dynamic(CreateClipPreview {
                        clip: handle.clone(),
                        scene: base_scene.clone(),
                    });
                    return;
                };

                preview_scene
            }
            TargetTracks::GraphNode { graph, node } => {
                let key = NodePreviewKey {
                    graph: graph.id(),
                    node_id: node.clone(),
                    pose_pin: EventMarkupNode::OUT_POSE.into(),
                };

                let Some(preview_scene) =
                    world.resource_scope::<NodePreviewScenes, _>(|_, preview| {
                        preview.previews.get(&key).cloned()
                    })
                else {
                    ctx.editor_actions.dynamic(CreateTrackNodePreview {
                        preview_key: key,
                        scene: base_scene.clone(),
                    });
                    return;
                };

                preview_scene
            }
        };

        let config = ScenePreviewConfig {
            animated_scene: preview_scene.clone(),
            view: self.orbit_view.clone(),
        };

        let ui_texture_id = ui.id().with("clip preview texture");
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

            if let Some(order) = self.last_order.take() {
                match order {
                    TimingOrder::Seek { time } => {
                        player.seek(time);
                        player.pause();
                    }
                }
            }
            self.current_time = Some(player.elapsed());
        } else {
            self.current_time = None;
        }

        orbit_camera_scene_show(&config, &mut self.orbit_view, ui, world, ui_texture_id);
    }

    fn display_name(&self) -> String {
        "Clip Preview".to_string()
    }

    fn handle_action(&mut self, action: DynWindowAction) {
        let Ok(action) = action.downcast::<ClipPreviewAction>() else {
            return;
        };

        match *action {
            ClipPreviewAction::TimingOrder(timing_order) => {
                self.last_order = Some(timing_order);
            }
            ClipPreviewAction::SelectTarget(target) => {
                self.target = target;
            }
            ClipPreviewAction::SelectBaseScene(handle) => {
                self.base_scene = Some(handle);
            }
        }
    }
}

impl ClipPreviewWindow {
    pub fn draw_base_scene_selector(
        &self,
        ui: &mut egui::Ui,
        world: &mut World,
        ctx: &mut EditorWindowContext,
    ) {
        using_wrap_ui(world, |mut env| {
            if let Some(new_handle) = env.mutable_buffered(
                &self.base_scene.clone().unwrap_or_default(),
                ui,
                ui.id().with("clip preview base scene selector"),
                &(),
            ) {
                ctx.editor_actions.window(
                    ctx.window_id,
                    ClipPreviewAction::SelectBaseScene(new_handle),
                );
            }
        });
    }
}
