pub struct PickerWidget {
    pub id_hash: egui::Id,
}

impl PickerWidget {
    pub fn new_salted(salt: impl std::hash::Hash) -> Self {
        Self {
            id_hash: egui::Id::new(salt),
        }
    }
}

impl PickerWidget {
    pub fn ui<R>(
        self,
        ui: &mut egui::Ui,
        selected_text: impl Into<egui::WidgetText>,
        show: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::InnerResponse<Option<R>> {
        ui.push_id(self.id_hash, |ui| {
            ui.menu_button(selected_text, |ui| show(ui))
        })
        .inner
    }
}
