pub struct Collapser {
    pub default_open: bool,
    pub id_salt: egui::Id,
}

pub struct CollapserResponse<H, B> {
    pub head: H,
    pub body: Option<B>,
}

impl Collapser {
    pub fn new() -> Self {
        Self {
            default_open: false,
            id_salt: egui::Id::new(0),
        }
    }

    pub fn with_default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    pub fn with_id_salt(mut self, salt: impl std::hash::Hash) -> Self {
        self.id_salt = egui::Id::new(salt);
        self
    }

    pub fn show<H, B>(
        self,
        ui: &mut egui::Ui,
        show_header: impl FnOnce(&mut egui::Ui) -> H,
        show_body: impl FnOnce(&mut egui::Ui) -> B,
    ) -> CollapserResponse<H, B> {
        let id = ui.id().with("Collapser").with(self.id_salt);
        let mut is_open = ui.memory_mut(|mem| *mem.data.get_temp_mut_or(id, self.default_open));

        let response = ui
            .vertical(|ui| {
                let header_response = ui
                    .horizontal(|ui| {
                        ui.toggle_value(&mut is_open, "v");
                        show_header(ui)
                    })
                    .inner;
                let body_response = if is_open {
                    let response = ui
                        .horizontal(|ui| {
                            ui.add_space(4.);
                            ui.separator();
                            ui.add_space(4.);
                            ui.vertical(|ui| show_body(ui)).inner
                        })
                        .inner;
                    Some(response)
                } else {
                    None
                };
                CollapserResponse {
                    head: header_response,
                    body: body_response,
                }
            })
            .inner;
        ui.memory_mut(|mem| mem.data.insert_temp(id, is_open));
        response
    }
}
