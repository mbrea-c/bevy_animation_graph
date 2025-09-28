use bevy::ecs::world::World;
use bevy_animation_graph::core::ragdoll::definition::{Joint, JointVariant, SphericalJoint};
use egui::{ComboBox, Widget};

use crate::ui::{
    core::EditorWindowContext,
    generic_widgets::{isometry3d::Isometry3dWidget, u32_flags::U32Flags},
    utils::using_inspector_env,
};

pub struct JointInspector<'a, 'b> {
    pub world: &'a mut World,
    pub ctx: &'a mut EditorWindowContext<'b>,
    pub joint: &'a mut Joint,
}

impl Widget for JointInspector<'_, '_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let response = egui::Grid::new("ragdoll body inspector").show(ui, |ui| {
            let mut response = ui.label("ID:");
            response |= ui.label(format!("{:?}", self.joint.id));
            ui.end_row();

            response |= ui.label("label:");
            response |= ui.text_edit_singleline(&mut self.joint.label);
            ui.end_row();

            ui.label("variant:");
            response |= ComboBox::from_id_salt("joint variant")
                .selected_text(match &self.joint.variant {
                    JointVariant::Spherical(_) => "Spherical",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.joint.variant,
                        JointVariant::Spherical(SphericalJoint::default()),
                        "Spherical",
                    );
                })
                .response;
            ui.end_row();

            match &mut self.joint.variant {
                JointVariant::Spherical(spherical_joint) => {}
            }

            response
        });

        response.inner
    }
}
