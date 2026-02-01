use bevy::{
    asset::{AssetId, Handle},
    platform::collections::HashMap,
    prelude::World,
    utils::default,
};
use bevy_animation_graph::core::{
    animated_scene::AnimatedScene,
    id::BoneId,
    ragdoll::{
        bone_mapping::{BodyMapping, BoneMapping, BoneWeight, RagdollBoneMap},
        definition::{Body, BodyId, Collider, ColliderId, Joint, JointId, Ragdoll},
    },
    skeleton::Skeleton,
};
use egui_dock::egui;

use crate::ui::{
    actions::{
        ragdoll::{
            CreateOrEditBodyMapping, CreateOrEditBoneMapping, EditRagdollBody, EditRagdollCollider,
            EditRagdollJoint, RecomputeMappingOffsets, RecomputeRagdollSymmetry,
        },
        window::DynWindowAction,
    },
    core::{EditorWindowExtension, LegacyEditorWindowContext},
    editor_windows::ragdoll_editor::{
        body_inspector::BodyInspector,
        body_mapping_inspector::BodyMappingInspector,
        body_tree::BodyTree,
        bone_mapping_inspector::BoneMappingInspector,
        bone_tree::BoneTree,
        collider_inspector::ColliderInspector,
        joint_inspector::JointInspector,
        ragdoll_preview::RagdollPreview,
        settings_panel::{RagdollEditorSettings, SettingsPanel},
        top_panel::TopPanel,
    },
    utils::with_assets_all,
};

mod body_inspector;
mod body_mapping_inspector;
mod body_tree;
mod bone_mapping_inspector;
mod bone_tree;
mod collider_inspector;
mod joint_inspector;
mod ragdoll_preview;
mod settings_panel;
mod top_panel;

#[derive(Debug)]
pub struct RagdollEditorWindow {
    pub ragdoll: Option<Handle<Ragdoll>>,
    pub ragdoll_bone_map: Option<Handle<RagdollBoneMap>>,
    pub scene: Option<Handle<AnimatedScene>>,
    /// If true, render the skeleton tree. If false, render the ragdoll tree
    pub show_bone_tree: bool,
    pub selected_item: Option<SelectedItem>,
    pub hovered_item: Option<SelectedItem>,

    pub body_edit_buffers: HashMap<BodyId, Body>,
    pub collider_edit_buffers: HashMap<ColliderId, Collider>,
    pub joint_edit_buffers: HashMap<JointId, Joint>,
    pub bone_mapping_buffers: HashMap<BoneId, BoneMapping>,
    pub body_mapping_buffers: HashMap<BodyId, BodyMapping>,

    pub show_global_settings: bool,
    pub settings: RagdollEditorSettings,
}

