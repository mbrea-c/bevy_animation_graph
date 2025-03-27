use bevy::{prelude::World, utils::default};
use bevy_animation_graph::core::{
    edge_data::AnimationEvent,
    event_track::{EventTrack, TrackItem},
};
use egui_dock::egui;

use crate::ui::core::{EditorWindowContext, EditorWindowExtension};

#[derive(Debug)]
pub struct EventTrackEditorWindow {
    /// The time at the center of the window, in seconds
    center_time: f32,
    /// How much time is shown in the window, in seconds
    time_range: f32,
    /// How much we have scrolled "down", in tracks. It's possible to scroll a fraction of a track
    vertical_scroll: f32,
    being_edited: Vec<EventTrack>,
}

impl Default for EventTrackEditorWindow {
    fn default() -> Self {
        Self {
            center_time: 0.,
            time_range: 5.,
            vertical_scroll: 0.,
            being_edited: vec![
                EventTrack {
                    name: "Basic".into(),
                    events: vec![TrackItem {
                        event: AnimationEvent::StringId("test".into()),
                        start_time: 1.,
                        end_time: 4.,
                    }],
                },
                EventTrack {
                    name: "Packed".into(),
                    events: vec![
                        TrackItem {
                            event: AnimationEvent::StringId("first".into()),
                            start_time: 0.,
                            end_time: 3.,
                        },
                        TrackItem {
                            event: AnimationEvent::StringId("second".into()),
                            start_time: 3.,
                            end_time: 5.,
                        },
                    ],
                },
                EventTrack {
                    name: "Overlapping".into(),
                    events: vec![
                        TrackItem {
                            event: AnimationEvent::StringId("other first".into()),
                            start_time: 0.,
                            end_time: 4.,
                        },
                        TrackItem {
                            event: AnimationEvent::StringId("other second".into()),
                            start_time: 3.,
                            end_time: 5.,
                        },
                    ],
                },
            ],
        }
    }
}

impl EditorWindowExtension for EventTrackEditorWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        let timeline_height = 30.;

        egui::SidePanel::left("Track names")
            .resizable(true)
            .default_width(200.)
            .min_width(100.)
            .show_inside(ui, |ui| {
                self.draw_track_names(ui, &self.being_edited, timeline_height);
            });

        egui::SidePanel::right("Event details")
            .resizable(true)
            .default_width(200.)
            .min_width(100.)
            .show_inside(ui, |ui| {
                ui.label("We figuring this out");
            });

        egui::TopBottomPanel::top("Timeline")
            .resizable(false)
            .exact_height(timeline_height)
            .frame(egui::Frame::none())
            .show_inside(ui, |ui| {
                self.draw_timeline(ui);
            });

        let available_size = ui.available_size();
        let (_, rect) = ui.allocate_space(available_size);

        for (track_number, track) in self.being_edited.iter().enumerate() {
            for event in &track.events {
                self.draw_event(ui, rect, event, track_number);
            }
        }

        if ui.rect_contains_pointer(rect) {
            ui.ctx().input(|i| {
                let mut total_scroll = 0.;
                for event in &i.events {
                    if let egui::Event::MouseWheel { unit, delta, .. } = event {
                        total_scroll += delta.y
                            * match unit {
                                egui::MouseWheelUnit::Point => 0.1,
                                egui::MouseWheelUnit::Line => 1.,
                                egui::MouseWheelUnit::Page => 10.,
                            };
                    }
                }
                if i.modifiers.ctrl {
                    self.time_range *= 1. - total_scroll * 0.1;
                } else if i.modifiers.shift {
                    self.vertical_scroll -= total_scroll;
                } else {
                    self.center_time += total_scroll * 0.4;
                }
            });
        }
    }

    fn display_name(&self) -> String {
        "Edit tracks".to_string()
    }
}

