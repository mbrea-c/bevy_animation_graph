use bevy::{
    asset::{AssetServer, Assets, Handle},
    gltf::Gltf,
    prelude::World,
};
use bevy_animation_graph::core::skeleton::serial::{SkeletonSerial, SkeletonSource};
use egui_dock::egui;

use crate::ui::{
    generic_widgets::{popup_asset_picker::PopupAssetPicker, string_picker::StringPickerWidget},
    native_windows::{EditorWindowContext, NativeEditorWindowExtension},
    state_management::global::{CloseWindow, skeleton::CreateSkeleton},
    utils::asset_path,
};

#[derive(Debug, Clone, Copy)]
pub struct CreateSkeletonWindow;

impl NativeEditorWindowExtension for CreateSkeletonWindow {
    fn ui(&self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        ui.push_id("Skeleton asset creator", |ui| {
            let mut queue = ctx.make_queue();

            let buffer_id = ui.id().with("buffer");
            let buffer: &mut SkeletonCreateBuffer = ctx.buffers.get_mut_or_default(buffer_id);

            ui.label("asset path:");
            ui.text_edit_singleline(&mut buffer.path);

            ui.label("GLTF file:");
            ui.add(PopupAssetPicker::new_salted(
                &mut buffer.gltf,
                world,
                "gtlf",
            ));

            // get gltf asset
            let gltf_assets = world.resource::<Assets<Gltf>>();
            let labels: Vec<String> = gltf_assets
                .get(&buffer.gltf)
                .map(|gltf| {
                    (0..gltf.scenes.len())
                        .map(|k| format!("Scene{}", k))
                        .collect()
                })
                .unwrap_or_default();

            ui.label("Scene label:");
            ui.add(
                StringPickerWidget::new(&mut buffer.label, &labels).salted("scene label picker"),
            );

            let asset_server = world.resource::<AssetServer>();

            ui.add_enabled_ui(validate_buffer(buffer), |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                    if ui.button("Create").clicked() {
                        queue.trigger(CreateSkeleton {
                            virtual_path: buffer.path.clone(),
                            skeleton: SkeletonSerial {
                                source: SkeletonSource::Gltf {
                                    source: asset_path(buffer.gltf.id().untyped(), asset_server),
                                    label: buffer.label.clone(),
                                },
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
        "Create skeleton".to_string()
    }
}

fn validate_buffer(buffer: &SkeletonCreateBuffer) -> bool {
    buffer.gltf != Handle::default()
        && !buffer.label.is_empty()
        && buffer.path.ends_with(".skn.ron")
}

#[derive(Default)]
pub struct SkeletonCreateBuffer {
    path: String,
    gltf: Handle<Gltf>,
    label: String,
}
