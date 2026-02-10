use bevy::{
    asset::{AssetServer, Assets, Handle},
    gltf::Gltf,
    prelude::World,
};
use bevy_animation_graph::core::{
    animation_clip::loader::{GraphClipSerial, GraphClipSource},
    skeleton::Skeleton,
};
use egui_dock::egui;

use crate::ui::{
    generic_widgets::{popup_asset_picker::PopupAssetPicker, string_picker::StringPickerWidget},
    native_windows::{EditorWindowContext, NativeEditorWindowExtension},
    state_management::global::{CloseWindow, clip::CreateClip},
    utils::asset_path,
};

#[derive(Debug, Clone, Copy)]
pub struct CreateAnimWindow;

impl NativeEditorWindowExtension for CreateAnimWindow {
    fn ui(&self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        ui.push_id("Graph clip Asset creator", |ui| {
            let mut queue = ctx.make_queue();

            let buffer_id = ui.id().with("buffer");
            let buffer: &mut AnimCreateBuffer = ctx.buffers.get_mut_or_default(buffer_id);

            ui.label("asset path:");
            ui.text_edit_singleline(&mut buffer.path);

            ui.label("skeleton:");
            ui.add(PopupAssetPicker::new_salted(
                &mut buffer.skeleton,
                world,
                "skeleton",
            ));

            ui.label("GLTF file:");
            ui.add(PopupAssetPicker::new_salted(
                &mut buffer.gltf,
                world,
                "gtlf",
            ));

            // get gltf asset
            let gltf_assets = world.resource::<Assets<Gltf>>();
            let anims: Vec<String> = gltf_assets
                .get(&buffer.gltf)
                .map(|gltf| {
                    gltf.named_animations
                        .keys()
                        .map(|k| k.to_string())
                        .collect()
                })
                .unwrap_or_default();

            ui.label("Animation label:");
            ui.add(
                StringPickerWidget::new(&mut buffer.animation, &anims)
                    .salted("animation label picker"),
            );

            let asset_server = world.resource::<AssetServer>();

            ui.add_enabled_ui(validate_buffer(buffer), |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                    if ui.button("Create").clicked() {
                        queue.trigger(CreateClip {
                            virtual_path: buffer.path.clone(),
                            clip_serial: GraphClipSerial {
                                source: GraphClipSource::GltfNamed {
                                    path: asset_path(buffer.gltf.id().untyped(), asset_server),
                                    animation_name: buffer.animation.clone(),
                                },
                                skeleton: asset_path(buffer.skeleton.id().untyped(), asset_server),
                                event_tracks: Default::default(),
                            },
                        });
                        queue.trigger(CloseWindow(queue.window_entity));
                    }
                });
            });

            ctx.consume_queue(queue);
        });
    }

    fn display_name(&self) -> String {
        "Create animation metadata".to_string()
    }
}

fn validate_buffer(buffer: &AnimCreateBuffer) -> bool {
    buffer.gltf != Handle::default()
        && buffer.skeleton != Handle::default()
        && !buffer.animation.is_empty()
        && buffer.path.ends_with(".anim.ron")
}

#[derive(Default)]
pub struct AnimCreateBuffer {
    path: String,
    skeleton: Handle<Skeleton>,
    gltf: Handle<Gltf>,
    animation: String,
}
