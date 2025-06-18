use bevy::{
    asset::{Assets, Handle},
    color::{Color, LinearRgba},
    core_pipeline::core_3d::Camera3d,
    ecs::hierarchy::ChildSpawnerCommands,
    image::Image,
    math::Vec3,
    pbr::PointLight,
    prelude::World,
    render::camera::{Camera, ClearColorConfig, RenderTarget},
    transform::components::Transform,
    utils::default,
};
use bevy_animation_graph::{
    core::{colliders::core::SkeletonColliders, skeleton::Skeleton},
    prelude::{AnimatedScene, AnimatedSceneHandle, AnimationSource},
};
use egui_dock::egui;

use crate::ui::{
    PreviewScene, SubSceneConfig, SubSceneSyncAction,
    actions::window::DynWindowAction,
    core::{EditorWindowContext, EditorWindowExtension},
    reflect_widgets::wrap_ui::using_wrap_ui,
    utils::{OrbitView, orbit_camera_scene_show, orbit_camera_transform, orbit_camera_update},
};

#[derive(Debug, Default)]
pub struct SkeletonCollidersPreviewWindow {
    pub orbit_view: OrbitView,
    pub target: Option<Handle<SkeletonColliders>>,
    pub base_scene: Option<Handle<AnimatedScene>>,
}

#[derive(Debug)]
pub enum CollidersPreviewAction {
    SelectBaseScene(Handle<AnimatedScene>),
    SelectTarget(Handle<SkeletonColliders>),
}

impl EditorWindowExtension for SkeletonCollidersPreviewWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        let timeline_height = 30.;

        egui::TopBottomPanel::top("Top panel")
            .resizable(false)
            .exact_height(timeline_height)
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| {
                self.draw_base_scene_selector(ui, world, ctx);
            });

        egui::SidePanel::left("Hierarchical tree view")
            .resizable(true)
            .show_inside(ui, |ui| {
                self.draw_tree_view(ui, world, ctx);
            });

        let Some(base_scene) = &self.base_scene else {
            ui.centered_and_justified(|ui| {
                ui.label("No base scene selected");
            });
            return;
        };

        let config = SkeletonCollidersPreviewConfig {
            animated_scene: base_scene.clone(),
            view: self.orbit_view.clone(),
        };

        let ui_texture_id = ui.id().with("clip preview texture");
        orbit_camera_scene_show(&config, &mut self.orbit_view, ui, world, ui_texture_id);
    }

    fn display_name(&self) -> String {
        "Clip Preview".to_string()
    }

    fn handle_action(&mut self, action: DynWindowAction) {
        let Ok(action) = action.downcast::<CollidersPreviewAction>() else {
            return;
        };

        match *action {
            CollidersPreviewAction::SelectBaseScene(handle) => {
                self.base_scene = Some(handle);
            }
            CollidersPreviewAction::SelectTarget(handle) => self.target = Some(handle),
        }
    }
}

impl SkeletonCollidersPreviewWindow {
    pub fn draw_base_scene_selector(
        &self,
        ui: &mut egui::Ui,
        world: &mut World,
        ctx: &mut EditorWindowContext,
    ) {
        ui.horizontal(|ui| {
            ui.label("Yoooo");
            using_wrap_ui(world, |mut env| {
                if let Some(new_handle) = env.mutable_buffered(
                    &self.base_scene.clone().unwrap_or_default(),
                    ui,
                    ui.id().with("skeleton colliders base scene selector"),
                    &(),
                ) {
                    ctx.editor_actions.window(
                        ctx.window_id,
                        CollidersPreviewAction::SelectBaseScene(new_handle),
                    );
                }
            });

            ui.label("Yoooo another one!");
            using_wrap_ui(world, |mut env| {
                if let Some(new_handle) = env.mutable_buffered(
                    &self.target.clone().unwrap_or_default(),
                    ui,
                    ui.id().with("skeleton colliders target selectors"),
                    &(),
                ) {
                    ctx.editor_actions.window(
                        ctx.window_id,
                        CollidersPreviewAction::SelectTarget(new_handle),
                    );
                }
            });
            ui.label("Les go we ballin");
        });
    }

    pub fn draw_tree_view(
        &self,
        ui: &mut egui::Ui,
        world: &mut World,
        ctx: &mut EditorWindowContext,
    ) {
        let Some(target) = &self.target else {
            ui.centered_and_justified(|ui| {
                ui.label("No target selected");
            });
            return;
        };

        world.resource_scope::<Assets<SkeletonColliders>, _>(|world, skeleton_colliders| {
            world.resource_scope::<Assets<Skeleton>, _>(|world, skeletons| {
                let Some(skeleton_colliders) = skeleton_colliders.get(target) else {
                    return;
                };
                let Some(skeleton) = skeletons.get(&skeleton_colliders.skeleton) else {
                    return;
                };

                // Tree, assemble!
            })
        });
    }
}

#[derive(Clone, PartialEq)]
pub struct SkeletonCollidersPreviewConfig {
    pub animated_scene: Handle<AnimatedScene>,
    pub view: OrbitView,
}

impl SubSceneConfig for SkeletonCollidersPreviewConfig {
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
                clear_color: ClearColorConfig::Custom(Color::from(LinearRgba::new(
                    1.0, 1.0, 1.0, 0.0,
                ))),
                target: RenderTarget::Image(render_target.into()),
                ..default()
            },
            orbit_camera_transform(&self.view),
        ));

        builder.spawn((
            AnimatedSceneHandle {
                handle: self.animated_scene.clone(),
                override_source: Some(AnimationSource::None),
            },
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
