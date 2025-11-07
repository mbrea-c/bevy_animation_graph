use uuid::Uuid;

pub struct UuidWidget<'a> {
    pub uuid: &'a mut Uuid,
    pub id_hash: egui::Id,
}

impl<'a> UuidWidget<'a> {
    pub fn new_salted(uuid: &'a mut Uuid, salt: impl std::hash::Hash) -> Self {
        Self {
            uuid,
            id_hash: egui::Id::new(salt),
        }
    }
}

#[derive(Clone)]
struct UuidData {
    original: Uuid,
    buffer: String,
}

impl UuidData {
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self {
            original: uuid,
            buffer: format!("{}", uuid.hyphenated()),
        }
    }
}

impl<'a> egui::Widget for UuidWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            let buffer_id = ui.id().with("buffer");
            let mut data = ui.memory_mut(|mem| {
                let prev_data = mem.data.get_temp::<UuidData>(buffer_id);
                if prev_data.is_none_or(|d| d.original != *self.uuid) {
                    mem.data
                        .insert_temp(buffer_id, UuidData::from_uuid(*self.uuid));
                }

                mem.data
                    .get_temp_mut_or_insert_with(buffer_id, || UuidData::from_uuid(*self.uuid))
                    .clone()
            });

            let mut response = ui.add(
                egui::TextEdit::singleline(&mut data.buffer).min_size(egui::Vec2::new(250., 0.)),
            );

            // Mark non-changed; we only consider the response changed if the uuid string is valid
            response.flags &= !egui::response::Flags::CHANGED;

            if let Ok(new_uuid) = Uuid::parse_str(&data.buffer) {
                *self.uuid = new_uuid;
                response.mark_changed();
            }

            ui.memory_mut(|mem| mem.data.insert_temp(buffer_id, data));

            response
        })
        .inner
    }
}
