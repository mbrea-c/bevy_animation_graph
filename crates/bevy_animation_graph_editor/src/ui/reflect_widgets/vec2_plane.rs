use std::any::Any;

use bevy::math::Vec2;
use bevy_inspector_egui::reflect_inspector::InspectorUi;
use egui::Widget;
use egui_dock::egui;

use crate::ui::utils::popup::CustomPopup;

use super::{EguiInspectorExtension, MakeBuffer, WidgetHash};

pub struct Vec2PlaneInspector;

impl EguiInspectorExtension for Vec2PlaneInspector {
    type Base = Vec2;
    type Buffer = ();

    fn mutable(
        value: &mut Self::Base,
        (): &mut Self::Buffer,
        ui: &mut egui::Ui,
        _options: &dyn Any,
        id: egui::Id,
        _env: InspectorUi<'_, '_>,
    ) -> bool {
        let mut changed = false;

        ui.push_id(id, |ui| {
            let mut should_open_picker = false;

            ui.horizontal(|ui| {
                if egui::DragValue::new(&mut value.x).ui(ui).changed() {
                    changed = true;
                }
                if egui::DragValue::new(&mut value.y).ui(ui).changed() {
                    changed = true;
                }

                if ui.button("⌖").clicked() {
                    should_open_picker = true;
                }
            });

            CustomPopup::new()
                .with_allow_opening(false)
                .with_force_open(should_open_picker)
                .with_save_on_click(Some(()))
                .show_if_saved(ui, |ui, ()| {
                    let desired_size = egui::Vec2::new(150., 150.);
                    let (_, area_rect) = ui.allocate_space(desired_size);

                    let response = ui.interact(
                        area_rect,
                        ui.id().with("vec2 plane editor"),
                        egui::Sense::click_and_drag(),
                    );

                    if let Some(interact_pos) = response.interact_pointer_pos() {
                        if response.clicked() || response.dragged_by(egui::PointerButton::Primary) {
                            let clipped_pos = area_rect.clamp(interact_pos);
                            let bevy_vec = Vec2::new(
                                (clipped_pos.x - area_rect.left()) / area_rect.width() * 2. - 1.,
                                -((clipped_pos.y - area_rect.top()) / area_rect.height() * 2. - 1.),
                            );
                            *value = bevy_vec;
                            changed = true;
                        }
                    }

                    let scaled_vec = Vec2::new(
                        ((value.x + 1.) / 2.) * area_rect.width() + area_rect.left(),
                        ((-value.y + 1.) / 2.) * area_rect.height() + area_rect.top(),
                    );

                    let axes_color = egui::Color32::GRAY;

                    ui.painter()
                        .rect_filled(area_rect, 3., egui::Color32::BLACK);

                    ui.painter().line_segment(
                        [area_rect.left_center(), area_rect.right_center()],
                        egui::Stroke {
                            width: 1.,
                            color: axes_color,
                        },
                    );

                    ui.painter().line_segment(
                        [area_rect.center_top(), area_rect.center_bottom()],
                        egui::Stroke {
                            width: 1.,
                            color: axes_color,
                        },
                    );

                    ui.painter().circle_filled(
                        egui::Pos2::new(scaled_vec.x, scaled_vec.y),
                        2.,
                        egui::Color32::RED,
                    );
                });
        });

        changed
    }

    fn readonly(
        value: &Self::Base,
        _buffer: &Self::Buffer,
        ui: &mut egui::Ui,
        _options: &dyn Any,
        _id: egui::Id,
        _env: InspectorUi<'_, '_>,
    ) {
        ui.label(format!("({}, {})", value.x, value.y));
    }
}

impl WidgetHash for Vec2 {
    // HACK: We really should not call this operation a widget "hash", but rather
    // a cache unique identifier generation or something along those lines
    fn widget_hash(&self) -> u64 {
        unsafe { std::mem::transmute(self.to_array()) }
    }
}

impl MakeBuffer<()> for Vec2 {
    fn make_buffer(&self) {}
}
