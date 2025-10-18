use bevy_animation_graph::core::ragdoll::configuration::RagdollConfig;

pub struct RagdollConfigWidget<'a> {
    pub config: &'a mut RagdollConfig,
    pub id_hash: egui::Id,
    pub width: f32,
}

impl<'a> RagdollConfigWidget<'a> {
    pub fn new_salted(config: &'a mut RagdollConfig, salt: impl std::hash::Hash) -> Self {
        Self {
            config,
            id_hash: egui::Id::new(salt),
            width: 300.,
        }
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

impl<'a> egui::Widget for RagdollConfigWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let button_response = ui.button("Edit");
        let popup_response =
            egui::Popup::from_toggle_button_response(&button_response).show(|ui| {
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
            });

        button_response
    }
}
