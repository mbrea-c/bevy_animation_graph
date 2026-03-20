use bevy::{
    asset::{Assets, Handle},
    ecs::world::World,
};
use bevy_animation_graph::core::ragdoll::definition::Ragdoll;
use egui::Widget;

use crate::ui::reflect_lib::ReflectWidgetContext;

#[derive(Debug, Default)]
pub struct RagdollEditorSettings {
    show_all_colliders: bool,
}

pub struct SettingsPanel<'a> {
    pub target: Handle<Ragdoll>,
    pub world: &'a mut World,
    pub settings: &'a mut RagdollEditorSettings,
}

impl Widget for SettingsPanel<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut response = ui.heading("Ragdoll settings");
        self.world
            .resource_scope::<Assets<Ragdoll>, _>(|world, mut ragdoll_assets| {
                let Some(ragdoll) = ragdoll_assets.get_mut(&self.target) else {
                    return;
                };
                response |=
                    ReflectWidgetContext::scope(world, |ctx| ctx.draw(ui, &mut ragdoll.total_mass));
            });
        response |= ui.separator();

        response |= ui.heading("Preview settings");
        response |= ui.checkbox(&mut self.settings.show_all_colliders, "Show all colliders");

        response
    }
}
