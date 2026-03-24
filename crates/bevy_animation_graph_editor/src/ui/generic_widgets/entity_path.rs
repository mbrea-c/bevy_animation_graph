use bevy_animation_graph::core::animation_clip::EntityPath;

pub struct EntityPathWidget<'a> {
    pub entity_path: &'a mut EntityPath,
    pub id_hash: egui::Id,
    pub options: Vec<&'a EntityPath>,
}

impl<'a> EntityPathWidget<'a> {
    pub fn new_salted(entity_path: &'a mut EntityPath, salt: impl std::hash::Hash) -> Self {
        Self {
            entity_path,
            id_hash: egui::Id::new(salt),
            options: Vec::new(),
        }
    }

    pub fn with_options(mut self, options: impl IntoIterator<Item = &'a EntityPath>) -> Self {
        self.options.extend(options);
        self.options.sort();
        self
    }
}

impl<'a> egui::Widget for EntityPathWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            let buffer_id = ui.id().with("entity path slashed string buffer");

            // clear buffer if outdated
            ui.memory_mut(|mem| {
                let prev = mem.data.get_temp::<Buffer>(buffer_id);
                if let Some(prev) = prev
                    && &prev.original != self.entity_path
                {
                    mem.data.remove_temp::<Buffer>(buffer_id);
                }
            });

            let mut buffer = ui.memory_mut(|mem| {
                mem.data
                    .get_temp_mut_or_insert_with(buffer_id, || Buffer {
                        value: self.entity_path.to_slashed_string(),
                        original: self.entity_path.clone(),
                    })
                    .clone()
            });

            let response = ui.add(
                egui::TextEdit::singleline(&mut buffer.value)
                    .desired_width(ui.available_width()),
            );

            let top_k = self
                .options
                .iter()
                .filter(|opt| opt.to_slashed_string().starts_with(&buffer.value))
                .take(10)
                .collect::<Vec<_>>();

            if !top_k.is_empty() && response.has_focus() {
                response.show_tooltip_ui(|ui| {
                    for opt in top_k {
                        let slashed = opt.to_slashed_string();
                        let Some(rest) = slashed.strip_prefix(&buffer.value) else {
                            continue;
                        };

                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                            ui.label(egui::RichText::new(&buffer.value).strong());
                            ui.label(egui::RichText::new(rest));
                        });
                    }
                });
            }

            if response.changed()
                && let Some(new_path) =
                    EntityPath::from_slashed_string_if_safe(buffer.value.clone())
            {
                *self.entity_path = new_path;
            }

            ui.memory_mut(|mem| mem.data.insert_temp(buffer_id, buffer));

            response
        })
        .inner
    }
}

#[derive(Clone, Default)]
struct Buffer {
    value: String,
    original: EntityPath,
}
