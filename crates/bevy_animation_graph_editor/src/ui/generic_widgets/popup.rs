use egui::containers::menu::MenuConfig;

pub struct PopupWidget {
    pub id_hash: egui::Id,
    pub button_label: String,
    pub max_width: Option<f32>,
}

impl PopupWidget {
    pub fn new_salted(salt: impl std::hash::Hash) -> Self {
        Self {
            id_hash: egui::Id::new(salt),
            button_label: "edit".into(),
            max_width: None,
        }
    }

    pub fn with_max_width(mut self, max_width: f32) -> Self {
        self.max_width = Some(max_width);
        self
    }

    pub fn ui(
        self,
        ui: &mut egui::Ui,
        inner: impl FnOnce(&mut egui::Ui) -> egui::Response,
    ) -> egui::Response {
        let max_width = self.max_width;
        let wrapped_inner = move |ui: &mut egui::Ui| -> egui::Response {
            if let Some(w) = max_width {
                ui.set_max_width(w);
            }
            inner(ui)
        };
        ui.push_id(self.id_hash, |ui| {
            ui.horizontal(|ui| {
                let config =
                    MenuConfig::new().close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);
                let (mut button_response, inner) = if egui::containers::menu::is_in_menu(ui) {
                    egui::containers::menu::SubMenuButton::new(&self.button_label)
                        .config(config)
                        .ui(ui, wrapped_inner)
                } else {
                    egui::containers::menu::MenuButton::new(&self.button_label)
                        .config(config)
                        .ui(ui, wrapped_inner)
                };

                if inner.is_some_and(|i| i.inner.changed()) {
                    button_response.mark_changed();
                }

                button_response
            })
            .inner
        })
        .inner
    }
}
