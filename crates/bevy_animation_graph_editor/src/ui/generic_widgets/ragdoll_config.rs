use bevy_animation_graph::core::{
    ragdoll::{configuration::RagdollConfig, definition::Ragdoll},
    skeleton::Skeleton,
};
use egui::Checkbox;

use crate::ui::generic_widgets::{
    body_id::{BodyIdReadonlyWidget, BodyIdWidget},
    body_mode::BodyModeWidget,
    bone_id::{BoneIdReadonlyWidget, BoneIdWidget},
    hashmap::HashMapWidget,
    option::CheapOptionWidget,
};

pub struct RagdollConfigWidget<'a> {
    pub config: &'a mut RagdollConfig,
    pub id_hash: egui::Id,
    pub ragdoll: Option<&'a Ragdoll>,
    pub skeleton: Option<&'a Skeleton>,
}

impl<'a> RagdollConfigWidget<'a> {
    pub fn new_salted(config: &'a mut RagdollConfig, salt: impl std::hash::Hash) -> Self {
        Self {
            config,
            id_hash: egui::Id::new(salt),
            ragdoll: None,
            skeleton: None,
        }
    }

    pub fn with_ragdoll(mut self, ragdoll: Option<&'a Ragdoll>) -> Self {
        self.ragdoll = ragdoll;
        self
    }

    pub fn with_skeleton(mut self, skeleton: Option<&'a Skeleton>) -> Self {
        self.skeleton = skeleton;
        self
    }
}

impl<'a> egui::Widget for RagdollConfigWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            let mut response = ui.button("Edit");
            let popup_response = egui::Popup::from_toggle_button_response(&response)
                .close_behavior(egui::PopupCloseBehavior::IgnoreClicks)
                .show(|ui| {
                    let mut response = ui.heading("Defaults");
                    ui.horizontal(|ui| {
                        response |= ui.label("default mode:");
                        response |= CheapOptionWidget::new_salted(
                            &mut self.config.default_mode,
                            "default mode editor",
                        )
                        .ui(ui, |ui, val| ui.add(BodyModeWidget::new_salted(val, "")));
                    });

                    ui.horizontal(|ui| {
                        response |= ui.label("default readback:");
                        response |= CheapOptionWidget::new_salted(
                            &mut self.config.default_readback,
                            "default mode editor",
                        )
                        .ui(ui, |ui, val| ui.add(Checkbox::without_text(val)));
                    });

                    response |= ui.heading("Readback overrides");
                    response |= HashMapWidget::new_salted(
                        &mut self.config.readback_overrides,
                        "readback overrides",
                    )
                    .ui(
                        ui,
                        |ui, key| {
                            ui.add(
                                BoneIdWidget::new_salted(key, "bone id edit")
                                    .with_skeleton(self.skeleton),
                            )
                        },
                        |ui, key| {
                            ui.add(
                                BoneIdReadonlyWidget::new_salted(key, "bone id readonly")
                                    .with_skeleton(self.skeleton),
                            )
                        },
                        |ui, value| ui.add(egui::Checkbox::without_text(value)),
                    );

                    response |= ui.heading("Body mode overrides");
                    response |= HashMapWidget::new_salted(
                        &mut self.config.mode_overrides,
                        "mode overrides",
                    )
                    .ui(
                        ui,
                        |ui, key| {
                            ui.add(
                                BodyIdWidget::new_salted(key, "body id edit")
                                    .with_ragdoll(self.ragdoll),
                            )
                        },
                        |ui, key| {
                            ui.add(
                                BodyIdReadonlyWidget::new_salted(key, "body id readonly")
                                    .with_ragdoll(self.ragdoll),
                            )
                        },
                        |ui, value| ui.add(BodyModeWidget::new_salted(value, "body mode picker")),
                    );

                    response
                });

            if let Some(popup_response) = popup_response
                && popup_response.inner.changed() {
                    response.mark_changed();
                }

            response
        })
        .inner
    }
}
