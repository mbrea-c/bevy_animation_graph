use bevy::{asset::Handle, prelude::World};
use egui_dock::egui;

use crate::ui::{
    generic_widgets::asset_picker::AssetPicker,
    native_windows::{EditorWindowContext, NativeEditorWindowExtension},
    state_management::global::{
        active_graph::{ActiveGraph, SetActiveGraph},
        get_global_state,
        inspector_selection::{InspectorSelection, SetInspectorSelection},
    },
};

#[derive(Debug)]
pub struct GraphPickerWindow;

impl NativeEditorWindowExtension for GraphPickerWindow {
    fn ui(&self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        let mut active = get_global_state::<ActiveGraph>(world)
            .map(|active_scene| active_scene.handle.clone())
            .unwrap_or_default();

        let response = ui.add(AssetPicker::new_salted(
            &mut active,
            world,
            "active graph asset picker",
        ));

        if response.changed() && active != Handle::default() {
            ctx.trigger(SetActiveGraph {
                new: ActiveGraph { handle: active },
            });
            ctx.trigger(SetInspectorSelection {
                selection: InspectorSelection::ActiveGraph,
            })
        }
    }

    fn display_name(&self) -> String {
        "Select Graph".to_string()
    }
}
