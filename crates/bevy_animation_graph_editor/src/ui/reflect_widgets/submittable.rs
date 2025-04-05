use std::{any::Any, marker::PhantomData};

use bevy::reflect::{FromReflect, Reflect, Reflectable};
use bevy_inspector_egui::reflect_inspector::InspectorUi;
use egui_dock::egui;

use super::{EguiInspectorExtension, HashExt, IntoBuffer};

#[derive(Default)]
pub struct SubmittableInspector<T> {
    _phantom_data: PhantomData<T>,
}

/// Wrapping a type in submittable is used to create a reflect-based editor that
/// 1. Uses the editor widget for the inner type
/// 2. Has a "submit" method and does not update the original value until submission
#[derive(Clone, Reflect, Debug, Default)]
pub struct Submittable<T> {
    pub value: T,
}

impl<T: Clone + Reflectable + FromReflect + Default> EguiInspectorExtension
    for SubmittableInspector<T>
{
    type Base = Submittable<T>;
    type Buffer = T;

    fn mutable(
        value: &mut Self::Base,
        buffer: &mut Self::Buffer,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        mut env: InspectorUi<'_, '_>,
    ) -> bool {
        env.ui_for_reflect_with_options(buffer, ui, id, options);
        if ui.button("Submit").clicked() {
            *value = Submittable {
                value: buffer.clone(),
            };
            true
        } else {
            false
        }
    }

    fn readonly(
        value: &Self::Base,
        _buffer: &Self::Buffer,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        mut env: InspectorUi<'_, '_>,
    ) {
        env.ui_for_reflect_readonly_with_options(&value.value, ui, id, options);
    }
}

impl<T: Clone> IntoBuffer<T> for Submittable<T> {
    fn into_buffer(&self) -> T {
        self.value.clone()
    }
}

impl<T: HashExt> HashExt for Submittable<T> {
    fn hash_ext(&self) -> u64 {
        self.value.hash_ext()
    }
}
