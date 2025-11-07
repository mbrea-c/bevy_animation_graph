pub struct ListWidget<'a, I> {
    pub list: &'a mut Vec<I>,
    pub id_hash: egui::Id,
}

impl<'a, I> ListWidget<'a, I> {
    pub fn new_salted(list: &'a mut Vec<I>, salt: impl std::hash::Hash) -> Self {
        Self {
            list,
            id_hash: egui::Id::new(salt),
        }
    }
}

impl<'a, I> ListWidget<'a, I> {
    pub fn ui(
        self,
        ui: &mut egui::Ui,
        mut show_item: impl FnMut(&mut egui::Ui, &mut I) -> egui::Response,
    ) -> egui::Response
    where
        I: Default,
    {
        ui.push_id(self.id_hash, |ui| {
            let mut response = ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover());

            let vertical_response = ui.vertical(|ui| {
                let mut move_up = None;
                let mut move_down = None;
                let mut delete = None;
                egui::Grid::new(self.id_hash.with("list grid")).show(ui, |ui| {
                    for i in 0..self.list.len() {
                        ui.push_id(i, |ui| {
                            ui.horizontal(|ui| {
                                if i > 0 && ui.button("⬆").clicked() {
                                    move_up = Some(i);
                                }

                                if i < self.list.len() - 1 && ui.button("⬇").clicked() {
                                    move_down = Some(i);
                                }

                                if ui.button("🗙").clicked() {
                                    delete = Some(i);
                                }
                            });
                        });
                        ui.push_id(i, |ui| {
                            let item = &mut self.list[i];
                            response |= show_item(ui, item);
                        });
                        ui.end_row();
                    }

                    ui.horizontal(|ui| {
                        if ui.button("+").clicked() {
                            self.list.push(I::default());
                        }
                    });
                    ui.label("Add item");
                    ui.end_row();
                });

                if let Some(idx) = move_up {
                    self.list.swap(idx, idx - 1);
                }
                if let Some(idx) = move_down {
                    self.list.swap(idx, idx + 1);
                }
                if let Some(idx) = delete {
                    self.list.remove(idx);
                }
            });

            response |= vertical_response.response;

            response
        })
        .inner
    }
}
