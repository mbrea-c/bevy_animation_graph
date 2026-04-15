use std::any::TypeId;

use bevy::reflect::{List, ListInfo, PartialReflect, Reflect, prelude::ReflectDefault};

use crate::ui::{
    generic_widgets::list_like::{ListLike, ListLikeWidget},
    reflect_lib::ReflectWidgetContext,
    utils::ui_buffer::{CloneBuffer, SelfContainedBuffer},
};

pub fn handle_list(
    ctx: &ReflectWidgetContext,
    ui: &mut egui::Ui,
    value: &mut (dyn Reflect + 'static),
    list_info: &ListInfo,
) -> egui::Response {
    let r_list = value.reflect_mut().as_list().unwrap();
    ListLikeWidget::new(&mut ReflectList(r_list, ctx, list_info.item_ty().id())).show(ui)
}

pub struct ReflectList<'a>(
    &'a mut (dyn List + 'static),
    &'a ReflectWidgetContext<'a>,
    TypeId,
);

impl<'a> ListLike for ReflectList<'a> {
    type Item = dyn PartialReflect + 'static;
    type ItemBuffer = ReflectItemBuffer;

    fn iter(&self) -> impl Iterator<Item = &Self::Item> {
        self.0.iter()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn shift_index(&mut self, index: usize, delta: i32) -> usize {
        let mut new_index = (index as i64 + delta as i64).clamp(0, self.len() as i64) as usize;

        if new_index > index {
            new_index = new_index.saturating_sub(1);
        }

        let item = self.0.remove(index);
        self.0.insert(new_index, item);

        new_index
    }

    fn remove(&mut self, index: usize) {
        self.0.remove(index);
    }

    fn update(&mut self, index: usize, buffer: &Self::ItemBuffer) -> bool {
        if buffer.0.is_none() {
            return false;
        }

        let val = buffer.value();
        self.0.remove(index);
        self.0.insert(index, val);

        true
    }

    fn push(&mut self, buffer: &Self::ItemBuffer) {
        if buffer.0.is_none() {
            return;
        }

        self.0.push(buffer.value());
    }

    fn edit_item(&self, ui: &mut egui::Ui, buffer: &mut Self::ItemBuffer) -> egui::Response {
        if let Some(val) = buffer.0.as_mut().and_then(|v| v.try_as_reflect_mut()) {
            self.1.draw(ui, val)
        } else {
            ui.label("Value is not Reflect")
        }
    }

    fn default(&self) -> Option<Box<Self::Item>> {
        if let Some(d) = self.1.bevy_registry.get_type_data::<ReflectDefault>(self.2) {
            Some(d.default())
        } else {
            None
        }
    }
}

pub struct ReflectItemBuffer(Option<Box<dyn PartialReflect + 'static>>);

impl Clone for ReflectItemBuffer {
    fn clone(&self) -> Self {
        Self(self.0.as_ref().and_then(|v| {
            v.reflect_clone().ok().map(|v| {
                let b: Box<dyn PartialReflect + 'static> = v;
                b
            })
        }))
    }
}

impl CloneBuffer<dyn PartialReflect + 'static, ()> for ReflectItemBuffer {
    fn new(_ui: &egui::Ui, (): &(), value: &(dyn PartialReflect + 'static)) -> Self {
        Self(value.reflect_clone().ok().map(|v| {
            let b: Box<dyn PartialReflect + 'static> = v;
            b
        }))
    }

    fn id(&self, ui: &egui::Ui) -> egui::Id {
        ui.id().with(self.0.as_ref().map(|v| v.reflect_hash()))
    }

    fn is_still_valid(&self, (): &(), value: &(dyn PartialReflect + 'static)) -> bool {
        self.0
            .as_ref()
            .is_some_and(|v| v.reflect_partial_eq(value).is_some_and(|v| v))
    }
}

impl SelfContainedBuffer<dyn PartialReflect + 'static, ()> for ReflectItemBuffer {
    fn value(&self) -> Box<dyn PartialReflect + 'static> {
        self.clone().0.unwrap()
    }
}
