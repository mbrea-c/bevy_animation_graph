use std::{any::Any, collections::VecDeque};

use bevy::platform::collections::HashSet;

use crate::ui::utils::{
    IndexTransform,
    ui_buffer::{CloneBuffer, ErasedCloneBuffer, SelfContainedBuffer},
};

pub struct ListLikeWidget<'a, L> {
    pub value: &'a mut L,
    pub id_hash: egui::Id,
}

impl<'a, L> ListLikeWidget<'a, L> {
    pub fn new(list_like: &'a mut L) -> Self {
        Self {
            value: list_like,
            id_hash: egui::Id::new("list like widget"),
        }
    }

    #[allow(dead_code)]
    pub fn salted(mut self, salt: impl std::hash::Hash) -> Self {
        self.id_hash = self.id_hash.with(salt);
        self
    }
}

impl<'a, L> ListLikeWidget<'a, L>
where
    L: ListLike + 'a,
{
    pub fn show(mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            ui.vertical(|ui| {
                let response = egui::Frame::new()
                    .outer_margin(3.)
                    .inner_margin(3.)
                    .corner_radius(3.)
                    .stroke((1., ui.style().visuals.weak_text_color()))
                    .show(ui, |ui| {
                        let mut response = ui.heading("Inputs");

                        let mut list_buffer = ListBuffer::from_ui(ui, &(), self.value);
                        let mut pending_events = VecDeque::new();

                        for index in 0..self.value.len() {
                            response |=
                                self.show_field(ui, index, &mut list_buffer, &mut pending_events);
                        }

                        ui.horizontal(|ui| {
                            if ui.button("+").clicked() {
                                pending_events.push_back(ListWidgetEvent::AppendDefault);
                                response.mark_changed();
                            }
                            ui.label("Add item");
                        });

                        while let Some(event) = pending_events.pop_front() {
                            let mut itf = IndexTransform::Noop;
                            match event {
                                ListWidgetEvent::Shift { index, delta } => {
                                    let new_index = self.value.shift_index(index, delta);
                                    itf = IndexTransform::Shift { index, new_index };
                                }
                                ListWidgetEvent::Removal { index } => {
                                    self.value.remove(index);
                                    itf = IndexTransform::Removal { index };
                                }
                                ListWidgetEvent::AppendDefault => {
                                    if let Some(default) = self.value.default() {
                                        let buffer = L::ItemBuffer::new(ui, &(), default.as_ref());
                                        self.value.push(&buffer);
                                    }
                                }
                            }
                            pending_events.retain_mut(|ev| {
                                if let Some(new) = ev.apply_transform(&itf) {
                                    *ev = new;
                                    true
                                } else {
                                    false
                                }
                            });

                            list_buffer.wants_update = list_buffer
                                .wants_update
                                .into_iter()
                                .filter_map(|i| itf.adjusted(i))
                                .collect();

                            itf.apply_vec(&mut list_buffer.item_buffers);
                        }

                        CloneBuffer::<L, ()>::save_back(&list_buffer, ui);

                        response
                    })
                    .inner;

                response
            })
            .inner
        })
        .inner
    }

    fn show_field(
        &mut self,
        ui: &mut egui::Ui,
        index: usize,
        list_buffer: &mut ListBuffer,
        pending_events: &mut VecDeque<ListWidgetEvent>,
    ) -> egui::Response {
        ui.push_id(index, |ui| {
            let ListBuffer {
                item_buffers,
                wants_update,
            } = list_buffer;

            let b: &mut dyn Any = item_buffers.get_mut(index).unwrap().as_mut();
            let buffer = b.downcast_mut::<L::ItemBuffer>().unwrap();

            ui.horizontal(|ui| {
                let controls_response =
                    self.item_controls(ui, index, self.value.len(), pending_events);

                let item_response = self.value.edit_item(ui, buffer);

                if item_response.changed() {
                    wants_update.insert(index);
                }

                if wants_update.contains(&index) {
                    if self.value.update(index, buffer) {
                        wants_update.remove(&index);
                        *buffer = L::ItemBuffer::new(ui, &(), &buffer.value());
                    }
                }

                controls_response | item_response
            })
            .inner
        })
        .inner
    }

    fn item_controls(
        &mut self,
        ui: &mut egui::Ui,
        index: usize,
        size: usize,
        pending_events: &mut VecDeque<ListWidgetEvent>,
    ) -> egui::Response {
        ui.scope(|ui| {
            ui.set_min_width(60.);
            let mut move_up = None;
            let mut move_down = None;
            let mut delete = None;

            let button =
                |ui: &mut egui::Ui, text: &str| ui.add(egui::Button::new(text).frame(false));

            let mut response = button(ui, "🗙");
            if response.clicked() {
                delete = Some(index);
            }

            let up_response = ui.add_enabled_ui(index > 0, |ui| button(ui, "⬆")).inner;
            if index > 0 && up_response.clicked() {
                move_up = Some(index);
            }
            response |= up_response;

            let down_response = ui
                .add_enabled_ui(index < (size - 1), |ui| button(ui, "⬇"))
                .inner;
            if index < size - 1 && down_response.clicked() {
                move_down = Some(index);
            }
            response |= down_response;

            if move_up.is_some() {
                response.mark_changed();
                pending_events.push_back(ListWidgetEvent::Shift { index, delta: -1 });
            }

            if move_down.is_some() {
                response.mark_changed();
                pending_events.push_back(ListWidgetEvent::Shift { index, delta: 1 });
            }

            if delete.is_some() {
                response.mark_changed();
                pending_events.push_back(ListWidgetEvent::Removal { index });
            }

            response
        })
        .inner
    }
}

