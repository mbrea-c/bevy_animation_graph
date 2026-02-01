use bevy::{
    asset::{Asset, AssetServer, Handle},
    ecs::world::World,
};
use egui::containers::menu::MenuConfig;

use crate::ui::{generic_widgets::asset_picker::AssetPicker, utils::handle_path_server};

pub struct PopupAssetPicker<'a, A: Asset> {
    pub handle: &'a mut Handle<A>,
    pub id_hash: egui::Id,
    pub world: &'a mut World,
}

impl<'a, A: Asset> PopupAssetPicker<'a, A> {
    pub fn new_salted(
        handle: &'a mut Handle<A>,
        world: &'a mut World,
        salt: impl std::hash::Hash,
    ) -> Self {
        Self {
            handle,
            id_hash: egui::Id::new(salt),
            world,
        }
    }
}

impl<'a, A: Asset> egui::Widget for PopupAssetPicker<'a, A> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let path = handle_path_server(
            self.handle.id().untyped(),
            self.world.resource::<AssetServer>(),
        );

        ui.push_id(self.id_hash, |ui| {
            ui.horizontal(|ui| {
                let atoms = "select";
                let add_contents = |ui: &mut egui::Ui| {
                    ui.add(AssetPicker::<A>::new_salted(
                        self.handle,
                        self.world,
                        "inner",
                    ))
                };

                let config =
                    MenuConfig::new().close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);
                let (mut button_response, inner) = if egui::containers::menu::is_in_menu(ui) {
                    egui::containers::menu::SubMenuButton::new(atoms)
                        .config(config)
                        .ui(ui, add_contents)
                } else {
                    egui::containers::menu::MenuButton::new(atoms)
                        .config(config)
                        .ui(ui, add_contents)
                };

                if inner.is_some_and(|i| i.inner.changed()) {
                    button_response.mark_changed();
                }

                let mut response = ui.label(path.to_string_lossy().to_string());
                response |= button_response;

                response
            })
            .inner
        })
        .inner
    }
}
