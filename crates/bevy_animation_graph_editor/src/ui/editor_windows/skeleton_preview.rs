use std::sync::Arc;

use bevy::{
    asset::{Assets, Handle},
    color::{
        Alpha, Color, LinearRgba,
        palettes::css::{self, DARK_RED, GRAY, ORANGE, YELLOW},
    },
    core_pipeline::core_3d::Camera3d,
    ecs::{
        hierarchy::ChildSpawnerCommands,
        system::{In, Query, Res},
    },
    gizmos::primitives::dim3::GizmoPrimitive3d,
    image::Image,
    math::{Vec3, primitives::Cuboid},
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
        config::{FlipNameMapper, SymmertryMode, SymmetryConfig},
    },
};
use egui_dock::egui;

use crate::{
    tree::{SkeletonTreeRenderer, Tree, TreeResult},
    ui::{
        PartOfSubScene, PreviewScene, SubSceneConfig, SubSceneSyncAction,
        actions::{
            colliders::{
                CreateOrEditCollider, DeleteCollider, UpdateDefaultLayers, UpdateSuffixes,
                UpdateSymmetryConfig, UpdateSymmetryEnabled,
            },
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
    pub reverse_index: Option<ReverseSkeletonIndex>,
    pub base_scene: Option<Handle<AnimatedScene>>,
    pub hovered: Option<BoneId>,
    pub selected: Option<BoneId>,
    pub edit_buffers: HashMap<SelectableCollider, ColliderConfig>,
    pub selectable_collider: SelectableCollider,
    pub draw_colliders: Vec<(ColliderConfig, Color)>,
    pub show_global_settings: bool,
    pub show_all_colliders: bool,
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
            reverse_index: None,
            base_scene: None,
            hovered: None,
            selected: None,
            selectable_collider: SelectableCollider::default(),
            edit_buffers: HashMap::default(),
            draw_colliders: Vec::default(),
            show_global_settings: false,
            show_all_colliders: true,
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
            .frame(egui::Frame::NONE.inner_margin(5.))
            .show_inside(ui, |ui| {
                self.draw_top_panel(ui, world, ctx);
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

        if self.show_global_settings {
            egui::Window::new("Skeleton collider settings").show(ui.ctx(), |ui| {
                self.draw_settings_panel(ui, world, ctx);
            });
        }

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
            CollidersPreviewAction::SelectTarget(handle) => {
                self.target = Some(handle);
                self.reverse_index = None;
            }
        }
    }
}

impl SkeletonCollidersPreviewWindow {
    pub fn draw_settings_panel(
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

        world.resource_scope::<Assets<SkeletonColliders>, _>(|world, skeleton_colliders| {
            let Some(skeleton_colliders) = skeleton_colliders.get(target) else {
                return;
            };

            let mut symmetry_enabled = skeleton_colliders.symmetry_enabled;
            if ui
                .checkbox(&mut symmetry_enabled, "Enable symmetry (mirror along X)")
                .changed()
            {
                ctx.editor_actions.dynamic(UpdateSymmetryEnabled {
                    colliders: target.clone(),
                    symmetry_enabled,
                });
            }

            using_wrap_ui(world, |mut env| {
                let FlipNameMapper::Pattern(pattern_mapper) =
                    &skeleton_colliders.symmetry.name_mapper;
                if let Some(new_mapper) = env.mutable_buffered(
                    pattern_mapper,
                    ui,
                    ui.id().with("skeleton colliders symmetry configuration"),
                    &(),
                ) {
                    ctx.editor_actions.dynamic(UpdateSymmetryConfig {
                        colliders: target.clone(),
                        symmetry: SymmetryConfig {
                            name_mapper: FlipNameMapper::Pattern(new_mapper),
                            mode: SymmertryMode::MirrorX,
                        },
                    });
                }
            });

            ui.separator();

            ui.heading("Default physics layers");
            let mut layer_membership = skeleton_colliders.default_layer_membership;
            let mut layer_filter = skeleton_colliders.default_layer_filter;
            let mut changed = false;

            ui.horizontal(|ui| {
                ui.label("Membership");
                if ui
                    .add(egui::DragValue::new(&mut layer_membership))
                    .changed()
                {
                    changed = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Filters");
                if ui.add(egui::DragValue::new(&mut layer_filter)).changed() {
                    changed = true
                }
            });

            if changed {
                ctx.editor_actions.dynamic(UpdateDefaultLayers {
                    colliders: target.clone(),
                    layer_membership,
                    layer_filter,
                });
            }

            let mut suffix = skeleton_colliders.suffix.clone();
            let mut mirror_suffix = skeleton_colliders.mirror_suffix.clone();
            let mut changed = false;

            ui.horizontal(|ui| {
                ui.label("Suffix");
                if ui.text_edit_singleline(&mut suffix).changed() {
                    changed = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Mirror suffix");
                if ui.text_edit_singleline(&mut mirror_suffix).changed() {
                    changed = true
                }
            });

            if changed {
                ctx.editor_actions.dynamic(UpdateSuffixes {
                    colliders: target.clone(),
                    suffix,
                    mirror_suffix,
                });
            }
        });

        ui.separator();

        ui.heading("Preview settings");
        ui.checkbox(&mut self.show_all_colliders, "Show all colliders");
    }

    pub fn draw_top_panel(
        &mut self,
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

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("⚙").clicked() {
                    self.show_global_settings = !self.show_global_settings;
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

                    let Some(skeleton) = skeletons.get(&skeleton_colliders.skeleton) else {
                        return;
                    };

                    let reverse_index = if let Some(reverse_index) = &self.reverse_index {
                        reverse_index
                    } else {
                        self.reverse_index = Some(ReverseSkeletonIndex::new(
                            skeleton,
                            &skeleton_colliders.symmetry,
                        ));

                        self.reverse_index.as_ref().unwrap()
                    };

                    let Some(bone_id) = self.selected else {
                        self.draw_colliders = Self::collect_drawable_colliders(
                            None,
                            skeleton_colliders,
                            skeleton,
                            reverse_index,
                            None,
                            self.show_all_colliders,
                        );
                        return;
                    };

                    let Some(bone_path) = skeleton.id_to_path(bone_id) else {
                        return;
                    };

                    ui.heading(
                        bone_path
                            .last()
                            .map(|n| n.to_string())
                            .unwrap_or("ROOT".to_string())
                            .to_string(),
                    );

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
                                match selectable {
                                    SelectableCollider::None => "NONE".to_string(),
                                    SelectableCollider::New => "New collider".to_string(),
                                    SelectableCollider::Existing(skeleton_collider_id) => {
                                        format!(
                                            "Existing: {}...",
                                            &format!("{skeleton_collider_id:?}")[0..10]
                                        )
                                    }
                                },
                            )
                            .clicked()
                        {
                            self.selectable_collider = selectable;
                        }
                    }

                    ui.separator();

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
                                        override_layers: false,
                                        layer_membership: skeleton_colliders
                                            .default_layer_membership,
                                        layer_filter: skeleton_colliders.default_layer_filter,
                                        attached_to: bone_id,
                                        ..default()
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

                    let active_collider = cfg.as_ref().map(|c| (*c).clone());

                    self.draw_colliders = Self::collect_drawable_colliders(
                        Some(bone_id),
                        skeleton_colliders,
                        skeleton,
                        reverse_index,
                        active_collider,
                        self.show_all_colliders,
                    );

                    if let Some(cfg) = cfg {
                        let mut should_clear = false;
                        Self::draw_collider_inspector(ui, world, cfg);

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

    pub fn draw_collider_inspector(
        ui: &mut egui::Ui,
        world: &mut World,
        config: &mut ColliderConfig,
    ) {
        using_wrap_ui(world, |mut env| {
            let id = ui.id().with("Collider creation shape");
            ui.label("Shape");

            if let Some(new_shape) = env.mutable_buffered(&config.shape, ui, id.with("shape"), &())
            {
                config.shape = new_shape;
            }

            ui.horizontal(|ui| {
                ui.label("Label");
                ui.text_edit_singleline(&mut config.label);
            });
            ui.checkbox(&mut config.use_suffixes, "Use suffixes");
            ui.label("Physics layers");
            ui.checkbox(&mut config.override_layers, "Override global");
            ui.horizontal(|ui| {
                ui.label("Membership");
                ui.add_enabled(
                    config.override_layers,
                    egui::DragValue::new(&mut config.layer_membership),
                );
            });
            ui.horizontal(|ui| {
                ui.label("Filters");
                ui.add_enabled(
                    config.override_layers,
                    egui::DragValue::new(&mut config.layer_filter),
                );
            });

            ui.label("Offsets");

            if let Some(offset_mode) =
                env.mutable_buffered(&config.offset_mode, ui, id.with("offset_mode"), &())
            {
                config.offset_mode = offset_mode;
            }

            if let Some(new_isometry) =
                env.mutable_buffered(&config.offset, ui, id.with("offset"), &())
            {
                config.offset = new_isometry;
            }
        });
    }

    pub fn collect_drawable_colliders(
        bone_id: Option<BoneId>,
        skeleton_colliders: &SkeletonColliders,
        skeleton: &Skeleton,
        reverse_index: &ReverseSkeletonIndex,
        active_collider: Option<ColliderConfig>,
        show_all_colliders: bool,
    ) -> Vec<(ColliderConfig, Color)> {
        let active_collider_color: Color = YELLOW.into();
        let bone_colliders_color: Color = ORANGE.into();
        let bone_symmetry_colliders_color: Color = ORANGE.with_alpha(0.4).into();
        let other_colliders_color: Color = DARK_RED.into();
        let other_symmetry_colliders_color: Color = DARK_RED.with_alpha(0.4).into();

        let mut draw_colliders = Vec::new();

        if let Some(bone_id) = bone_id {
            let bone_colliders = skeleton_colliders
                .get_colliders(bone_id)
                .cloned()
                .unwrap_or_default();

            // Active collider
            if let Some(active_collider) = &active_collider {
                draw_colliders.push((active_collider.clone(), active_collider_color));
            }

            // This bone's colliders
            draw_colliders.extend(
                bone_colliders
                    .iter()
                    .filter(|c| active_collider.as_ref().is_none_or(|cfg| c.id != cfg.id))
                    .cloned()
                    .map(|c| (c, bone_colliders_color)),
            );

            // Other bone's colliders that map to this one via symmetry
            if skeleton_colliders.symmetry_enabled {
                draw_colliders.extend(
                    reverse_index
                        .mapping_to_bone(bone_id)
                        .into_iter()
                        .flat_map(|bone_id| {
                            skeleton_colliders
                                .get_colliders(bone_id)
                                .cloned()
                                .unwrap_or_default()
                        })
                        .map(|mut c| {
                            c.offset.translation = skeleton_colliders
                                .symmetry
                                .mode
                                .apply_position(c.offset.translation.into())
                                .into();
                            c.offset.rotation = skeleton_colliders
                                .symmetry
                                .mode
                                .apply_quat(c.offset.rotation);
                            c.attached_to = bone_id;

                            c
                        })
                        .map(|c| (c, bone_symmetry_colliders_color)),
                );
            }
        }

        // Other bones' colliders not mapping to this bone
        if show_all_colliders {
            for other_bone_id in skeleton.iter_bones().filter(|b| bone_id != Some(*b)) {
                let other_bone_colliders = skeleton_colliders
                    .get_colliders(other_bone_id)
                    .cloned()
                    .unwrap_or_default();

                // Owned colliders
                draw_colliders.extend(
                    other_bone_colliders
                        .iter()
                        .cloned()
                        .map(|c| (c, other_colliders_color)),
                );

                // Colliders applied via symmetry
                if skeleton_colliders.symmetry_enabled {
                    draw_colliders.extend(
                        reverse_index
                            .mapping_to_bone(other_bone_id)
                            .into_iter()
                            .flat_map(|bone_id| {
                                skeleton_colliders
                                    .get_colliders(bone_id)
                                    .cloned()
                                    .unwrap_or_default()
                            })
                            .map(|mut c| {
                                c.offset.translation = skeleton_colliders
                                    .symmetry
                                    .mode
                                    .apply_position(c.offset.translation.into())
                                    .into();
                                c.offset.rotation = skeleton_colliders
                                    .symmetry
                                    .mode
                                    .apply_quat(c.offset.rotation);
                                c.attached_to = other_bone_id;

                                c
                            })
                            .map(|c| (c, other_symmetry_colliders_color)),
                    );
                }
            }
        }

        draw_colliders
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
                clear_color: ClearColorConfig::Custom(Color::from(LinearRgba::new(0., 0., 0., 1.))),
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
    skeletons: Res<Assets<Skeleton>>,
) {
    for (mut player, PartOfSubScene(target_id)) in &mut q_animation_players {
        if id != *target_id {
            continue;
        }

        let Some(skeleton) = skeletons.get(player.skeleton()) else {
            continue;
        };

        let mut drawable = vec![];
        if let Some(clicked) = input.selected {
            drawable.push((clicked, css::LIGHT_SKY_BLUE.into()));
        }

        if let Some(hovered) = input.hovered {
            if input.selected.is_none_or(|b| b != hovered) {
                drawable.push((hovered, GRAY.into()));
            }
        }

        player.gizmo_for_bones_with_color(drawable);

        for (cfg, color) in input.draw_colliders.drain(..) {
            let Some(default_transforms) = skeleton.default_transforms(cfg.attached_to).cloned()
            else {
                continue;
            };

            player.custom_relative_gizmo(CustomRelativeDrawCommand {
                bone_id: cfg.attached_to,
                f: Arc::new(move |bone_transform, gizmos| {
                    let offset_transform =
                        bone_transform * cfg.local_transform(&default_transforms);

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

/// Reverse index mapping bones that map to each bone via symmetry
#[derive(Default, Debug)]
pub struct ReverseSkeletonIndex {
    mapping: HashMap<BoneId, Vec<BoneId>>,
}

impl ReverseSkeletonIndex {
    pub fn new(skeleton: &Skeleton, symmetry: &SymmetryConfig) -> Self {
        let mut mapping = HashMap::default();

        for bone_id in skeleton.iter_bones() {
            let Some(path) = skeleton.id_to_path(bone_id) else {
                continue;
            };

            let target_path = symmetry.name_mapper.flip(&path);
            let Some(target_id) = skeleton.path_to_id(target_path) else {
                continue;
            };

            mapping.entry(target_id).or_insert(Vec::new()).push(bone_id);
        }

        Self { mapping }
    }

    /// Access to the list of bones that map to the current bone under the provided symmetry
    pub fn mapping_to_bone(&self, bone_id: BoneId) -> Vec<BoneId> {
        self.mapping.get(&bone_id).cloned().unwrap_or_default()
    }
}
