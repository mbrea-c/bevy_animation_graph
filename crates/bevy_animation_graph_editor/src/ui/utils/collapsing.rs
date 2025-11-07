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
                        let label = if is_open { "⏷" } else { "⏵" };
                        if ui.add(egui::Button::new(label).frame(false)).clicked() {
                            is_open = !is_open;
                        }
                        show_header(ui)
                    })
                    .inner;
                let body_response = if is_open {
                    let response = ui.horizontal(|ui| {
                        ui.add_space(12.);
                        ui.vertical(|ui| show_body(ui)).inner
                    });
                    let rect = response.response.rect;
                    let offset = egui::Vec2::new(5., 0.);
                    ui.painter().line_segment(
                        [rect.left_top() + offset, rect.left_bottom() + offset],
                        ui.visuals().widgets.noninteractive.bg_stroke,
                    );
                    Some(response.inner)
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
