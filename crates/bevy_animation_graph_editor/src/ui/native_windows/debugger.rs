use bevy::{
    ecs::hierarchy::ChildSpawnerCommands,
    light::PointLight,
    math::Vec3,
    prelude::{Handle, Image, In, Query, Transform, World},
};
use bevy_animation_graph::core::{
    animated_scene::{AnimatedScene, AnimatedSceneHandle},
    animation_graph_player::AnimationGraphPlayer,
    context::node_states::StateKey,
    edge_data::DataValue,
    pose::Pose,
};
use egui_dock::egui;

use crate::ui::{
    OverrideSceneAnimation, PartOfSubScene, SubSceneConfig, SubSceneSyncAction,
    native_windows::{EditorWindowContext, NativeEditorWindowExtension},
    state_management::global::{
        active_graph_context::ActiveContexts,
        active_graph_node::{ActiveGraphNode, SetActiveGraphNode},
        active_scene::ActiveScene,
        get_global_state,
    },
    utils::{self, orbit_camera_scene_show, using_inspector_env},
};

#[derive(Debug, Default)]
pub struct DebuggerWindow;

impl NativeEditorWindowExtension for DebuggerWindow {
    fn ui(&self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        let Some(active_node) = get_global_state::<ActiveGraphNode>(world).cloned() else {
            return;
        };
        let Some(active_scene) = get_global_state::<ActiveScene>(world).cloned() else {
            return;
        };

        let Some(data_pin_map) =
            utils::get_node_output_data_pins(world, active_node.handle.id(), &active_node.node)
        else {
            return;
        };

        let Some(contexts) = get_global_state::<ActiveContexts>(world) else {
            return;
        };

        let Some((entity, context_id)) = contexts
            .by_asset
            .get(&active_node.handle.id().untyped())
            .cloned()
        else {
            return;
        };

        let mut query = world.query::<&AnimationGraphPlayer>();
        let Ok(player) = query.get(world, entity) else {
            return;
        };

        let Some(graph_context) = Some(context_id)
            .zip(player.get_context_arena())
            .and_then(|(id, ca)| ca.get_context(id))
        else {
            return;
        };

        // Select data to display cached value of
        let mut selected_pin_id = active_node.selected_pin.clone().unwrap_or("".to_string());
        egui::ComboBox::new("cache debugger window", "Pin")
            .selected_text(&selected_pin_id)
            .show_ui(ui, |ui| {
                for (id, spec) in data_pin_map.iter() {
                    ui.selectable_value(&mut selected_pin_id, id.clone(), id)
                        .on_hover_ui(|ui| {
                            ui.label(format!("{spec:?}"));
                        });
                }
            });
        let new_pin_id = if selected_pin_id.is_empty() {
            None
        } else {
            Some(selected_pin_id)
        };

        if new_pin_id != active_node.selected_pin {
            ctx.trigger(SetActiveGraphNode {
                new: ActiveGraphNode {
                    selected_pin: new_pin_id.clone(),
                    ..active_node.clone()
                },
            });
        }

        // Now get the selected value and display it!

        let Some(val) = new_pin_id.and_then(|pin_id| {
            graph_context
                .node_caches
                .get_output_data(active_node.node.clone(), StateKey::Default, pin_id.clone())
                .ok()
        }) else {
            return;
        };

        match &val {
            DataValue::Pose(pose) => {
                self.pose_readonly(pose, active_scene.handle.clone(), ui, world, ui.id());
            }
            _ => using_inspector_env(world, |mut env| env.ui_for_reflect_readonly(&val, ui)),
        }
    }

    fn display_name(&self) -> String {
        "Debugger".to_string()
    }
}

#[derive(Clone, PartialEq)]
pub struct PoseSubSceneConfig {
    pose: Pose,
    animated_scene: Handle<AnimatedScene>,
}

impl SubSceneConfig for PoseSubSceneConfig {
    fn spawn(&self, builder: &mut ChildSpawnerCommands, _render_target: Handle<Image>) {
        builder.spawn((
            PointLight::default(),
            Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        ));

        builder.spawn((
            AnimatedSceneHandle::new(self.animated_scene.clone()),
            OverrideSceneAnimation(self.pose.clone()),
        ));
    }

    fn sync_action(&self, new_config: &Self) -> SubSceneSyncAction {
        match (
            self.animated_scene == new_config.animated_scene,
            self.pose == new_config.pose,
        ) {
            (true, true) => SubSceneSyncAction::Nothing,
            (true, false) => SubSceneSyncAction::Update,
            (false, _) => SubSceneSyncAction::Respawn,
        }
    }

    fn update(&self, id: egui::Id, world: &mut World) {
        world
            .run_system_cached_with(update_pose_subscene, (id, self.pose.clone()))
            .unwrap();
    }
}

fn update_pose_subscene(
    In((target_id, pose)): In<(egui::Id, Pose)>,
    mut query: Query<(&mut OverrideSceneAnimation, &PartOfSubScene)>,
) {
    for (mut override_scene, PartOfSubScene(id)) in &mut query {
        if target_id == *id {
            override_scene.0 = pose;
            break;
        }
    }
}

// some helpers only used here
impl DebuggerWindow {
    // Easier to manually display widget here than rely on bevy-inspector-egui, as we need mutable
    // world access for setting up the Bevy scene
    fn pose_readonly(
        &self,
        pose: &Pose,
        scene: Handle<AnimatedScene>,
        ui: &mut egui::Ui,
        world: &mut World,
        id: egui::Id,
    ) {
        let config = PoseSubSceneConfig {
            pose: pose.clone(),
            animated_scene: scene,
        };

        let mut size = ui.available_size();
        size.y *= 0.6;

        ui.allocate_ui(size, |ui| {
            orbit_camera_scene_show(&config, ui, world, id);
        });

        ui.separator();

        let size = ui.available_size();

        egui::ScrollArea::both()
            .max_width(size.x)
            .max_height(size.y)
            .show(ui, |ui| {
                ui.collapsing("Inspect pose values", |ui| {
                    using_inspector_env(world, |mut env| env.ui_for_reflect_readonly(pose, ui));
                });
            });
    }
}
