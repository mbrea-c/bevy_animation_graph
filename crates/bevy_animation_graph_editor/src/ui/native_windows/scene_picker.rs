use bevy::{asset::Handle, prelude::World};
use egui_dock::egui;

use crate::ui::{
    generic_widgets::asset_picker::AssetPicker,
    global_state::{
        active_scene::{ActiveScene, SetActiveScene},
        get_global_state,
    },
    native_windows::{EditorWindowContext, NativeEditorWindowExtension},
};

#[derive(Debug)]
pub struct ScenePickerWindow;

impl NativeEditorWindowExtension for ScenePickerWindow {
    fn ui(&self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        let mut active = get_global_state::<ActiveScene>(world)
            .map(|active_scene| active_scene.handle.clone())
            .unwrap_or_default();

        let response = ui.add(AssetPicker::new_salted(
            &mut active,
            world,
            "active scene asset picker",
        ));

        if response.changed() && active != Handle::default() {
            ctx.trigger(SetActiveScene {
                new: ActiveScene { handle: active },
            });
        }
    }

    fn display_name(&self) -> String {
        "Select Scene".to_string()
    }
}
