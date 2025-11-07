use bevy_animation_graph::core::ragdoll::definition::BodyMode;

use crate::ui::generic_widgets::picker::PickerWidget;

pub struct BodyModeWidget<'a> {
    pub body_mode: &'a mut BodyMode,
    pub id_hash: egui::Id,
}

impl<'a> BodyModeWidget<'a> {
    pub fn new_salted(body_mode: &'a mut BodyMode, salt: impl std::hash::Hash) -> Self {
        Self {
            body_mode,
            id_hash: egui::Id::new(salt),
        }
    }
}

impl<'a> egui::Widget for BodyModeWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            let labeler = |b| format!("{:?}", b);
            PickerWidget::new_salted("body mode picker")
                .ui(ui, labeler(*self.body_mode), |ui| {
                    let mut option = |mode| {
                        ui.selectable_value(self.body_mode, mode, labeler(mode));
                    };

                    option(BodyMode::Kinematic);
                    option(BodyMode::Dynamic);
                })
                .response
        })
        .inner
    }
}