pub trait ListLike {
    type Item: ?Sized;
    type ItemBuffer: SelfContainedBuffer<Self::Item, ()>;

    fn iter(&self) -> impl Iterator<Item = &Self::Item>;
    fn len(&self) -> usize;
    /// Returns new index
    fn shift_index(&mut self, index: usize, delta: i32) -> usize;
    fn remove(&mut self, index: usize);
    /// If this returns false, the update failed
    fn update(&mut self, index: usize, buffer: &Self::ItemBuffer) -> bool;
    fn push(&mut self, buffer: &Self::ItemBuffer);

    fn default(&self) -> Option<Box<Self::Item>>;

    fn edit_item(&self, ui: &mut egui::Ui, buffer: &mut Self::ItemBuffer) -> egui::Response;
}

struct ListBuffer {
    item_buffers: Vec<Box<dyn ErasedCloneBuffer>>,
    wants_update: HashSet<usize>,
}

impl Clone for ListBuffer {
    fn clone(&self) -> Self {
        Self {
            item_buffers: self
                .item_buffers
                .iter()
                .map(|v| v.as_ref().clone_box())
                .collect(),
            wants_update: self.wants_update.clone(),
        }
    }
}

impl<L> CloneBuffer<L, ()> for ListBuffer
where
    L: ListLike,
{
    fn new(ui: &egui::Ui, (): &(), value: &L) -> Self {
        Self {
            item_buffers: value
                .iter()
                .map(|v| {
                    let b: Box<dyn ErasedCloneBuffer> = Box::new(L::ItemBuffer::new(ui, &(), v));
                    b
                })
                .collect(),
            wants_update: HashSet::new(),
        }
    }

    fn id(&self, ui: &egui::Ui) -> egui::Id {
        ui.id().with("list like buffer")
    }

    fn is_still_valid(&self, (): &(), value: &L) -> bool {
        self.item_buffers.len() == value.len()
            && self.item_buffers.iter().zip(value.iter()).all(|(b, v)| {
                let b: &dyn Any = b.as_ref();
                b.downcast_ref::<L::ItemBuffer>()
                    .is_some_and(|b| b.is_still_valid(&(), v))
            })
    }
}

enum ListWidgetEvent {
    Shift { index: usize, delta: i32 },
    Removal { index: usize },
    AppendDefault,
}

impl ListWidgetEvent {
    fn apply_transform(&self, itf: &IndexTransform) -> Option<Self> {
        match self {
            ListWidgetEvent::Shift { index, delta } => {
                itf.adjusted(*index).map(|i| ListWidgetEvent::Shift {
                    index: i,
                    delta: *delta,
                })
            }
            ListWidgetEvent::Removal { index } => itf
                .adjusted(*index)
                .map(|i| ListWidgetEvent::Removal { index: i }),
            ListWidgetEvent::AppendDefault => Some(ListWidgetEvent::AppendDefault),
        }
    }
}
