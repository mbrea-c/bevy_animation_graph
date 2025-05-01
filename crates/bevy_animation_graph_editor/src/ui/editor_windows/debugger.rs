use bevy::{
    color::LinearRgba,
    ecs::hierarchy::ChildSpawnerCommands,
    math::Vec3,
    pbr::PointLight,
    prelude::{Camera, Camera3d, ClearColorConfig, Handle, Image, In, Query, Transform, World},
    render::camera::RenderTarget,
    utils::default,
};
use bevy_animation_graph::{
    core::{animation_graph::SourcePin, pose::Pose},
    prelude::{
        AnimatedScene, AnimatedSceneHandle, AnimatedSceneInstance, AnimationGraphPlayer, DataValue,
    },
};
use egui_dock::egui;

use crate::ui::{
    OverrideSceneAnimation, PartOfSubScene, PreviewScene, SubSceneConfig, SubSceneSyncAction,
    core::{EditorWindowContext, EditorWindowExtension, InspectorSelection},
    utils::{
        self, OrbitView, orbit_camera_scene_show, orbit_camera_transform, orbit_camera_update,
        using_inspector_env,
    },
};

#[derive(Debug, Default)]
pub struct DebuggerWindow {
    pub orbit_view: OrbitView,
}

impl EditorWindowExtension for DebuggerWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        if ctx.global_state.scene.is_none() {
            return;
        };
        let mut query = world.query::<(&AnimatedSceneInstance, &PreviewScene)>();
        let Ok((instance, _)) = query.single(world) else {
            return;
        };
        let entity = instance.player_entity();
        let mut query = world.query::<&AnimationGraphPlayer>();

        let InspectorSelection::Node(node_selection) = &mut ctx.global_state.inspector_selection
        else {
            return;
        };

        let Some(data_pin_map) = utils::get_node_output_data_pins(
            world,
            node_selection.graph.id(),
            &node_selection.node,
        ) else {
            return;
        };

        let Ok(player) = query.get(world, entity) else {
            return;
        };

        let Some(graph_selection) = &ctx.global_state.graph_editor else {
            return;
        };

        let Some(scene_selection) = ctx.global_state.scene.as_ref() else {
            return;
        };

        let Some(graph_context) = scene_selection
            .active_context
            .get(&graph_selection.graph.id().untyped())
            .and_then(|id| Some(id).zip(player.get_context_arena()))
            .and_then(|(id, ca)| ca.get_context(*id))
        else {
            return;
        };

        // Select data to display cached value of
        let mut selected_pin_id = node_selection
            .selected_pin_id
            .clone()
            .unwrap_or("".to_string());
        egui::ComboBox::new("cache debugger window", "Pin")
            .selected_text(&selected_pin_id)
            .show_ui(ui, |ui| {
                for (id, spec) in data_pin_map.iter() {
                    ui.selectable_value(&mut selected_pin_id, id.clone(), id)
                        .on_hover_ui(|ui| {
                            ui.label(format!("{:?}", spec));
                        });
                }
            });
        let new_pin_id = if selected_pin_id.is_empty() {
            None
        } else {
            Some(selected_pin_id)
        };

        if new_pin_id != node_selection.selected_pin_id {
            node_selection.selected_pin_id = new_pin_id.clone();
        }

        // Now get the selected value and display it!

        let Some(val) = new_pin_id.and_then(|pin_id| {
            graph_context.caches.get_primary(|c| {
                let node_id = node_selection.node.clone();
                let pin_id = pin_id.clone();
                c.get_data(&SourcePin::NodeData(node_id, pin_id)).cloned()
            })
        }) else {
            return;
        };

        match &val {
            DataValue::Pose(pose) => {
                self.pose_readonly(pose, scene_selection.scene.clone(), ui, world, ui.id(), ctx);
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
    view: OrbitView,
}

impl SubSceneConfig for PoseSubSceneConfig {
    fn spawn(&self, builder: &mut ChildSpawnerCommands, render_target: Handle<Image>) {
        builder.spawn((
            PointLight::default(),
            Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        ));

        builder.spawn((
            Camera3d::default(),
            Camera {
                // render before the "main pass" camera
                order: -1,
                clear_color: ClearColorConfig::Custom(LinearRgba::new(1.0, 1.0, 1.0, 0.0).into()),
                target: RenderTarget::Image(render_target.into()),
                ..default()
            },
            // Position based on orbit camera parameters
            orbit_camera_transform(&self.view),
        ));

        builder.spawn((
            AnimatedSceneHandle(self.animated_scene.clone()),
            OverrideSceneAnimation(self.pose.clone()),
        ));
    }

    fn sync_action(&self, new_config: &Self) -> SubSceneSyncAction {
        match (
            self.animated_scene == new_config.animated_scene,
            self.pose == new_config.pose,
            self.view == new_config.view,
        ) {
            (true, true, true) => SubSceneSyncAction::Nothing,
            (true, false, _) => SubSceneSyncAction::Update,
            (true, _, false) => SubSceneSyncAction::Update,
            (false, _, _) => SubSceneSyncAction::Respawn,
        }
    }

    fn update(&self, id: egui::Id, world: &mut World) {
        world
            .run_system_cached_with(update_pose_subscene, (id, self.pose.clone()))
            .unwrap();
        world
            .run_system_cached_with(orbit_camera_update, (id, self.view.clone()))
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
        &mut self,
        pose: &Pose,
        scene: Handle<AnimatedScene>,
        ui: &mut egui::Ui,
        world: &mut World,
        id: egui::Id,
        _ctx: &mut EditorWindowContext,
    ) {
        let config = PoseSubSceneConfig {
            pose: pose.clone(),
            animated_scene: scene,
            view: self.orbit_view.clone(),
        };

        let mut size = ui.available_size();
        size.y *= 0.6;

        ui.allocate_ui(size, |ui| {
            orbit_camera_scene_show(&config, &mut self.orbit_view, ui, world, id);
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
