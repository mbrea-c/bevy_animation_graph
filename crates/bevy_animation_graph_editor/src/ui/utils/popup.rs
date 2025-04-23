use std::hash::Hash;

pub struct CustomPopup<S> {
    pub save_on_click: Option<S>,
    pub allow_opening: bool,
    pub sense_rect: egui::Rect,
    pub default_size: egui::Vec2,
    pub salt: egui::Id,
}

impl<S> CustomPopup<S>
where
    S: Default + Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            save_on_click: None,
            allow_opening: true,
            sense_rect: egui::Rect::ZERO,
            salt: egui::Id::new(0),
            default_size: egui::Vec2::ZERO,
        }
    }

    pub fn with_salt(mut self, salt: impl Hash) -> Self {
        self.salt = self.salt.with(salt);
        self
    }

    pub fn with_save_on_click(mut self, save_on_click: Option<S>) -> Self {
        self.save_on_click = save_on_click;
        self
    }

    pub fn with_sense_rect(mut self, allowable_rect: egui::Rect) -> Self {
        self.sense_rect = allowable_rect;
        self
    }

    pub fn with_default_size(mut self, default_size: egui::Vec2) -> Self {
        self.default_size = default_size;
        self
    }

    pub fn with_allow_opening(mut self, allow_opening: bool) -> Self {
        self.allow_opening = allow_opening;
        self
    }

    pub fn show_if_saved<T>(
        self,
        ui: &mut egui::Ui,
        ui_builder: impl FnOnce(&mut egui::Ui, S) -> T,
    ) -> Option<T> {
        let popup_id = ui.id().with(self.salt);

        if self.allow_opening
            && ui.input(|i| i.pointer.secondary_clicked())
            && ui
                .input(|i| i.pointer.interact_pos())
                .is_some_and(|p| self.sense_rect.contains(p))
        {
            let pointer_pos = ui.input(|i| i.pointer.interact_pos()).unwrap_or_default();

            Self::mut_popup_state(ui, popup_id, |s| {
                s.open = true;
            });

            if let Some(save_on_click) = self.save_on_click {
                ui.memory_mut(|mem| {
                    mem.data
                        .insert_persisted(popup_id, (pointer_pos, save_on_click))
                })
            }
        }

        let (pointer_pos, saved_on_click) = ui
            .memory_mut(|mem| mem.data.get_persisted(popup_id))
            .unwrap_or_default();

        // We need to do some response hacking, since it's otherwise nontrivial
        // to draw a popup on right click position.
        //
        // The alternative is to implement our own popup widget, which I'm reluctant
        // to do now given that egui 0.32 is overhauling popups.
        let new_rect = egui::Rect {
            min: pointer_pos,
            max: pointer_pos + egui::Vec2::new(80., 0.),
        };

        Self::custom_popup(
            ui,
            popup_id,
            egui::AboveOrBelow::Below,
            self.default_size,
            new_rect,
            |ui| ui_builder(ui, saved_on_click),
        )
    }

    fn is_open(ui: &mut egui::Ui, id: egui::Id) -> bool {
        ui.memory_mut(|mem| mem.data.get_persisted::<PopupState>(id))
            .is_some_and(|p| p.open)
    }

    fn mut_popup_state(ui: &mut egui::Ui, id: egui::Id, mutate: impl FnOnce(&mut PopupState)) {
        let mut old_state = ui
            .memory_mut(|mem| mem.data.get_persisted::<PopupState>(id))
            .unwrap_or_default();
        mutate(&mut old_state);
        ui.memory_mut(|mem| mem.data.insert_persisted(id, old_state));
    }

    fn custom_popup<R>(
        parent_ui: &mut egui::Ui,
        popup_id: egui::Id,
        above_or_below: egui::AboveOrBelow,
        default_size: egui::Vec2,
        rect: egui::Rect,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> Option<R> {
        if !Self::is_open(parent_ui, popup_id) {
            return None;
        }

        let (mut pos, pivot) = match above_or_below {
            egui::AboveOrBelow::Above => (rect.left_top(), egui::Align2::LEFT_BOTTOM),
            egui::AboveOrBelow::Below => (rect.left_bottom(), egui::Align2::LEFT_TOP),
        };

        if let Some(to_global) = parent_ui
            .ctx()
            .layer_transform_to_global(parent_ui.layer_id())
        {
            pos = to_global * pos;
        }

        let frame = egui::Frame::popup(parent_ui.style());
        let frame_margin = frame.total_margin();
        let inner_width = (rect.width() - frame_margin.sum().x).max(0.0);

        let response = egui::Area::new(popup_id)
            .kind(egui::UiKind::Popup)
            .order(egui::Order::Foreground)
            .fixed_pos(pos)
            .default_size(default_size)
            .pivot(pivot)
            .show(parent_ui.ctx(), |ui| {
                frame
                    .show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                            ui.set_min_width(inner_width);
                            add_contents(ui)
                        })
                        .inner
                    })
                    .inner
            });

        if parent_ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            Self::mut_popup_state(parent_ui, popup_id, |s| {
                s.open = false;
            });
        }
        Some(response.inner)
    }
}

#[derive(Debug, Clone, Default)]
struct PopupState {
    open: bool,
}
