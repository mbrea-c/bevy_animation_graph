use bevy::{
    asset::{Assets, Handle},
    ecs::world::World,
};
use bevy_animation_graph::core::ragdoll::definition::Ragdoll;
use egui::Widget;

use crate::ui::core::LegacyEditorWindowContext;

#[derive(Debug, Default)]
pub struct RagdollEditorSettings {
    show_all_colliders: bool,
}

pub struct SettingsPanel<'a, 'b> {
    pub target: Handle<Ragdoll>,
    pub world: &'a mut World,
    pub ctx: &'a mut LegacyEditorWindowContext<'b>,
    pub settings: &'a mut RagdollEditorSettings,
}

impl Widget for SettingsPanel<'_, '_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut response = ui.heading("Ragdoll settings");
        self.world
            .resource_scope::<Assets<Ragdoll>, _>(|_world, ragdoll_assets| {
                let Some(_ragdoll) = ragdoll_assets.get(&self.target) else {
                    return;
                };
            });
        response |= ui.separator();

        response |= ui.heading("Preview settings");
        response |= ui.checkbox(&mut self.settings.show_all_colliders, "Show all colliders");

        response
    }
}
