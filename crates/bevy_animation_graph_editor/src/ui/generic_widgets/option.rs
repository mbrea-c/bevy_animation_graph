pub struct CheapOptionWidget<'a, T> {
    pub value: &'a mut Option<T>,
    pub id_hash: egui::Id,
}

impl<'a, T> CheapOptionWidget<'a, T> {
    pub fn new_salted(value: &'a mut Option<T>, salt: impl std::hash::Hash) -> Self {
        Self {
            value,
            id_hash: egui::Id::new(salt),
        }
    }
}

impl<'a, T> CheapOptionWidget<'a, T>
where
    T: Clone + Default + Send + Sync + 'static,
{
    pub fn ui(
        self,
        ui: &mut egui::Ui,
        show: impl FnOnce(&mut egui::Ui, &mut T) -> egui::Response,
    ) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            let buffer_id = ui.id().with("cheap option widget buffer");
            let mut check = self.value.is_some();
            let mut value = self.value.clone().unwrap_or_else(|| {
                ui.memory_mut(|mem| mem.data.get_temp::<T>(buffer_id))
                    .unwrap_or_default()
            });

            let response = ui
                .horizontal(|ui| {
                    let mut response = ui.add(egui::Checkbox::without_text(&mut check));

                    response |= ui.add_enabled_ui(check, |ui| show(ui, &mut value)).inner;

                    response
                })
                .inner;

            ui.memory_mut(|mem| mem.data.insert_temp(buffer_id, value.clone()));

            if check {
                *self.value = Some(value);
            } else {
                *self.value = None;
            }

            response
        })
        .inner
    }
}
