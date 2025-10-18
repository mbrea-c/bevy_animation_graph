use std::any::Any;

use bevy_animation_graph::core::{id::BoneId, ragdoll::configuration::RagdollConfig};
use bevy_inspector_egui::reflect_inspector::InspectorUi;
use egui_dock::egui;

use crate::ui::generic_widgets::{bone_id::BoneIdWidget, hashmap::HashMapWidget};

use super::{EguiInspectorExtension, MakeBuffer};

#[derive(Default)]
pub struct RagdollConfigInspector;

impl EguiInspectorExtension for RagdollConfigInspector {
    type Base = RagdollConfig;
    type Buffer = ();

    fn mutable(
        value: &mut Self::Base,
        _buffer: &mut Self::Buffer,
        ui: &mut egui::Ui,
        _options: &dyn Any,
        _id: egui::Id,
        mut env: InspectorUi<'_, '_>,
    ) -> bool {
        let button_response = ui.button("Edit");
        let popup_response = egui::Popup::from_toggle_button_response(&button_response)
            .close_behavior(egui::PopupCloseBehavior::IgnoreClicks)
            .show(|ui| {
                let default_mode_changed = ui
                    .push_id("default mode", |ui| {
                        env.ui_for_reflect(&mut value.default_mode, ui)
                    })
                    .inner;
                let default_readback_changed = ui
                    .push_id("default readback", |ui| {
                        env.ui_for_reflect(&mut value.default_readback, ui)
                    })
                    .inner;
                let readback_overrides_changed =
                    HashMapWidget::new_salted(&mut value.readback_overrides, "readback overrides")
                        .ui(
                            ui,
                            |ui, key| ui.add(BoneIdWidget::new_salted(key, "bone id edit")),
                            |ui, key| ui.label(format!("{}", key.id().hyphenated())),
                            |ui, value| ui.add(egui::Checkbox::without_text(value)),
                        )
                        .changed();

                // let mut response = ui.label("default mode:");
                // response |= egui::ComboBox::from_id_salt("joint variant")
                //     .selected_text(match &self.joint.variant {
                //         JointVariant::Spherical(_) => "Spherical",
                //         JointVariant::Revolute(_) => "Revolute",
                //     })
                //     .show_ui(ui, |ui| {
                //         ui.selectable_value(
                //             &mut self.joint.variant,
                //             JointVariant::Spherical(SphericalJoint::default()),
                //             "Spherical",
                //         );
                //         ui.selectable_value(
                //             &mut self.joint.variant,
                //             JointVariant::Revolute(RevoluteJoint::default()),
                //             "Revolute",
                //         );
                //     })
                //     .response;
                default_mode_changed || default_readback_changed || readback_overrides_changed
            });

        popup_response.map(|r| r.inner).unwrap_or(false)
    }

    fn readonly(
        _value: &Self::Base,
        _buffer: &Self::Buffer,
        _ui: &mut egui::Ui,
        _options: &dyn Any,
        _id: egui::Id,
        _env: InspectorUi<'_, '_>,
    ) {
        todo!()
    }
}

impl MakeBuffer<()> for RagdollConfig {
    fn make_buffer(&self) {
        ()
    }
}
