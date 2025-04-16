use std::{
    any::{Any, TypeId},
    marker::PhantomData,
};

use bevy::asset::{Asset, AssetServer, Assets, Handle};
use bevy_inspector_egui::reflect_inspector::InspectorUi;
use egui_dock::egui;

use super::{EguiInspectorExtension, MakeBuffer};

pub struct AssetPickerInspector<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for AssetPickerInspector<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: Asset> EguiInspectorExtension for AssetPickerInspector<T> {
    type Base = Handle<T>;
    type Buffer = ();

    fn mutable(
        value: &mut Self::Base,
        _: &mut Self::Buffer,
        ui: &mut egui::Ui,
        _options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) -> bool {
        let value_id = value.id();

        let Some(world) = env.context.world.as_mut() else {
            return false;
        };
        let (mut world_t_assets, world) = world.split_off_resource(TypeId::of::<Assets<T>>());
        let Ok(t_assets) = world_t_assets.get_resource_mut::<Assets<T>>() else {
            return false;
        };
        let Some((asset_server, _)) = world.split_off_resource_typed::<AssetServer>() else {
            return false;
        };

        let mut selected = value_id;
        egui::ComboBox::from_id_salt(id)
            .selected_text(if t_assets.contains(selected) {
                asset_server
                    .get_path(selected)
                    .map(|p| p.path().to_string_lossy().into())
                    .unwrap_or("Unsaved Asset".to_string())
            } else {
                "None".to_string()
            })
            .show_ui(ui, |ui| {
                let mut assets_ids: Vec<_> = t_assets.ids().collect();
                assets_ids.sort();
                for asset_id in assets_ids {
                    ui.selectable_value(
                        &mut selected,
                        asset_id,
                        asset_server
                            .get_path(asset_id)
                            .unwrap_or_default()
                            .path()
                            .to_str()
                            .unwrap_or_default(),
                    );
                }
            });
        if selected != value_id {
            *value = asset_server.get_id_handle(selected).unwrap();
            true
        } else {
            false
        }
    }

    fn readonly(
        _value: &Self::Base,
        buffer: &Self::Buffer,
        ui: &mut egui::Ui,
        _options: &dyn Any,
        id: egui::Id,
        mut env: InspectorUi<'_, '_>,
    ) {
        env.ui_for_reflect_readonly_with_options(buffer, ui, id, &());
    }
}

impl<T: Asset> MakeBuffer<()> for Handle<T> {
    fn make_buffer(&self) {}
}