impl EventTrackEditorWindow {
    /// Draw a single event in the track editor window
    fn draw_event(
        &self,
        ui: &mut egui::Ui,
        area_rect: egui::Rect,
        event: &TrackItem,
        track_number: usize,
    ) -> egui::Response {
        let time_to_pixel = area_rect.width() / self.time_range;
        let viewport_relative_start = event.start_time - self.left_time();
        let viewport_relative_end = event.end_time - self.left_time();

        // These are relative to the "area rect"
        let pixel_x_start = time_to_pixel * viewport_relative_start;
        let pixel_x_end = time_to_pixel * viewport_relative_end;

        let pixel_y_start = self.track_to_viewport_pixel(track_number as f32);
        let pixel_y_end = self.track_to_viewport_pixel((track_number + 1) as f32);

        let start = area_rect.left_top() + egui::Vec2::new(pixel_x_start, pixel_y_start);
        let end = area_rect.left_top() + egui::Vec2::new(pixel_x_end, pixel_y_end);

        let rect = egui::Rect {
            min: start,
            max: end,
        };

        let clipped_rect = area_rect.intersect(rect);

        let response = ui.interact(
            rect,
            ui.id().with((&event.event, track_number)),
            egui::Sense::click_and_drag(),
        );

        let color = if response.hovered() {
            egui::Color32::LIGHT_BLUE
        } else {
            egui::Color32::BLUE
        };

        ui.painter().rect_filled(clipped_rect, 2., color);
        ui.painter().rect_stroke(
            clipped_rect,
            2.,
            egui::Stroke {
                width: 1.,
                color: egui::Color32::LIGHT_GRAY,
            },
        );

        if area_rect.intersects(rect) {
            ui.painter().text(
                clipped_rect.left_center(),
                egui::Align2::LEFT_CENTER,
                match &event.event {
                    AnimationEvent::StringId(s) => s.clone(),
                    x => format!("{:?}", x),
                },
                egui::FontId::default(),
                egui::Color32::LIGHT_GRAY,
            );
        }

        response
    }

    fn draw_track_names(
        &self,
        ui: &mut egui::Ui,
        tracks: &Vec<EventTrack>,
        vertical_offset_pixels: f32,
    ) {
        let available_size = ui.available_size();
        let (_, area_rect) = ui.allocate_space(available_size);

        for (track_number, track) in tracks.iter().enumerate() {
            let pixel_y_start =
                self.track_to_viewport_pixel(track_number as f32) + vertical_offset_pixels;
            let pixel_y_end =
                self.track_to_viewport_pixel((track_number + 1) as f32) + vertical_offset_pixels;

            let start = egui::Pos2::new(area_rect.left(), area_rect.top() + pixel_y_start);
            let end = egui::Pos2::new(area_rect.right(), area_rect.top() + pixel_y_end);

            let rect = egui::Rect {
                min: start,
                max: end,
            };

            let clipped_rect = area_rect.intersect(rect);
            if area_rect.intersects(rect) {
                ui.painter().text(
                    clipped_rect.left_center(),
                    egui::Align2::LEFT_CENTER,
                    &track.name,
                    egui::FontId::default(),
                    egui::Color32::LIGHT_GRAY,
                );
            }
        }
    }

    fn draw_timeline(&self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();
        let (_, area_rect) = ui.allocate_space(available_size);
        let height = area_rect.height();

        let interval = self.timeline_interval();
        let first_indicator = self.left_time().div_euclid(interval) * interval;
        let indicators = (0..50)
            .map(|n| first_indicator + (n as f32) * interval)
            .filter(|&n| n >= self.left_time() && n <= self.right_time())
            .collect::<Vec<_>>();

        for indicator in indicators {
            let pixel_pos = self.time_to_pixel(indicator, area_rect);
            // draw time indicator in the bottom half of timeline
            ui.painter().line_segment(
                [
                    egui::Pos2::new(pixel_pos, area_rect.bottom()),
                    egui::Pos2::new(pixel_pos, area_rect.bottom() - height / 2.),
                ],
                egui::Stroke {
                    width: 0.3,
                    color: egui::Color32::LIGHT_GRAY,
                },
            );

            ui.painter().text(
                egui::Pos2::new(pixel_pos, area_rect.bottom() - height / 2.),
                egui::Align2::CENTER_BOTTOM,
                format!("{:.2}s", indicator),
                egui::FontId {
                    size: 10.,
                    ..default()
                },
                egui::Color32::LIGHT_GRAY,
            );
        }
    }

    fn draw_event_editor(&self, ui: &mut egui::Ui, event: &TrackItem) {}

    fn left_time(&self) -> f32 {
        self.center_time - self.time_range / 2.
    }

    fn right_time(&self) -> f32 {
        self.center_time + self.time_range / 2.
    }

    // Pretty naive time interval calculation. In the future we may want
    // to "snap" to more "rounder", human readable values
    fn timeline_interval(&self) -> f32 {
        self.time_range / 10.
    }

    /// Supports track fractional numbers
    fn track_to_viewport_pixel(&self, track_number: f32) -> f32 {
        (track_number - self.vertical_scroll) * 20.
    }

    fn time_to_pixel(&self, time: f32, viewport: egui::Rect) -> f32 {
        let time_to_pixel = viewport.width() / self.time_range;
        viewport.left() + (time - self.left_time()) * time_to_pixel
    }
}
