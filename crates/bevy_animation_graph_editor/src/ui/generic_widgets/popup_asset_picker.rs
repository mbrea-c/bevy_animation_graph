use bevy::{
    asset::{Asset, AssetServer, Handle},
    ecs::world::World,
};

use crate::ui::{
    generic_widgets::{asset_picker::AssetPicker, popup::PopupWidget},
    utils::handle_path_server,
};

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
                let mut response =
                    PopupWidget::new_salted("asset picker popup").ui(ui, |ui: &mut egui::Ui| {
                        ui.add(AssetPicker::<A>::new_salted(
                            self.handle,
                            self.world,
                            "inner",
                        ))
                    });

                response |= ui.label(path.to_string_lossy().to_string());

                response
            })
            .inner
        })
        .inner
    }
}
