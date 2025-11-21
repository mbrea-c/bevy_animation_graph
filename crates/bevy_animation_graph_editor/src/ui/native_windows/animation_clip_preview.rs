use bevy::prelude::World;
use bevy_animation_graph::{
    builtin_nodes::EventMarkupNode,
    prelude::{AnimatedSceneInstance, AnimationGraphPlayer},
};
use egui_dock::egui;

use crate::ui::{
    PartOfSubScene, PreviewScene,
    actions::clip_preview::{
        ClipPreviewScenes, CreateClipPreview, CreateTrackNodePreview, NodePreviewKey,
        NodePreviewScenes,
    },
    global_state::register_if_missing,
    native_windows::{EditorWindowContext, NativeEditorWindowExtension},
    reflect_widgets::wrap_ui::using_wrap_ui,
    utils::orbit_camera_scene_show,
    view_state::clip_preview::{
        ClipPreviewTimingOrder, ClipPreviewViewState, SetClipPreviewBaseScene, SetElapsedTime,
    },
};

use super::{
    EditorWindowRegistrationContext, event_track_editor::TargetTracks,
    scene_preview::ScenePreviewConfig,
};

#[derive(Debug, Default)]
pub struct ClipPreviewWindow;

impl NativeEditorWindowExtension for ClipPreviewWindow {
    fn init(&self, world: &mut World, ctx: &EditorWindowRegistrationContext) {
        register_if_missing::<ClipPreviewViewState>(world, ctx.view);
    }

    fn ui(&self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        let timeline_height = 30.;

        egui::TopBottomPanel::top("Clip preview base scene selector")
            .resizable(false)
            .exact_height(timeline_height)
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| {
                self.draw_base_scene_selector(ui, world, ctx);
            });

        let Some(state) = ctx.get_view_state::<ClipPreviewViewState>(world) else {
            return;
        };

        let Some(target) = state.target_tracks() else {
            ui.centered_and_justified(|ui| {
                ui.label("No target selected");
            });
            return;
        };
        let Some(base_scene) = state.base_scene() else {
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
        };

        let order_status = ctx
            .get_view_state::<ClipPreviewViewState>(world)
            .and_then(|s| s.order_status())
            .cloned();

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

            if let Some(order) = order_status
                && !order.applied_by.contains(&ctx.window_entity)
            {
                match &order.order {
                    ClipPreviewTimingOrder::Seek { time } => {
                        player.seek(*time);
                        player.pause();
                    }
                }
            }
            ctx.trigger(SetElapsedTime {
                entity: ctx.view_entity,
                time: player.elapsed(),
            });
        }

        orbit_camera_scene_show(&config, ui, world, ui_texture_id);
    }

    fn display_name(&self) -> String {
        "Clip Preview".to_string()
    }
}

impl ClipPreviewWindow {
    pub fn draw_base_scene_selector(
        &self,
        ui: &mut egui::Ui,
        world: &mut World,
        ctx: &mut EditorWindowContext,
    ) {
        let base_scene = ctx
            .get_view_state::<ClipPreviewViewState>(world)
            .and_then(|s| s.base_scene());
        using_wrap_ui(world, |mut env| {
            if let Some(new_handle) = env.mutable_buffered(
                &base_scene.unwrap_or_default(),
                ui,
                ui.id().with("clip preview base scene selector"),
                &(),
            ) {
                ctx.trigger(SetClipPreviewBaseScene {
                    entity: ctx.view_entity,
                    scene: new_handle,
                });
            }
        });
    }
}