impl Default for RagdollEditorWindow {
    fn default() -> Self {
        Self {
            ragdoll: None,
            ragdoll_bone_map: None,
            scene: None,
            show_bone_tree: false,
            selected_item: None,
            hovered_item: None,

            body_edit_buffers: HashMap::default(),
            collider_edit_buffers: HashMap::default(),
            joint_edit_buffers: HashMap::default(),
            bone_mapping_buffers: HashMap::default(),
            body_mapping_buffers: HashMap::default(),

            show_global_settings: false,
            settings: default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SelectedItem {
    Body(BodyId),
    Collider(ColliderId),
    Joint(JointId),
    Bone(BoneId),
}

impl SelectedItem {
    pub fn body(&self) -> Option<BodyId> {
        match self {
            SelectedItem::Body(id) => Some(*id),
            _ => None,
        }
    }
    pub fn collider(&self) -> Option<ColliderId> {
        match self {
            SelectedItem::Collider(id) => Some(*id),
            _ => None,
        }
    }
    pub fn joint(&self) -> Option<JointId> {
        match self {
            SelectedItem::Joint(id) => Some(*id),
            _ => None,
        }
    }
    pub fn bone(&self) -> Option<BoneId> {
        match self {
            SelectedItem::Bone(id) => Some(*id),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum RagdollEditorAction {
    SelectBaseScene(Handle<AnimatedScene>),
    SelectRagdoll(Handle<Ragdoll>),
    SelectRagdollBoneMap(Handle<RagdollBoneMap>),
    SelectNode(SelectedItem),
    HoverNode(SelectedItem),
    ResetBuffers,
    ToggleSettingsWindow,
}

impl EditorWindowExtension for RagdollEditorWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut LegacyEditorWindowContext) {
        self.top_panel(ui, world, ctx);
        self.left_panel(ui, world, ctx);
        self.right_panel(ui, world, ctx);
        self.settings_popup(ui, world);
        self.center_panel(ui, world);

        self.hovered_item = None;
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
                self.scene = Some(handle);
            }
            RagdollEditorAction::SelectRagdoll(handle) => {
                self.ragdoll = Some(handle);
            }
            RagdollEditorAction::SelectNode(ragdoll_node) => {
                self.selected_item = Some(ragdoll_node);
            }
            RagdollEditorAction::HoverNode(hovered_item) => {
                self.hovered_item = Some(hovered_item);
            }
            RagdollEditorAction::ToggleSettingsWindow => {
                self.show_global_settings = !self.show_global_settings;
            }
            RagdollEditorAction::ResetBuffers => {
                self.body_edit_buffers.clear();
                self.collider_edit_buffers.clear();
                self.joint_edit_buffers.clear();
            }
            RagdollEditorAction::SelectRagdollBoneMap(handle) => {
                self.ragdoll_bone_map = Some(handle);
            }
        }
    }
}

impl RagdollEditorWindow {
    pub fn top_panel(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut World,
        ctx: &mut LegacyEditorWindowContext,
    ) {
        let timeline_height = 30.;

        egui::TopBottomPanel::top("Top panel")
            .resizable(false)
            .exact_height(timeline_height)
            .frame(egui::Frame::NONE.inner_margin(5.))
            .show_inside(ui, |ui| {
                TopPanel {
                    ragdoll: self.ragdoll.clone(),
                    scene: self.scene.clone(),
                    ragdoll_bone_map: self.ragdoll_bone_map.clone(),
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
        ctx: &mut LegacyEditorWindowContext,
    ) {
        egui::SidePanel::left("Hierarchical tree view")
            .resizable(true)
            .default_width(300.)
            .show_inside(ui, |ui| {
                ui.checkbox(&mut self.show_bone_tree, "Show skeleton tree");
                egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                    if self.show_bone_tree {
                        if let Some(animscn) = self.scene.clone() {
                            with_assets_all(world, [animscn.id()], |world, [animscn]| {
                                BoneTree {
                                    skeleton: animscn.skeleton.clone(),
                                    world,
                                    ctx,
                                }
                                .draw(ui);
                            });
                        }
                    } else if let Some(ragdoll) = self.ragdoll.clone()
                        && let Some(ragdoll_bone_map) = self.ragdoll_bone_map.clone()
                    {
                        BodyTree {
                            ragdoll,
                            ragdoll_bone_map,
                            world,
                            ctx,
                        }
                        .draw(ui);
                    }
                });
            });
    }

    fn submit_row(
        ui: &mut egui::Ui,
        ctx: &mut LegacyEditorWindowContext,
        on_submit: impl FnOnce(&mut LegacyEditorWindowContext),
    ) {
        ui.horizontal(|ui| {
            if ui.button("Apply").clicked() {
                on_submit(ctx);
                ctx.window_action(RagdollEditorAction::ResetBuffers);
            }
            if ui.button("Reset").clicked() {
                ctx.window_action(RagdollEditorAction::ResetBuffers);
            }
        });
    }

    pub fn right_panel(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut World,
        ctx: &mut LegacyEditorWindowContext,
    ) {
        egui::SidePanel::right("Inspector panel")
            .resizable(true)
            .default_width(350.)
            .show_inside(ui, |ui| {
                egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                    match self.selected_item {
                        Some(SelectedItem::Body(body_id)) => {
                            if let Some(ragdoll_handle) = &self.ragdoll
                                && let Some(ragdoll_bone_map_handle) = &self.ragdoll_bone_map
                            {
                                with_assets_all(world, [ragdoll_handle.id()], |_, [ragdoll]| {
                                    let buffer = self
                                        .body_edit_buffers
                                        .entry(body_id)
                                        .or_insert_with(|| {
                                            ragdoll
                                                .get_body(body_id)
                                                .cloned()
                                                .unwrap_or_else(Body::new)
                                        });

                                    let _response = ui.add(BodyInspector { body: buffer });

                                    Self::submit_row(ui, ctx, |ctx| {
                                        ctx.editor_actions.dynamic(EditRagdollBody {
                                            ragdoll: ragdoll_handle.clone(),
                                            body: buffer.clone(),
                                        });
                                        ctx.editor_actions.dynamic(RecomputeMappingOffsets {
                                            ragdoll_bone_map: ragdoll_bone_map_handle.clone(),
                                        });
                                        ctx.editor_actions.dynamic(RecomputeRagdollSymmetry {
                                            ragdoll_bone_map: ragdoll_bone_map_handle.clone(),
                                        });
                                    });
                                });
                            }

                            if let Some(animscn_handle) = &self.scene
                                && let Some(bone_map_handle) = &self.ragdoll_bone_map
                            {
                                with_body_mapping_assets(
                                    world,
                                    animscn_handle.id(),
                                    bone_map_handle.id(),
                                    |_, _, skeleton, bone_map| {
                                        ui.separator();

                                        let body_mapping = self
                                            .body_mapping_buffers
                                            .entry(body_id)
                                            .or_insert_with(|| {
                                                bone_map
                                                    .bodies_from_bones
                                                    .get(&body_id)
                                                    .cloned()
                                                    .unwrap_or(BodyMapping {
                                                        body_id,
                                                        bone: BoneWeight::default(),
                                                        created_from: None,
                                                    })
                                            });
                                        ui.add(BodyMappingInspector {
                                            body_mapping,
                                            skeleton,
                                        });

                                        Self::submit_row(ui, ctx, |ctx| {
                                            ctx.editor_actions.dynamic(CreateOrEditBodyMapping {
                                                ragdoll_bone_map: bone_map_handle.clone(),
                                                body_mapping: body_mapping.clone(),
                                            });
                                            ctx.editor_actions.dynamic(RecomputeMappingOffsets {
                                                ragdoll_bone_map: bone_map_handle.clone(),
                                            });
                                            ctx.editor_actions.dynamic(RecomputeRagdollSymmetry {
                                                ragdoll_bone_map: bone_map_handle.clone(),
                                            });
                                        });
                                    },
                                );
                            }
                        }
                        Some(SelectedItem::Collider(collider_id)) => {
                            if let Some(ragdoll_handle) = &self.ragdoll
                                && let Some(bone_map_handle) = &self.ragdoll_bone_map
                            {
                                with_assets_all(
                                    world,
                                    [ragdoll_handle.id()],
                                    |world, [ragdoll]| {
                                        let buffer = self
                                            .collider_edit_buffers
                                            .entry(collider_id)
                                            .or_insert_with(|| {
                                                ragdoll
                                                    .get_collider(collider_id)
                                                    .cloned()
                                                    .unwrap_or_else(Collider::new)
                                            });

                                        let _response = ui.add(ColliderInspector {
                                            world,
                                            collider: buffer,
                                        });

                                        Self::submit_row(ui, ctx, |ctx| {
                                            ctx.editor_actions.dynamic(EditRagdollCollider {
                                                ragdoll: ragdoll_handle.clone(),
                                                collider: buffer.clone(),
                                            });
                                            ctx.editor_actions.dynamic(RecomputeRagdollSymmetry {
                                                ragdoll_bone_map: bone_map_handle.clone(),
                                            });
                                        });
                                    },
                                );
                            }
                        }
                        Some(SelectedItem::Joint(joint_id)) => {
                            if let Some(ragdoll_handle) = &self.ragdoll
                                && let Some(bone_map_handle) = &self.ragdoll_bone_map
                            {
                                with_assets_all(world, [ragdoll_handle.id()], |_, [ragdoll]| {
                                    let buffer = self
                                        .joint_edit_buffers
                                        .entry(joint_id)
                                        .or_insert_with(|| {
                                            ragdoll
                                                .get_joint(joint_id)
                                                .cloned()
                                                .unwrap_or_else(Joint::new)
                                        });

                                    let _response = ui.add(JointInspector {
                                        joint: buffer,
                                        ragdoll,
                                    });

                                    Self::submit_row(ui, ctx, |ctx| {
                                        ctx.editor_actions.dynamic(EditRagdollJoint {
                                            ragdoll: ragdoll_handle.clone(),
                                            joint: buffer.clone(),
                                        });
                                        ctx.editor_actions.dynamic(RecomputeRagdollSymmetry {
                                            ragdoll_bone_map: bone_map_handle.clone(),
                                        });
                                    });
                                });
                            }
                        }
                        Some(SelectedItem::Bone(bone_id)) => {
                            if let Some(animscn_handle) = &self.scene
                                && let Some(bone_map_handle) = &self.ragdoll_bone_map
                                && let Some(ragdoll_handle) = &self.ragdoll
                            {
                                with_bone_mapping_assets(
                                    world,
                                    animscn_handle.id(),
                                    bone_map_handle.id(),
                                    ragdoll_handle.id(),
                                    |_, _, skeleton, bone_map, ragdoll| {
                                        if let Some(entity_path) = skeleton.id_to_path(bone_id) {
                                            let bone = self
                                                .bone_mapping_buffers
                                                .entry(bone_id)
                                                .or_insert_with(|| {
                                                    bone_map
                                                        .bones_from_bodies
                                                        .get(&entity_path)
                                                        .cloned()
                                                        .unwrap_or(BoneMapping {
                                                            bone_id: entity_path.clone(),
                                                            bodies: Vec::new(),
                                                            created_from: None,
                                                        })
                                                });

                                            ui.add(BoneMappingInspector { bone, ragdoll });

                                            Self::submit_row(ui, ctx, |ctx| {
                                                ctx.editor_actions.dynamic(
                                                    CreateOrEditBoneMapping {
                                                        ragdoll_bone_map: bone_map_handle.clone(),
                                                        bone_mapping: bone.clone(),
                                                    },
                                                );
                                                ctx.editor_actions.dynamic(
                                                    RecomputeMappingOffsets {
                                                        ragdoll_bone_map: bone_map_handle.clone(),
                                                    },
                                                );
                                                ctx.editor_actions.dynamic(
                                                    RecomputeRagdollSymmetry {
                                                        ragdoll_bone_map: bone_map_handle.clone(),
                                                    },
                                                );
                                            });
                                        }
                                    },
                                );
                            }
                        }
                        _ => {
                            ui.label("Nothing is selected");
                        }
                    }
                });
            });
    }

    pub fn center_panel(&mut self, ui: &mut egui::Ui, world: &mut World) {
        if let Some(base_scene) = &self.scene
            && let Some(ragdoll) = &self.ragdoll
        {
            RagdollPreview {
                world,
                ragdoll: ragdoll.clone(),
                base_scene: base_scene.clone(),
                body_buffers: self.body_edit_buffers.clone(),
                collider_buffers: self.collider_edit_buffers.clone(),
                joint_buffers: self.joint_edit_buffers.clone(),
                hovered_item: self.hovered_item.clone(),
                selected_item: self.selected_item.clone(),
            }
            .draw(ui);
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("No base scene selected");
            });
        }
    }

    pub fn settings_popup(&mut self, ui: &mut egui::Ui, world: &mut World) {
        if self.show_global_settings {
            egui::Window::new("Skeleton collider settings").show(ui.ctx(), |ui| {
                if let Some(target) = &self.ragdoll {
                    ui.add(SettingsPanel {
                        target: target.clone(),
                        world,
                        settings: &mut self.settings,
                    });
                }
            });
        }
    }
}

