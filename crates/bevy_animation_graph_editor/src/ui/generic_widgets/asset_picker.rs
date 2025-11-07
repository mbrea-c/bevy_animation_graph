use bevy::{
    asset::{Asset, AssetId, AssetServer, Assets, Handle},
    ecs::world::World,
};

use crate::ui::{
    generic_widgets::tree::{Tree, TreeWidget, paths::PathTreeRenderer},
    utils::{asset_sort_key, handle_path},
};

pub struct AssetPicker<'a, A: Asset> {
    pub handle: &'a mut Handle<A>,
    pub id_hash: egui::Id,
    pub world: &'a mut World,
}

impl<'a, A: Asset> AssetPicker<'a, A> {
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

impl<'a, A: Asset> egui::Widget for AssetPicker<'a, A> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            self.world
                .resource_scope::<Assets<A>, _>(|world, mut assets| {
                    let asset_server = world.resource::<AssetServer>();
                    let mut asset_ids: Vec<_> = assets.ids().collect();
                    asset_ids.sort_by_key(|id| asset_sort_key(*id, asset_server));
                    let paths = asset_ids
                        .into_iter()
                        .map(|id| (handle_path(id.untyped(), asset_server), id))
                        .collect();
                    let path_tree = Tree::from_paths(paths);
                    TreeWidget::new_salted(
                        path_tree,
                        PathTreeRenderer {
                            is_selected: {
                                let selected_id = self.handle.id();
                                Box::new(move |_, v: &AssetId<A>| *v == selected_id)
                            },
                        },
                        "asset picker tree",
                    )
                    .show(ui, |result| {
                        if let Some(id) = result.clicked
                            && let Some(handle) = assets.get_strong_handle(id)
                        {
                            *self.handle = handle;
                        }
                    })
                })
        })
        .inner
    }
}
