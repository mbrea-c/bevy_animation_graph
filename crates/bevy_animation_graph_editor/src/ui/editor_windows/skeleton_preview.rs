use std::sync::Arc;

use bevy::{
    asset::{Assets, Handle},
    color::{
        Color, LinearRgba,
        palettes::css::{self, DARK_RED, GRAY, ORANGE},
    },
    core_pipeline::core_3d::Camera3d,
    ecs::{
        hierarchy::ChildSpawnerCommands,
        system::{In, Query},
    },
    gizmos::primitives::dim3::GizmoPrimitive3d,
    image::Image,
    math::{Isometry3d, Vec3, primitives::Cuboid},
    pbr::PointLight,
    platform::collections::HashMap,
    prelude::World,
    render::camera::{Camera, ClearColorConfig, RenderTarget},
    transform::components::Transform,
    utils::default,
};
use bevy_animation_graph::{
    core::{
        colliders::core::{ColliderConfig, ColliderShape, SkeletonColliderId, SkeletonColliders},
        id::BoneId,
        skeleton::Skeleton,
    },
    prelude::{
        AnimatedScene, AnimatedSceneHandle, AnimationGraphPlayer, AnimationSource,
        CustomRelativeDrawCommand,
    },
};
use egui_dock::egui;

use crate::{
    tree::{SkeletonTreeRenderer, Tree, TreeResult},
    ui::{
        PartOfSubScene, PreviewScene, SubSceneConfig, SubSceneSyncAction,
        actions::{
            colliders::{CreateOrEditCollider, DeleteCollider},
            window::DynWindowAction,
        },
        core::{EditorWindowContext, EditorWindowExtension},
        reflect_widgets::wrap_ui::using_wrap_ui,
        utils::{OrbitView, orbit_camera_scene_show, orbit_camera_transform, orbit_camera_update},
    },
};

#[derive(Debug)]
pub struct SkeletonCollidersPreviewWindow {
    pub orbit_view: OrbitView,
    pub target: Option<Handle<SkeletonColliders>>,
    pub base_scene: Option<Handle<AnimatedScene>>,
    pub hovered: Option<BoneId>,
    pub selected: Option<BoneId>,
    pub edit_buffers: HashMap<SelectableCollider, ColliderConfig>,
    pub selectable_collider: SelectableCollider,
    pub draw_colliders: Vec<(ColliderConfig, Color)>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum SelectableCollider {
    #[default]
    None,
    New,
    Existing(SkeletonColliderId),
}

impl Default for SkeletonCollidersPreviewWindow {
    fn default() -> Self {
        Self {
            orbit_view: OrbitView::default(),
            target: None,
            base_scene: None,
            hovered: None,
            selected: None,
            selectable_collider: SelectableCollider::default(),
            edit_buffers: HashMap::default(),
            draw_colliders: Vec::default(),
        }
    }
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
            .default_width(200.)
            .show_inside(ui, |ui| {
                self.draw_tree_view(ui, world, ctx);
            });

        egui::SidePanel::right("Inspector panel")
            .resizable(true)
            .default_width(200.)
            .show_inside(ui, |ui| {
                self.draw_bone_inspector(ui, world, ctx);
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
            hovered: self.hovered,
            selected: self.selected,
            draw_colliders: std::mem::take(&mut self.draw_colliders),
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
        });
    }

