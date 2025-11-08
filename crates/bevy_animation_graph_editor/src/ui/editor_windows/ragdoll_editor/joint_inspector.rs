use bevy_animation_graph::core::ragdoll::definition::{
    AngleLimit, Joint, JointVariant, Ragdoll, RevoluteJoint, SphericalJoint,
};
use egui::{ComboBox, Widget};

use crate::ui::generic_widgets::{
    angle_limit::AngleLimitWidget, body_id::BodyIdWidget, vec3::Vec3Widget,
};

pub struct JointInspector<'a> {
    pub joint: &'a mut Joint,
    pub ragdoll: &'a Ragdoll,
}

impl Widget for JointInspector<'_> {
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
                    JointVariant::Revolute(_) => "Revolute",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.joint.variant,
                        JointVariant::Spherical(SphericalJoint::default()),
                        "Spherical",
                    );
                    ui.selectable_value(
                        &mut self.joint.variant,
                        JointVariant::Revolute(RevoluteJoint::default()),
                        "Revolute",
                    );
                })
                .response;
            ui.end_row();

            match &mut self.joint.variant {
                JointVariant::Spherical(spherical_joint) => {
                    response |= ui.label("body 1:");
                    response |= ui.add(
                        BodyIdWidget::new_salted(&mut spherical_joint.body1, "joint body 1")
                            .with_ragdoll(Some(self.ragdoll)),
                    );
                    ui.end_row();

                    response |= ui.label("body 2:");
                    response |= ui.add(
                        BodyIdWidget::new_salted(&mut spherical_joint.body2, "joint body 2")
                            .with_ragdoll(Some(self.ragdoll)),
                    );
                    ui.end_row();

                    response |= ui.label("position:");
                    response |= ui.add(Vec3Widget::new_salted(
                        &mut spherical_joint.position,
                        "position",
                    ));
                    ui.end_row();

                    response |= ui.label("twist axis:");
                    response |= ui.add(Vec3Widget::new_salted(
                        &mut spherical_joint.twist_axis,
                        "joint twist axis",
                    ));
                    ui.end_row();

                    let mut swing_enabled = spherical_joint.swing_limit.is_some();
                    response |= ui.label("swing limits enabled:");
                    response |= ui.add(egui::Checkbox::without_text(&mut swing_enabled));
                    ui.end_row();
                    if !swing_enabled {
                        spherical_joint.swing_limit = None;
                    }
                    if swing_enabled && spherical_joint.swing_limit.is_none() {
                        spherical_joint.swing_limit = Some(AngleLimit::default());
                    }

                    if let Some(limit) = &mut spherical_joint.swing_limit {
                        response |= ui.label("swing limits:");
                        response |=
                            ui.add(AngleLimitWidget::new_salted(limit, "swing angle limit"));
                        ui.end_row();
                    }

                    let mut twist_enabled = spherical_joint.twist_limit.is_some();
                    response |= ui.label("twist limits enabled:");
                    response |= ui.add(egui::Checkbox::without_text(&mut twist_enabled));
                    ui.end_row();
                    if !twist_enabled {
                        spherical_joint.twist_limit = None;
                    }
                    if twist_enabled && spherical_joint.twist_limit.is_none() {
                        spherical_joint.twist_limit = Some(AngleLimit::default());
                    }

                    if let Some(limit) = &mut spherical_joint.twist_limit {
                        response |= ui.label("twist limits:");
                        response |=
                            ui.add(AngleLimitWidget::new_salted(limit, "twist angle limit"));
                        ui.end_row();
                    }

                    response |= ui.label("point compliance:");
                    response |= ui.add(egui::DragValue::new(&mut spherical_joint.point_compliance));
                    ui.end_row();

                    response |= ui.label("swing compliance:");
                    response |= ui.add(egui::DragValue::new(&mut spherical_joint.swing_compliance));
                    ui.end_row();

                    response |= ui.label("swing compliance:");
                    response |= ui.add(egui::DragValue::new(&mut spherical_joint.twist_compliance));
                    ui.end_row();
                }
                JointVariant::Revolute(revolute_joint) => {
                    response |= ui.label("body 1:");
                    response |= ui.add(
                        BodyIdWidget::new_salted(&mut revolute_joint.body1, "joint body 1")
                            .with_ragdoll(Some(self.ragdoll)),
                    );
                    ui.end_row();

                    response |= ui.label("body 2:");
                    response |= ui.add(
                        BodyIdWidget::new_salted(&mut revolute_joint.body2, "joint body 2")
                            .with_ragdoll(Some(self.ragdoll)),
                    );
                    ui.end_row();

                    response |= ui.label("position:");
                    response |= ui.add(Vec3Widget::new_salted(
                        &mut revolute_joint.position,
                        "position",
                    ));
                    ui.end_row();

                    response |= ui.label("hinge axis:");
                    response |= ui.add(Vec3Widget::new_salted(
                        &mut revolute_joint.hinge_axis,
                        "joint hinge axis",
                    ));
                    ui.end_row();

                    let mut swing_enabled = revolute_joint.angle_limit.is_some();
                    response |= ui.label("angle limits enabled:");
                    response |= ui.add(egui::Checkbox::without_text(&mut swing_enabled));
                    ui.end_row();
                    if !swing_enabled {
                        revolute_joint.angle_limit = None;
                    }
                    if swing_enabled && revolute_joint.angle_limit.is_none() {
                        revolute_joint.angle_limit = Some(AngleLimit::default());
                    }

                    if let Some(limit) = &mut revolute_joint.angle_limit {
                        response |= ui.label("angle limits:");
                        response |= ui.add(AngleLimitWidget::new_salted(limit, "angle limit"));
                        ui.end_row();
                    }

                    response |= ui.label("point compliance:");
                    response |= ui.add(egui::DragValue::new(&mut revolute_joint.point_compliance));
                    ui.end_row();
                    response |= ui.label("align compliance:");
                    response |= ui.add(egui::DragValue::new(&mut revolute_joint.align_compliance));
                    ui.end_row();
                    response |= ui.label("limit compliance:");
                    response |= ui.add(egui::DragValue::new(&mut revolute_joint.limit_compliance));
                    ui.end_row();
                }
            }

            response
        });

        response.inner
    }
}
