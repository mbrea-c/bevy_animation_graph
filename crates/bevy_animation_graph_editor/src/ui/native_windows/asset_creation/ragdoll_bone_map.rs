use bevy::{
    asset::{AssetServer, Handle},
    prelude::World,
};
use bevy_animation_graph::core::{
    ragdoll::{bone_mapping_loader::RagdollBoneMapSerial, definition::Ragdoll},
    skeleton::Skeleton,
};
use egui_dock::egui;

use crate::ui::{
    generic_widgets::popup_asset_picker::PopupAssetPicker,
    native_windows::{EditorWindowContext, NativeEditorWindowExtension},
    state_management::global::{CloseWindow, ragdoll_bone_map::CreateRagdollBoneMap},
    utils::asset_path,
};

#[derive(Debug, Clone, Copy)]
pub struct CreateRagdollBoneMapWindow;

impl NativeEditorWindowExtension for CreateRagdollBoneMapWindow {
    fn ui(&self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        ui.push_id("Ragdoll bone map asset creator", |ui| {
            let mut queue = ctx.make_queue();

            let buffer_id = ui.id().with("buffer");
            let buffer: &mut BoneMapCreateBuffer = ctx.buffers.get_mut_or_default(buffer_id);

            ui.label("asset path:");
            ui.text_edit_singleline(&mut buffer.path);

            ui.label("skeleton:");
            ui.add(PopupAssetPicker::new_salted(
                &mut buffer.skeleton,
                world,
                "skeleton",
            ));

            ui.label("ragdoll:");
            ui.add(PopupAssetPicker::new_salted(
                &mut buffer.ragdoll,
                world,
                "ragdoll",
            ));

            let asset_server = world.resource::<AssetServer>();

            ui.add_enabled_ui(validate_buffer(buffer), |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                    if ui.button("Create").clicked() {
                        queue.trigger(CreateRagdollBoneMap {
                            virtual_path: buffer.path.clone(),
                            ragdoll_bone_map: RagdollBoneMapSerial {
                                skeleton: asset_path(buffer.skeleton.id().untyped(), asset_server),
                                ragdoll: asset_path(buffer.ragdoll.id().untyped(), asset_server),
                                ..Default::default()
                            },
                        });
                        queue.trigger_window(CloseWindow::default());
                    }
                });
            });

            ctx.consume_queue(queue);
        });
    }

    fn display_name(&self) -> String {
        "Create ragdoll bone map".to_string()
    }
}

fn validate_buffer(buffer: &BoneMapCreateBuffer) -> bool {
    buffer.ragdoll != Handle::default()
        && buffer.skeleton != Handle::default()
        && buffer.path.ends_with(".bm.ron")
}

#[derive(Default)]
pub struct BoneMapCreateBuffer {
    path: String,
    skeleton: Handle<Skeleton>,
    ragdoll: Handle<Ragdoll>,
}
