use bevy::math::Vec3;

use crate::ui::{
    generic_widgets::vec3::Vec3Widget,
    reflect_lib::{ReflectWidget, ReflectWidgetContext},
};

#[derive(Default)]
pub struct Vec3ReflectWidget;

impl ReflectWidget for Vec3ReflectWidget {
    type Target = Vec3;

    fn draw(
        &self,
        ui: &mut egui::Ui,
        value: &mut Self::Target,
        _: &ReflectWidgetContext,
    ) -> egui::Response {
        ui.add(Vec3Widget::new_salted(value, "vec3 widget"))
    }
}
