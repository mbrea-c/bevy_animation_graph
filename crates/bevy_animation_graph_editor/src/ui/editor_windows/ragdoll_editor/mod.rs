use bevy::{
    asset::Handle,
    color::{
        Alpha, Color,
        palettes::css::{DARK_RED, ORANGE, YELLOW},
    },
    platform::collections::HashMap,
    prelude::World,
    utils::default,
};
use bevy_animation_graph::{
    core::{
        colliders::core::{ColliderConfig, SkeletonColliderId, SkeletonColliders},
        id::BoneId,
        ragdoll::definition::{Body, BodyId, Ragdoll},
        skeleton::Skeleton,
    },
    prelude::{AnimatedScene, config::SymmetryConfig},
};
use egui_dock::egui;

use crate::{
    tree::RagdollNode,
    ui::{
        actions::{ragdoll::EditRagdollBody, window::DynWindowAction},
        core::{EditorWindowContext, EditorWindowExtension},
        editor_windows::ragdoll_editor::{
            body_inspector::BodyInspector,
            body_tree::BodyTree,
            ragdoll_preview::RagdollPreview,
            settings_panel::{RagdollEditorSettings, SettingsPanel},
            top_panel::TopPanel,
        },
        reflect_widgets::wrap_ui::using_wrap_ui,
        utils::{OrbitView, with_assets_all},
    },
};

mod body_inspector;
mod body_tree;
mod ragdoll_preview;
mod settings_panel;
mod top_panel;

#[derive(Debug)]
pub struct RagdollEditorWindow {
    pub orbit_view: OrbitView,
    pub target: Option<Handle<Ragdoll>>,
    pub reverse_index: Option<ReverseSkeletonIndex>,
    pub base_scene: Option<Handle<AnimatedScene>>,
    pub hovered: Option<BoneId>,
    pub selected: Option<BoneId>,
    pub selected_node: Option<RagdollNode>,
    pub body_edit_buffers: HashMap<BodyId, Body>,
    pub selectable_collider: SelectableCollider,
    pub draw_colliders: Vec<(ColliderConfig, Color)>,
    pub show_global_settings: bool,
    pub show_all_colliders: bool,
    pub settings: RagdollEditorSettings,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum SelectableCollider {
    #[default]
    None,
    New,
    Existing(SkeletonColliderId),
}

impl Default for RagdollEditorWindow {
    fn default() -> Self {
        Self {
            orbit_view: OrbitView::default(),
            target: None,
            reverse_index: None,
            base_scene: None,
            hovered: None,
            selected: None,
            selected_node: None,
            selectable_collider: SelectableCollider::default(),
            body_edit_buffers: HashMap::default(),
            draw_colliders: Vec::default(),
            show_global_settings: false,
            show_all_colliders: true,
            settings: default(),
        }
    }
}

#[derive(Debug)]
pub enum RagdollEditorAction {
    SelectBaseScene(Handle<AnimatedScene>),
    SelectRagdoll(Handle<Ragdoll>),
    SelectNode(RagdollNode),
    ResetBodyBuffers,
    ToggleSettingsWindow,
}

impl EditorWindowExtension for RagdollEditorWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        self.top_panel(ui, world, ctx);
        self.left_panel(ui, world, ctx);
        self.right_panel(ui, world, ctx);
        self.settings_popup(ui, world, ctx);
        self.center_panel(ui, world, ctx);
    }

    fn display_name(&self) -> String {
        "Clip Preview".to_string()
    }

    fn handle_action(&mut self, action: DynWindowAction) {
        let Ok(action) = action.downcast::<RagdollEditorAction>() else {
            return;
        };

        match *action {
            RagdollEditorAction::SelectBaseScene(handle) => {
                self.base_scene = Some(handle);
            }
            RagdollEditorAction::SelectRagdoll(handle) => {
                self.target = Some(handle);
                self.reverse_index = None;
            }
            RagdollEditorAction::SelectNode(ragdoll_node) => {
                self.selected_node = Some(ragdoll_node);
            }
            RagdollEditorAction::ToggleSettingsWindow => {
                self.show_global_settings = !self.show_global_settings;
            }
            RagdollEditorAction::ResetBodyBuffers => self.body_edit_buffers.clear(),
        }
    }
}

impl RagdollEditorWindow {
    pub fn top_panel(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut World,
        ctx: &mut EditorWindowContext,
    ) {
        let timeline_height = 30.;

        egui::TopBottomPanel::top("Top panel")
            .resizable(false)
            .exact_height(timeline_height)
            .frame(egui::Frame::NONE.inner_margin(5.))
            .show_inside(ui, |ui| {
                TopPanel {
                    ragdoll: self.target.clone(),
                    scene: self.base_scene.clone(),
                    world,
                    ctx,
                }
                .draw(ui);
            });
    }

    pub fn left_panel(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut World,
        ctx: &mut EditorWindowContext,
    ) {
        egui::SidePanel::left("Hierarchical tree view")
            .resizable(true)
            .default_width(300.)
            .show_inside(ui, |ui| {
                egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                    if let Some(ragdoll) = self.target.clone() {
                        BodyTree {
                            ragdoll,
                            world,
                            ctx,
                        }
                        .draw(ui);
                    }
                });
            });
    }

    pub fn right_panel(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut World,
        ctx: &mut EditorWindowContext,
    ) {
        egui::SidePanel::right("Inspector panel")
            .resizable(true)
            .default_width(300.)
            .show_inside(ui, |ui| {
                egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                    match self.selected_node {
                        Some(RagdollNode::Body(body_id)) => {
                            if let Some(ragdoll_handle) = &self.target {
                                with_assets_all(
                                    world,
                                    [ragdoll_handle.id()],
                                    |world, [ragdoll]| {
                                        let buffer = self
                                            .body_edit_buffers
                                            .entry(body_id)
                                            .or_insert_with(|| {
                                                ragdoll
                                                    .get_body(body_id)
                                                    .cloned()
                                                    .unwrap_or_else(|| Body::new())
                                            });

                                        let _response = ui.add(BodyInspector {
                                            world,
                                            ctx,
                                            body: buffer,
                                        });

                                        if ui.button("Apply").clicked() {
                                            ctx.editor_actions.dynamic(EditRagdollBody {
                                                ragdoll: ragdoll_handle.clone(),
                                                body: buffer.clone(),
                                            });
                                            ctx.window_action(
                                                RagdollEditorAction::ResetBodyBuffers,
                                            );
                                        }
                                    },
                                );
                            }
                        }
                        Some(RagdollNode::Collider(collider_id)) => {}
                        Some(RagdollNode::Joint(joint_id)) => {}
                        None => {
                            ui.label("Nothing is selected");
                        }
                    }
                });
            });
    }

    pub fn center_panel(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut World,
        ctx: &mut EditorWindowContext,
    ) {
        if let Some(base_scene) = &self.base_scene
            && let Some(ragdoll) = &self.target
        {
            RagdollPreview {
                world,
                ctx,
                orbit_view: &mut self.orbit_view,
                ragdoll: ragdoll.clone(),
                base_scene: base_scene.clone(),
                body_buffers: self.body_edit_buffers.clone(),
            }
            .draw(ui);
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("No base scene selected");
            });
        }
    }

    pub fn settings_popup(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut World,
        ctx: &mut EditorWindowContext,
    ) {
        if self.show_global_settings {
            egui::Window::new("Skeleton collider settings").show(ui.ctx(), |ui| {
                if let Some(target) = &self.target {
                    ui.add(SettingsPanel {
                        target: target.clone(),
                        world,
                        ctx,
                        settings: &mut self.settings,
                    });
                }
            });
        }
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
