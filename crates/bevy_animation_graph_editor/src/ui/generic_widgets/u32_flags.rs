use bevy::utils::default;

pub struct U32Flags<'a> {
    pub flags: &'a mut u32,
    pub id_hash: egui::Id,
}

impl<'a> U32Flags<'a> {
    pub fn new_salted(flags: &'a mut u32, salt: impl std::hash::Hash) -> Self {
        Self {
            flags,
            id_hash: egui::Id::new(salt),
        }
    }
}

impl<'a> egui::Widget for U32Flags<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        // ui.horizontal(|ui| {
        //     for i in 0..32 {
        //         let id = ui.id().with(self.id_hash).with(format!("flag bit {i}"));
        //         let mut checked = (*self.flags | (1 << i)) != 0;
        //         let bit_response = ui
        //             .push_id(id, |ui| ui.add(egui::Checkbox::without_text(&mut checked)))
        //             .inner;
        //         cumulative_response = Some(if let Some(r) = cumulative_response {
        //             r | bit_response
        //         } else {
        //             bit_response
        //         });
        //     }
        //     cumulative_response.expect("Number of bits is hardcoded")
        // })
        // .inner
        ui.push_id(self.id_hash, |ui| {
            let side = 10.;

            let mut total_response =
                ui.allocate_response(egui::Vec2::new(16. * side, 4. * side), egui::Sense::click());

            let n_row = 16;

            for i in 0..32 {
                let row = i / n_row;
                let col = i % n_row;
                let bit_rect = egui::Rect::from_min_max(
                    total_response.rect.min
                        + egui::Vec2::new(side * col as f32, ((row * 2) + 1) as f32 * side),
                    total_response.rect.min
                        + egui::Vec2::new(side * (col + 1) as f32, ((row * 2) + 2) as f32 * side),
                );

                let is_mouse_over_bit = total_response
                    .hover_pos()
                    .is_some_and(|p| bit_rect.contains(p));

                let mask = 1 << i;
                let bit_on = (*self.flags & mask) != 0;

                let fill_color = match (is_mouse_over_bit, bit_on) {
                    (_, true) => egui::Color32::from_rgb(30, 30, 230),
                    (true, false) => egui::Color32::from_rgb(120, 120, 255),
                    (false, false) => egui::Color32::GRAY,
                };
                ui.painter().rect_filled(bit_rect, 0., fill_color);
                ui.painter().text(
                    bit_rect.center_top(),
                    egui::Align2::CENTER_BOTTOM,
                    format!("{i}"),
                    egui::FontId {
                        size: 7.5,
                        family: default(),
                    },
                    ui.visuals().text_color(),
                );

                if total_response.clicked() && is_mouse_over_bit {
                    *self.flags ^= mask;
                    total_response.mark_changed()
                }
            }

            total_response
        })
        .inner
    }
}
