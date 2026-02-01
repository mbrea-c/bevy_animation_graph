use std::{any::TypeId, path::PathBuf};

use bevy::{
    asset::{AssetPath, UntypedAssetId},
    prelude::World,
};
use bevy_animation_graph::core::{
    animation_clip::GraphClip,
    animation_graph::AnimationGraph,
    ragdoll::{bone_mapping::RagdollBoneMap, definition::Ragdoll},
    state_machine::high_level::StateMachine,
};
use egui_dock::egui;

use crate::ui::{
    actions::{
        EditorAction,
        saving::{SaveAction, SaveMultiple},
        window::CloseWindowAction,
    },
    core::{EditorWindowExtension, LegacyEditorWindowContext},
    reflect_widgets::wrap_ui::using_wrap_ui,
};

#[derive(Debug)]
pub struct SaveWindowAssetMeta {
    pub id: UntypedAssetId,
    pub should_save: bool,
    pub should_rename: bool,
    pub virtual_path: PathBuf,
    pub current_path: Option<AssetPath<'static>>,
}

#[derive(Debug)]
pub struct SaveWindow {
    assets: Vec<SaveWindowAssetMeta>,
}

impl SaveWindow {
    pub fn new(mut assets: Vec<SaveWindowAssetMeta>) -> Self {
        assets.sort_by_key(|a| (a.id.type_id(), a.id));
        Self { assets }
    }
}

impl EditorWindowExtension for SaveWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut LegacyEditorWindowContext) {
        for meta in &mut self.assets {
            ui.label(Self::displays_type_name(meta.id.type_id()));
            ui.horizontal(|ui| {
                ui.checkbox(&mut meta.should_save, "Save");
                if let Some(path) = &meta.current_path {
                    ui.label(format!("{path}"));
                } else {
                    ui.label("<new asset>");
                }
            });
            ui.horizontal(|ui| {
                ui.add_enabled_ui(meta.current_path.is_some(), |ui| {
                    ui.checkbox(&mut meta.should_rename, "Rename");
                });
                if meta.should_rename {
                    using_wrap_ui(world, |mut env| {
                        if let Some(new_path) =
                            env.mutable_buffered(&meta.virtual_path, ui, ui.id(), &())
                        {
                            meta.virtual_path = new_path;
                        }
                    });
                }
            });
            ui.separator();
        }

        if ui.button("Save").clicked() {
            ctx.editor_actions
                .push(EditorAction::Save(SaveAction::Multiple(SaveMultiple {
                    assets: self
                        .assets
                        .iter()
                        .filter(|meta| meta.should_save)
                        .map(|meta| {
                            (
                                meta.id,
                                if meta.should_rename {
                                    meta.virtual_path.clone()
                                } else {
                                    meta.current_path
                                        .clone()
                                        .unwrap_or_default()
                                        .path()
                                        .to_path_buf()
                                },
                            )
                        })
                        .collect(),
                })));

            ctx.editor_actions
                .dynamic(CloseWindowAction { id: ctx.window_id });
        }
    }

    fn display_name(&self) -> String {
        "Save".to_string()
    }

    fn closeable(&self) -> bool {
        true
    }
}

impl SaveWindow {
    fn displays_type_name(type_id: TypeId) -> String {
        if type_id == TypeId::of::<AnimationGraph>() {
            "Animation Graph".into()
        } else if type_id == TypeId::of::<StateMachine>() {
            "State Machine".into()
        } else if type_id == TypeId::of::<GraphClip>() {
            "Animation Clip".into()
        } else if type_id == TypeId::of::<Ragdoll>() {
            "Ragdoll".into()
        } else if type_id == TypeId::of::<RagdollBoneMap>() {
            "Ragdoll Bone Map".into()
        } else {
            "Unknown type (?)".into()
        }
    }
}
