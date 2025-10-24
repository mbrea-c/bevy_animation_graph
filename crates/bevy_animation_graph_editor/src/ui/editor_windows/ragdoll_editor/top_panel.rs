use bevy::{asset::Handle, ecs::world::World};
use bevy_animation_graph::{
    core::ragdoll::{bone_mapping::RagdollBoneMap, definition::Ragdoll},
    prelude::AnimatedScene,
};

use crate::ui::{
    core::LegacyEditorWindowContext, editor_windows::ragdoll_editor::RagdollEditorAction,
    reflect_widgets::wrap_ui::using_wrap_ui,
};

pub struct TopPanel<'a, 'b> {
    pub ragdoll: Option<Handle<Ragdoll>>,
    pub ragdoll_bone_map: Option<Handle<RagdollBoneMap>>,
    pub scene: Option<Handle<AnimatedScene>>,
    pub world: &'a mut World,
    pub ctx: &'a mut LegacyEditorWindowContext<'b>,
}

impl TopPanel<'_, '_> {
    pub fn draw(self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Scene:");
            using_wrap_ui(self.world, |mut env| {
                if let Some(new_handle) = env.mutable_buffered(
                    &self.scene.clone().unwrap_or_default(),
                    ui,
                    ui.id().with("ragdoll base scene selector"),
                    &(),
                ) {
                    self.ctx
                        .window_action(RagdollEditorAction::SelectBaseScene(new_handle));
                }
            });

            ui.label("Ragdoll:");
            using_wrap_ui(self.world, |mut env| {
                if let Some(new_handle) = env.mutable_buffered(
                    &self.ragdoll.clone().unwrap_or_default(),
                    ui,
                    ui.id().with("ragdoll selectors"),
                    &(),
                ) {
                    self.ctx
                        .window_action(RagdollEditorAction::SelectRagdoll(new_handle));
                }
            });

            ui.label("Ragdoll bone map:");
            using_wrap_ui(self.world, |mut env| {
                if let Some(new_handle) = env.mutable_buffered(
                    &self.ragdoll_bone_map.clone().unwrap_or_default(),
                    ui,
                    ui.id().with("ragdoll bone map selectors"),
                    &(),
                ) {
                    self.ctx
                        .window_action(RagdollEditorAction::SelectRagdollBoneMap(new_handle));
                }
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("⚙").clicked() {
                    self.ctx
                        .window_action(RagdollEditorAction::ToggleSettingsWindow);
                }
            });
        });
    }
}