    pub fn draw_bone_inspector(
        &mut self,
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

        egui::ScrollArea::both().show(ui, |ui| {
            world.resource_scope::<Assets<SkeletonColliders>, _>(|world, skeleton_colliders| {
                world.resource_scope::<Assets<Skeleton>, _>(|world, skeletons| {
                    let Some(skeleton_colliders) = skeleton_colliders.get(target) else {
                        return;
                    };

                    let Some(bone_id) = self.selected else {
                        return;
                    };

                    let Some(skeleton) = skeletons.get(&skeleton_colliders.skeleton) else {
                        return;
                    };

                    let Some(bone_path) = skeleton.id_to_path(bone_id) else {
                        return;
                    };

                    ui.heading(format!(
                        "{}",
                        bone_path
                            .last()
                            .map(|n| n.to_string())
                            .unwrap_or("ROOT".to_string())
                    ));

                    let bone_colliders = skeleton_colliders
                        .get_colliders(bone_id)
                        .cloned()
                        .unwrap_or_default();

                    let selectables = bone_colliders
                        .iter()
                        .map(|cfg| SelectableCollider::Existing(cfg.id))
                        .chain([SelectableCollider::New]);

                    for selectable in selectables {
                        if ui
                            .selectable_label(
                                self.selectable_collider == selectable,
                                format!("{:?}", selectable),
                            )
                            .clicked()
                        {
                            self.selectable_collider = selectable;
                        }
                    }

                    ui.separator();

                    let edit_ui =
                        |ui: &mut egui::Ui, world: &mut World, config: &mut ColliderConfig| {
                            using_wrap_ui(world, |mut env| {
                                let id = ui.id().with("Collider creation shape");
                                ui.label("Shape");
                                if let Some(new_shape) =
                                    env.mutable_buffered(&mut config.shape, ui, id, &())
                                {
                                    config.shape = new_shape;
                                }

                                ui.add(egui::DragValue::new(&mut config.layers));

                                ui.label("Offsets");
                                if let Some(new_isometry) =
                                    env.mutable_buffered(&mut config.offset, ui, id, &())
                                {
                                    config.offset = new_isometry;
                                }
                            });
                        };

                    let cfg = if let Some(selectable) =
                        self.edit_buffers.get_mut(&self.selectable_collider)
                    {
                        Some(selectable)
                    } else {
                        match self.selectable_collider {
                            SelectableCollider::None => {}
                            SelectableCollider::New => {
                                self.edit_buffers.insert(
                                    self.selectable_collider.clone(),
                                    ColliderConfig {
                                        id: SkeletonColliderId::generate(),
                                        shape: ColliderShape::Cuboid(Cuboid::new(1., 1., 1.)),
                                        layers: 0,
                                        attached_to: bone_id,
                                        offset: Isometry3d::default(),
                                    },
                                );
                            }
                            SelectableCollider::Existing(skeleton_collider_id) => {
                                bone_colliders
                                    .iter()
                                    .find(|cfg| cfg.id == skeleton_collider_id)
                                    .map(|cfg| {
                                        self.edit_buffers.insert(
                                            SelectableCollider::Existing(cfg.id),
                                            cfg.clone(),
                                        )
                                    });
                            }
                        }

                        self.edit_buffers.get_mut(&self.selectable_collider)
                    };

                    self.draw_colliders.extend(
                        bone_colliders
                            .iter()
                            .filter(|c| cfg.as_ref().map_or(true, |cfg| c.id != cfg.id))
                            .cloned()
                            .map(|c| (c, DARK_RED.into())),
                    );

                    if let Some(cfg) = cfg {
                        self.draw_colliders.push((cfg.clone(), ORANGE.into()));

                        let mut should_clear = false;
                        edit_ui(ui, world, cfg);

                        if ui.button("Apply").clicked() {
                            ctx.editor_actions.dynamic(CreateOrEditCollider {
                                colliders: target.clone(),
                                config: cfg.clone(),
                            });

                            should_clear = true;
                        }

                        if ui.button("Delete").clicked() {
                            ctx.editor_actions.dynamic(DeleteCollider {
                                colliders: target.clone(),
                                bone_id,
                                collider_id: cfg.id,
                            });

                            should_clear = true;
                        }

                        if should_clear {
                            self.edit_buffers.clear();
                        }
                    }
                });
            });
        });
    }

