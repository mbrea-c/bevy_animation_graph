use bevy::{ecs::world::World, log::warn_once};
use bevy_animation_graph::core::ragdoll::definition::{
    Joint, JointVariant, Ragdoll, SphericalJoint,
};
use egui::{ComboBox, Widget};

use crate::ui::{
    core::EditorWindowContext,
    generic_widgets::{body_id::BodyIdWidget, vec3::Vec3Widget},
};

pub struct JointInspector<'a, 'b> {
    pub world: &'a mut World,
    pub ctx: &'a mut EditorWindowContext<'b>,
    pub joint: &'a mut Joint,
    pub ragdoll: &'a Ragdoll,
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

            response |= ui.label("use symmetry:");
            response |= ui.add(egui::Checkbox::without_text(&mut self.joint.use_symmetry));
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
                JointVariant::Spherical(spherical_joint) => {
                    response |= ui.label("body 1:");
                    response |= ui.add(
                        BodyIdWidget::new_salted(&mut spherical_joint.body1, "joint body 1")
                            .with_ragdoll(self.ragdoll),
                    );
                    ui.end_row();

                    response |= ui.label("body 2:");
                    response |= ui.add(
                        BodyIdWidget::new_salted(&mut spherical_joint.body2, "joint body 2")
                            .with_ragdoll(self.ragdoll),
                    );
                    ui.end_row();

                    response |= ui.label("position:");
                    response |= ui.add(Vec3Widget::new_salted(
                        &mut spherical_joint.position,
                        "position",
                    ));
                    ui.end_row();

                    response |= ui.label("swing axis:");
                    response |= ui.add(Vec3Widget::new_salted(
                        &mut spherical_joint.swing_axis,
                        "joint swing axis",
                    ));
                    ui.end_row();

                    response |= ui.label("twist axis:");
                    response |= ui.add(Vec3Widget::new_salted(
                        &mut spherical_joint.twist_axis,
                        "joint twist axis",
                    ));
                    ui.end_row();

                    //pub swing_limit: Option<AngleLimit>,
                    //pub twist_limit: Option<AngleLimit>,
                    warn_once!("Reminder to add ui for angle limits!");

                    response |= ui.label("linear damping:");
                    response |= ui.add(egui::DragValue::new(&mut spherical_joint.damping_linear));
                    ui.end_row();

                    response |= ui.label("angular damping:");
                    response |= ui.add(egui::DragValue::new(&mut spherical_joint.damping_angular));
                    ui.end_row();

                    response |= ui.label("position lagrange:");
                    response |=
                        ui.add(egui::DragValue::new(&mut spherical_joint.position_lagrange));
                    ui.end_row();

                    response |= ui.label("swing lagrange:");
                    response |= ui.add(egui::DragValue::new(&mut spherical_joint.swing_lagrange));
                    ui.end_row();

                    response |= ui.label("twist lagrange:");
                    response |= ui.add(egui::DragValue::new(&mut spherical_joint.twist_lagrange));
                    ui.end_row();

                    response |= ui.label("compliance:");
                    response |= ui.add(egui::DragValue::new(&mut spherical_joint.compliance));
                    ui.end_row();

                    response |= ui.label("force:");
                    response |= ui.add(Vec3Widget::new_salted(
                        &mut spherical_joint.force,
                        "joint force",
                    ));
                    ui.end_row();

                    response |= ui.label("swing torque:");
                    response |= ui.add(Vec3Widget::new_salted(
                        &mut spherical_joint.swing_torque,
                        "joint swing torque",
                    ));
                    ui.end_row();

                    response |= ui.label("twist torque:");
                    response |= ui.add(Vec3Widget::new_salted(
                        &mut spherical_joint.twist_torque,
                        "joint twist torque",
                    ));
                    ui.end_row();
                }
            }

            response
        });

        response.inner
    }
}