fn with_body_mapping_assets(
    world: &mut World,
    animscn_id: AssetId<AnimatedScene>,
    bone_map_id: AssetId<RagdollBoneMap>,
    f: impl FnOnce(&mut World, &AnimatedScene, &Skeleton, &RagdollBoneMap),
) {
    with_assets_all(world, [animscn_id], |world, [animscn]| {
        with_assets_all(world, [animscn.skeleton.id()], |world, [skeleton]| {
            with_assets_all(world, [bone_map_id], |world, [bone_map]| {
                f(world, animscn, skeleton, bone_map);
            });
        });
    });
}

fn with_bone_mapping_assets(
    world: &mut World,
    animscn_id: AssetId<AnimatedScene>,
    bone_map_id: AssetId<RagdollBoneMap>,
    ragdoll_id: AssetId<Ragdoll>,
    f: impl FnOnce(&mut World, &AnimatedScene, &Skeleton, &RagdollBoneMap, &Ragdoll),
) {
    with_assets_all(world, [animscn_id], |world, [animscn]| {
        with_assets_all(world, [animscn.skeleton.id()], |world, [skeleton]| {
            with_assets_all(world, [bone_map_id], |world, [bone_map]| {
                with_assets_all(world, [ragdoll_id], |world, [ragdoll]| {
                    f(world, animscn, skeleton, bone_map, ragdoll);
                });
            });
        });
    });
}