    pub fn draw_tree_view(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut World,
        _ctx: &mut EditorWindowContext,
    ) {
        let Some(target) = &self.target else {
            ui.centered_and_justified(|ui| {
                ui.label("No target selected");
            });
            return;
        };

        egui::ScrollArea::both().show(ui, |ui| {
            world.resource_scope::<Assets<SkeletonColliders>, _>(|world, skeleton_colliders| {
                world.resource_scope::<Assets<Skeleton>, _>(|_, skeletons| {
                    let Some(skeleton_colliders) = skeleton_colliders.get(target) else {
                        return;
                    };
                    let Some(skeleton) = skeletons.get(&skeleton_colliders.skeleton) else {
                        return;
                    };

                    // Tree, assemble!
                    let response =
                        Tree::skeleton_tree(skeleton).picker_selector(ui, SkeletonTreeRenderer {});

                    match response {
                        TreeResult::Leaf(_, response) | TreeResult::Node(_, response) => {
                            self.hovered = response.hovered;
                            if let Some(clicked) = response.clicked {
                                self.selected = Some(clicked);
                                self.edit_buffers.clear();
                            }
                        }
                        _ => {}
                    };
                })
            });
        });
    }
}

#[derive(Clone)]
pub struct SkeletonCollidersPreviewConfig {
    pub animated_scene: Handle<AnimatedScene>,
    pub view: OrbitView,
    pub hovered: Option<BoneId>,
    pub selected: Option<BoneId>,
    pub draw_colliders: Vec<(ColliderConfig, Color)>,
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
        match self.animated_scene == new_config.animated_scene {
            true => SubSceneSyncAction::Update,
            false => SubSceneSyncAction::Respawn,
        }
    }

    fn update(&self, id: egui::Id, world: &mut World) {
        world
            .run_system_cached_with(orbit_camera_update, (id, self.view.clone()))
            .unwrap();
        world
            .run_system_cached_with(
                highlight_bones,
                (
                    id,
                    HighlightSettings {
                        hovered: self.hovered,
                        selected: self.selected,
                        draw_colliders: self.draw_colliders.clone(),
                    },
                ),
            )
            .unwrap();
    }
}

struct HighlightSettings {
    hovered: Option<BoneId>,
    selected: Option<BoneId>,
    draw_colliders: Vec<(ColliderConfig, Color)>,
}

fn highlight_bones(
    In((id, mut input)): In<(egui::Id, HighlightSettings)>,
    mut q_animation_players: Query<(&mut AnimationGraphPlayer, &PartOfSubScene)>,
) {
    for (mut player, PartOfSubScene(target_id)) in &mut q_animation_players {
        if id != *target_id {
            continue;
        }

        let mut drawable = vec![];
        if let Some(clicked) = input.selected {
            drawable.push((clicked, css::LIGHT_SKY_BLUE.into()));
        }

        if let Some(hovered) = input.hovered {
            if !input.selected.is_some_and(|b| b == hovered) {
                drawable.push((hovered, GRAY.into()));
            }
        }

        player.gizmo_for_bones_with_color(drawable);

        for (cfg, color) in input.draw_colliders.drain(..) {
            player.custom_relative_gizmo(CustomRelativeDrawCommand {
                bone_id: cfg.attached_to,
                f: Arc::new(move |bone_transform, gizmos| {
                    let offset_transform = bone_transform * Transform::from_isometry(cfg.offset);

                    match cfg.shape {
                        ColliderShape::Sphere(sphere) => {
                            gizmos.primitive_3d(&sphere, offset_transform.to_isometry(), color);
                        }
                        ColliderShape::Capsule(capsule3d) => {
                            gizmos.primitive_3d(&capsule3d, offset_transform.to_isometry(), color);
                        }
                        ColliderShape::Cuboid(cuboid) => {
                            gizmos.primitive_3d(&cuboid, offset_transform.to_isometry(), color);
                        }
                    }
                }),
            });
        }
    }
}
