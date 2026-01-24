pub struct StringPickerWidget<'a> {
    pub selected: &'a mut String,
    pub options: &'a Vec<String>,
    pub id_hash: egui::Id,
}

impl<'a> StringPickerWidget<'a> {
    pub fn new(selected: &'a mut String, options: &'a Vec<String>) -> Self {
        Self {
            selected,
            options,
            id_hash: egui::Id::new("string picker"),
        }
    }

    pub fn salted(mut self, salt: impl std::hash::Hash) -> Self {
        self.id_hash = self.id_hash.with(salt);
        self
    }
}

impl<'a> egui::Widget for StringPickerWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            let old_selected = self.selected.clone();

            let mut response = egui::ComboBox::from_id_salt("string picker combobox")
                .selected_text(&*self.selected)
                .show_ui(ui, |ui| {
                    for val in self.options {
                        ui.selectable_value(self.selected, val.to_string(), val);
                    }
                })
                .response;

            let changed = self.selected != &old_selected;

            if changed {
                response.mark_changed();
            }

            response
        })
        .inner
    }
}
