use bevy::{
    asset::{Assets, Handle},
    ecs::world::World,
};
use bevy_animation_graph::core::ragdoll::definition::Ragdoll;
use egui::Widget;

use crate::ui::core::EditorWindowContext;

#[derive(Debug, Default)]
pub struct RagdollEditorSettings {
    show_all_colliders: bool,
}

pub struct SettingsPanel<'a, 'b> {
    pub target: Handle<Ragdoll>,
    pub world: &'a mut World,
    pub ctx: &'a mut EditorWindowContext<'b>,
    pub settings: &'a mut RagdollEditorSettings,
}

impl Widget for SettingsPanel<'_, '_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.heading("Ragdoll settings");
        self.world
            .resource_scope::<Assets<Ragdoll>, _>(|_world, ragdoll_assets| {
                let Some(_ragdoll) = ragdoll_assets.get(&self.target) else {
                    return;
                };
            });
        ui.separator();

        ui.heading("Preview settings");
        let show_all_colliders_response =
            ui.checkbox(&mut self.settings.show_all_colliders, "Show all colliders");

        show_all_colliders_response
    }
}
