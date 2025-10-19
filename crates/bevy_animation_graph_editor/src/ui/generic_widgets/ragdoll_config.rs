use bevy_animation_graph::core::ragdoll::configuration::RagdollConfig;

use crate::ui::generic_widgets::{bone_id::BoneIdWidget, hashmap::HashMapWidget};

pub struct RagdollConfigWidget<'a> {
    pub config: &'a mut RagdollConfig,
    pub id_hash: egui::Id,
}

impl<'a> RagdollConfigWidget<'a> {
    pub fn new_salted(config: &'a mut RagdollConfig, salt: impl std::hash::Hash) -> Self {
        Self {
            config,
            id_hash: egui::Id::new(salt),
        }
    }
}

impl<'a> egui::Widget for RagdollConfigWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            let mut response = ui.button("Edit");
            let popup_response = egui::Popup::from_toggle_button_response(&response)
                .close_behavior(egui::PopupCloseBehavior::IgnoreClicks)
                .show(|ui| {
                    ui.push_id("default mode", |ui| {
                        // env.ui_for_reflect(&mut value.default_mode, ui)
                    })
                    .inner;
                    ui.push_id("default readback", |ui| {
                        // env.ui_for_reflect(&mut value.default_readback, ui)
                    })
                    .inner;
                    let response = HashMapWidget::new_salted(
                        &mut self.config.readback_overrides,
                        "readback overrides",
                    )
                    .ui(
                        ui,
                        |ui, key| ui.add(BoneIdWidget::new_salted(key, "bone id edit")),
                        |ui, key| ui.label(format!("{}", key.id().hyphenated())),
                        |ui, value| ui.add(egui::Checkbox::without_text(value)),
                    );

                    response
                });

            if let Some(popup_response) = popup_response {
                if popup_response.inner.changed() {
                    response.mark_changed();
                }
            }

            response
        })
        .inner
    }
}
