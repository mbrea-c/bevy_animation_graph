use bevy::{
    color::LinearRgba,
    ecs::world::CommandQueue,
    math::Vec3,
    pbr::PointLight,
    prelude::{
        AppTypeRegistry, Camera, Camera3d, ChildBuild, ChildBuilder, ClearColorConfig, Handle,
        Image, In, Query, Transform, World,
    },
    render::camera::RenderTarget,
    utils::default,
};
use bevy_animation_graph::{
    core::{animation_graph::SourcePin, pose::Pose},
    prelude::{
        AnimatedScene, AnimatedSceneBundle, AnimatedSceneHandle, AnimatedSceneInstance,
        AnimationGraphPlayer, DataValue,
    },
};
use bevy_inspector_egui::reflect_inspector::{Context, InspectorUi};
use egui_dock::egui;

use crate::ui::{
    core::{EditorContext, EditorWindowExtension, InspectorSelection},
    provide_texture_for_scene,
    utils::{self, render_image},
    OverrideSceneAnimation, PartOfSubScene, PreviewScene, SubSceneConfig, SubSceneSyncAction,
};

#[derive(Debug)]
pub struct DebuggerWindow;

impl EditorWindowExtension for DebuggerWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorContext) {
        if ctx.selection.scene.is_none() {
            return;
        };
        let mut query = world.query::<(&AnimatedSceneInstance, &PreviewScene)>();
        let Ok((instance, _)) = query.get_single(world) else {
            return;
        };
        let entity = instance.player_entity;
        let mut query = world.query::<&AnimationGraphPlayer>();

        let unsafe_world = world.as_unsafe_world_cell();
        let type_registry = unsafe {
            unsafe_world
                .get_resource::<AppTypeRegistry>()
                .unwrap()
                .0
                .clone()
        };
        let type_registry = type_registry.read();
        let mut queue = CommandQueue::default();
        let mut cx = Context {
            world: Some(unsafe { unsafe_world.world_mut() }.into()),
            queue: Some(&mut queue),
        };
        let mut env = InspectorUi::for_bevy(&type_registry, &mut cx);

        let world = unsafe { unsafe_world.world_mut() };

        let InspectorSelection::Node(node_selection) = &mut ctx.selection.inspector_selection
        else {
            return;
        };

        let Some(data_pin_map) =
            utils::get_node_output_data_pins(world, node_selection.graph, &node_selection.node)
        else {
            return;
        };

        let Ok(player) = query.get(world, entity) else {
            return;
        };

        let Some(graph_selection) = &ctx.selection.graph_editor else {
            return;
        };

        let Some(scene_selection) = ctx.selection.scene.as_ref() else {
            return;
        };

        let Some(graph_context) = scene_selection
            .active_context
            .get(&graph_selection.graph.untyped())
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
        let new_pin_id = if selected_pin_id == "" {
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
            _ => {
                env.ui_for_reflect_readonly(&val, ui);
            }
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
                clear_color: ClearColorConfig::Custom(LinearRgba::new(0.0, 0.0, 0.0, 1.0).into()),
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
    // Easier to manually display widget here than rely on bevy-inspector-egui, as we need
    // mutable world access.
    fn pose_readonly(
        &self,
        pose: &Pose,
        scene: Handle<AnimatedScene>,
        ui: &mut egui::Ui,
        world: &mut World,
        id: egui::Id,
        _ctx: &mut EditorContext,
    ) {
        let config = PoseSubSceneConfig {
            pose: pose.clone(),
            animated_scene: scene,
        };
        // First we need to make sure the subscene is created and get the camera image handle
        let texture = provide_texture_for_scene(world, id, config);

        render_image(ui, world, &texture);
    }
}
