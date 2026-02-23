use std::marker::PhantomData;

use crate::ui::reflect_lib::{ReflectWidget, WidgetRegistry};

pub fn default_widget_registry() -> WidgetRegistry {
    let mut registry = WidgetRegistry::default();

    registry.add(StringReflectWidget);
    registry.add(NumericReflectWidget::<f32>::default());
    registry.add(NumericReflectWidget::<f64>::default());
    registry.add(NumericReflectWidget::<u32>::default());
    registry.add(NumericReflectWidget::<u64>::default());
    registry.add(NumericReflectWidget::<i32>::default());
    registry.add(NumericReflectWidget::<i64>::default());

    registry
}

pub struct StringReflectWidget;

impl ReflectWidget for StringReflectWidget {
    type Target = String;

    fn draw(
        &self,
        ui: &mut egui::Ui,
        value: &mut Self::Target,
        _: &super::ReflectWidgetContext,
    ) -> egui::Response {
        ui.text_edit_singleline(value)
    }
}

#[derive(Default)]
pub struct NumericReflectWidget<T> {
    override_speed: Option<f32>,
    __t: PhantomData<T>,
}

impl<T: egui::emath::Numeric + Send + Sync> ReflectWidget for NumericReflectWidget<T> {
    type Target = T;

    fn draw(
        &self,
        ui: &mut egui::Ui,
        value: &mut Self::Target,
        _: &super::ReflectWidgetContext,
    ) -> egui::Response {
        let mut w = egui::DragValue::new(value);
        if let Some(speed) = self.override_speed {
            w = w.speed(speed);
        }
        ui.add(w)
    }
}
