use bevy::prelude::World;
use egui_dock::egui;

use crate::ui::{
    native_windows::{EditorWindowContext, NativeEditorWindowExtension, OwnedQueue},
    state_management::global::CloseWindow,
};

pub trait CreateAssetFromPath {
    const NAME: &'static str;
    const EXTENSION: &'static str;

    fn create(&self, path: String, queue: &mut OwnedQueue);
}

impl<W: CreateAssetFromPath + std::fmt::Debug + Send + Sync + 'static> NativeEditorWindowExtension
    for W
{
    fn ui(&self, ui: &mut egui::Ui, _: &mut World, ctx: &mut EditorWindowContext) {
        ui.push_id(Self::NAME, |ui| {
            let mut queue = ctx.make_queue();

            let buffer_id = ui.id().with("buffer");
            let path: &mut String = ctx.buffers.get_mut_or_default(buffer_id);

            ui.label("Asset path");
            ui.text_edit_singleline(path);

            ui.add_enabled_ui(path.ends_with(Self::EXTENSION), |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                    if ui.button("Create").clicked() {
                        self.create(path.clone(), &mut queue);
                        queue.trigger(CloseWindow(queue.window_entity));
                    }
                });
            });

            ctx.consume_queue(queue);
        });
    }

    fn display_name(&self) -> String {
        Self::NAME.into()
    }
}
