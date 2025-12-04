use bevy_animation_graph::core::edge_data::DataSpec;

use crate::ui::generic_widgets::picker::PickerWidget;

pub struct DataSpecWidget<'a> {
    pub data_spec: &'a mut DataSpec,
    pub id_hash: egui::Id,
}

impl<'a> DataSpecWidget<'a> {
    pub fn new_salted(data_spec: &'a mut DataSpec, salt: impl std::hash::Hash) -> Self {
        Self {
            data_spec,
            id_hash: egui::Id::new(salt),
        }
    }
}

impl<'a> egui::Widget for DataSpecWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            let mut selected = *self.data_spec;
            let mut response = PickerWidget::new_salted("data spec picker")
                .ui(ui, format!("{:?}", selected), |ui| {
                    for val in [
                        DataSpec::F32,
                        DataSpec::Bool,
                        DataSpec::Vec2,
                        DataSpec::Vec3,
                        DataSpec::EntityPath,
                        DataSpec::Quat,
                        DataSpec::BoneMask,
                        DataSpec::Pose,
                        DataSpec::EventQueue,
                        DataSpec::RagdollConfig,
                    ] {
                        ui.selectable_value(&mut selected, val, format!("{:?}", val));
                    }
                })
                .response;

            if selected != *self.data_spec {
                response.mark_changed();
            }

            if response.changed() {
                *self.data_spec = selected;
            }

            response
        })
        .inner
    }
}
