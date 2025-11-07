use bevy::{asset::Handle, prelude::World};
use egui_dock::egui;

use crate::ui::{
    generic_widgets::asset_picker::AssetPicker,
    global_state::{
        active_fsm::{ActiveFsm, SetActiveFsm},
        get_global_state,
        inspector_selection::{InspectorSelection, SetInspectorSelection},
    },
    native_windows::{EditorWindowContext, NativeEditorWindowExtension},
};

#[derive(Debug)]
pub struct FsmPickerWindow;

impl NativeEditorWindowExtension for FsmPickerWindow {
    fn ui(&self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        let mut active = get_global_state::<ActiveFsm>(world)
            .map(|active_scene| active_scene.handle.clone())
            .unwrap_or_default();

        let response = ui.add(AssetPicker::new_salted(
            &mut active,
            world,
            "active graph asset picker",
        ));

        if response.changed() && active != Handle::default() {
            ctx.trigger(SetActiveFsm {
                new: ActiveFsm { handle: active },
            });
            ctx.trigger(SetInspectorSelection {
                selection: InspectorSelection::ActiveFsm,
            })
        }
    }

    fn display_name(&self) -> String {
        "Select FSM".to_string()
    }
}
