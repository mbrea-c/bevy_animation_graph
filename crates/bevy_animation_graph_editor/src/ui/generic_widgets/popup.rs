use egui::containers::menu::MenuConfig;

pub struct PopupWidget {
    pub id_hash: egui::Id,
    pub button_label: String,
}

impl PopupWidget {
    pub fn new_salted(salt: impl std::hash::Hash) -> Self {
        Self {
            id_hash: egui::Id::new(salt),
            button_label: "edit".into(),
        }
    }

    pub fn ui(
        self,
        ui: &mut egui::Ui,
        inner: impl FnOnce(&mut egui::Ui) -> egui::Response,
    ) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            ui.horizontal(|ui| {
                let config =
                    MenuConfig::new().close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);
                let (mut button_response, inner) = if egui::containers::menu::is_in_menu(ui) {
                    egui::containers::menu::SubMenuButton::new(&self.button_label)
                        .config(config)
                        .ui(ui, inner)
                } else {
                    egui::containers::menu::MenuButton::new(&self.button_label)
                        .config(config)
                        .ui(ui, inner)
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
