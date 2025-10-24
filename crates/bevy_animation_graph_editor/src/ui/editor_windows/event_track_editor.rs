use std::{any::TypeId, hash::Hash};

use bevy::{
    asset::{Assets, Handle},
    platform::collections::HashMap,
    prelude::World,
    reflect::Reflect,
    utils::default,
};
use bevy_animation_graph::{
    core::{
        animation_graph::NodeId,
        edge_data::AnimationEvent,
        event_track::{EventTrack, TrackItem, TrackItemValue},
    },
    nodes::EventMarkupNode,
    prelude::{AnimationGraph, GraphClip},
};
use egui_dock::egui;
use uuid::Uuid;

use crate::ui::{
    actions::{
        EditorAction,
        event_tracks::{EditEventAction, EventTrackAction, NewEventAction, NewTrackAction},
        window::{DynWindowAction, TypeTargetedWindowAction},
    },
    core::{EditorWindowExtension, LegacyEditorWindowContext},
    reflect_widgets::{submittable::Submittable, wrap_ui::using_wrap_ui},
    utils::popup::CustomPopup,
    windows::WindowId,
};

use super::animation_clip_preview::{ClipPreviewAction, ClipPreviewWindow, TimingOrder};

#[derive(Debug)]
pub struct EventTrackEditorWindow {
    scroll_config: ScrollConfig,
    selected_track: Option<String>,
    selected_event: Option<Uuid>,
    target_tracks: Option<TargetTracks>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollConfig {
    /// The time at the center of the window, in seconds
    center_time: f32,
    /// How much time is shown in the window, in seconds
    time_range: f32,
    /// How much we have scrolled "down", in tracks. It's possible to scroll a fraction of a track
    vertical_scroll: f32,
}

#[derive(Debug, Clone, Reflect, Hash)]
pub enum TargetTracks {
    Clip(Handle<GraphClip>),
    GraphNode {
        graph: Handle<AnimationGraph>,
        node: NodeId,
    },
}

#[derive(Debug, Clone, Copy)]
struct ActiveTracks<'a> {
    target: &'a TargetTracks,
    tracks: &'a HashMap<String, EventTrack>,
}

pub enum EventTrackEditorAction {
    SelectEvent { track_name: String, event_id: Uuid },
    SetScrollConfig(ScrollConfig),
    SetTrackSource(Option<TargetTracks>),
}

impl Default for EventTrackEditorWindow {
    fn default() -> Self {
        Self {
            scroll_config: ScrollConfig {
                center_time: 0.,
                time_range: 5.,
                vertical_scroll: 0.,
            },
            selected_track: None,
            selected_event: None,
            target_tracks: None,
        }
    }
}

impl EditorWindowExtension for EventTrackEditorWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut LegacyEditorWindowContext) {
        let timeline_height = 30.;

        let sister_window_id = ctx.windows.find_window_with_type::<ClipPreviewWindow>();
        let sister_window = sister_window_id
            .and_then(|id| ctx.windows.get_window(id))
            .and_then(|w| w.as_inner::<ClipPreviewWindow>());

        egui::SidePanel::left("Track names")
            .resizable(true)
            .default_width(200.)
            .min_width(100.)
            .show_inside(ui, |ui| {
                Self::with_tracks(
                    ui,
                    &self.target_tracks,
                    world,
                    |ui, world, active_tracks| {
                        self.draw_track_names(ui, world, active_tracks, 2. * timeline_height, ctx);
                    },
                    |ui, _| {
                        ui.label("No event track selected");
                    },
                );
            });

        egui::SidePanel::right("Event details")
            .resizable(true)
            .default_width(200.)
            .min_width(100.)
            .show_inside(ui, |ui| {
                Self::with_event(
                    ui,
                    &self.target_tracks,
                    &self.selected_track,
                    &self.selected_event,
                    world,
                    |ui, world, event| {
                        self.draw_event_editor(ui, world, &event.value, ctx);
                    },
                    |_, _| {},
                );
            });

        egui::TopBottomPanel::top("Asset selector")
            .resizable(false)
            .exact_height(timeline_height)
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| {
                self.draw_track_source_selector(ui, world, ctx);
            });

        egui::TopBottomPanel::top("Timeline")
            .resizable(false)
            .exact_height(timeline_height)
            .frame(egui::Frame::NONE)
            .show_inside(ui, |ui| {
                self.draw_timeline(
                    ui,
                    sister_window.and_then(|w| w.current_time),
                    sister_window_id,
                    ctx,
                );
            });

        Self::with_tracks(
            ui,
            &self.target_tracks,
            world,
            |ui, world, active_tracks| {
                self.draw_event_tracks(ui, world, active_tracks, ctx);
            },
            |ui, _| {
                ui.label("No event track selected");
            },
        );
    }

    fn display_name(&self) -> String {
        "Edit tracks".to_string()
    }

    fn handle_action(&mut self, action: DynWindowAction) {
        let Ok(editor_action): Result<Box<EventTrackEditorAction>, _> = action.downcast() else {
            return;
        };

        match *editor_action {
            EventTrackEditorAction::SelectEvent {
                track_name,
                event_id,
            } => {
                self.selected_track = Some(track_name);
                self.selected_event = Some(event_id);
            }
            EventTrackEditorAction::SetScrollConfig(scroll_config) => {
                self.scroll_config = scroll_config
            }
            EventTrackEditorAction::SetTrackSource(target_tracks) => {
                self.target_tracks = target_tracks;
            }
        }
    }
}

impl EventTrackEditorWindow {
    fn draw_event_tracks(
        &self,
        ui: &mut egui::Ui,
        world: &mut World,
        active_tracks: ActiveTracks,
        ctx: &mut LegacyEditorWindowContext,
    ) {
        let available_size = ui.available_size();
        let (_, rect) = ui.allocate_space(available_size);

        let hovered_track_number = ui
            .input(|i| i.pointer.latest_pos())
            .and_then(|pos| rect.contains(pos).then_some(pos))
            .map(|pos| self.pixel_to_track(pos.y, rect).floor())
            .and_then(|pos| (pos >= 0.).then_some(pos))
            .map(|pos| pos as usize);

        let mut hovered_track = None;

        for (track_number, track) in Self::sorted_tracks(active_tracks).enumerate() {
            // Highlight currently hovered track row
            if let Some(hovered_track_number) =
                hovered_track_number.and_then(|t| (t == track_number).then_some(t))
            {
                hovered_track = Some(track.name.clone());
                ui.painter().rect_filled(
                    egui::Rect {
                        min: egui::Pos2::new(
                            rect.min.x,
                            self.track_to_pixel(hovered_track_number as f32, rect),
                        ),
                        max: egui::Pos2::new(
                            rect.max.x,
                            self.track_to_pixel((hovered_track_number + 1) as f32, rect),
                        ),
                    },
                    0.,
                    egui::Color32::from_white_alpha(10),
                );
            }
            for event in track.events.iter() {
                let response = self.draw_event(ui, rect, &event.value, track_number);
                if response.clicked() {
                    ctx.editor_actions.window(
                        ctx.window_id,
                        EventTrackEditorAction::SelectEvent {
                            track_name: track.name.clone(),
                            event_id: event.id,
                        },
                    );
                }
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
                let mut current_config = self.scroll_config;

                if i.modifiers.ctrl {
                    current_config.time_range *= 1. - total_scroll * 0.1;
                } else if i.modifiers.shift {
                    current_config.vertical_scroll -= total_scroll;
                } else {
                    current_config.center_time += total_scroll * 0.4;
                }

                if current_config != self.scroll_config {
                    ctx.editor_actions.window(
                        ctx.window_id,
                        EventTrackEditorAction::SetScrollConfig(current_config),
                    );
                }
            });
        }

        CustomPopup::new()
            .with_salt("Create event popup")
            .with_sense_rect(rect)
            .with_allow_opening(hovered_track.is_some())
            .with_save_on_click(hovered_track)
            .show_if_saved(ui, |ui, last_hovered_track| {
                ui.menu_button("New event", |ui| {
                    if let Some(new_item) = using_wrap_ui(world, |mut env| {
                        env.mutable_buffered(
                            &Submittable {
                                value: TrackItemValue::default(),
                            },
                            ui,
                            ui.id(),
                            &(),
                        )
                    }) {
                        ctx.editor_actions.push(EditorAction::EventTrack(
                            EventTrackAction::NewEvent(NewEventAction {
                                target_tracks: active_tracks.target.clone(),
                                track_id: last_hovered_track,
                                item: TrackItem::new(new_item.value),
                            }),
                        ));
                    }
                });
            });
    }

    /// Draw a single event in the track editor window
    fn draw_event(
        &self,
        ui: &mut egui::Ui,
        area_rect: egui::Rect,
        event: &TrackItemValue,
        track_number: usize,
    ) -> egui::Response {
        let pixel_x_start = self.time_to_pixel(event.start_time, area_rect);
        let pixel_x_end = self.time_to_pixel(event.end_time, area_rect);

        let pixel_y_start = self.track_to_pixel(track_number as f32, area_rect);
        let pixel_y_end = self.track_to_pixel((track_number + 1) as f32, area_rect);

        let start = egui::Pos2::new(pixel_x_start, pixel_y_start);
        let end = egui::Pos2::new(pixel_x_end, pixel_y_end);

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
            egui::StrokeKind::Middle,
        );

        if area_rect.intersects(rect) {
            ui.painter().text(
                clipped_rect.left_center(),
                egui::Align2::LEFT_CENTER,
                match &event.event {
                    AnimationEvent::StringId(s) => s.clone(),
                    x => format!("{x:?}"),
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
        world: &mut World,
        active_tracks: ActiveTracks,
        vertical_offset_pixels: f32,
        ctx: &mut LegacyEditorWindowContext,
    ) {
        let available_size = ui.available_size();
        let (_, area_rect) = ui.allocate_space(available_size);

        for (track_number, track) in active_tracks.tracks.values().enumerate() {
            let pixel_y_start =
                self.track_to_pixel(track_number as f32, area_rect) + vertical_offset_pixels;
            let pixel_y_end =
                self.track_to_pixel((track_number + 1) as f32, area_rect) + vertical_offset_pixels;

            let start = egui::Pos2::new(area_rect.left(), pixel_y_start);
            let end = egui::Pos2::new(area_rect.right(), pixel_y_end);

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

        CustomPopup::new()
            .with_salt("Create track popup")
            .with_sense_rect(area_rect)
            .with_allow_opening(true)
            .with_save_on_click(Some(()))
            .show_if_saved(ui, |ui, ()| {
                ui.menu_button("New track", |ui| {
                    if let Some(new_track) = using_wrap_ui(world, |mut env| {
                        env.mutable_buffered(
                            &Submittable {
                                value: "".to_string(),
                            },
                            ui,
                            ui.id(),
                            &(),
                        )
                    }) {
                        ctx.editor_actions.push(EditorAction::EventTrack(
                            EventTrackAction::NewTrack(NewTrackAction {
                                target_tracks: active_tracks.target.clone(),
                                track_id: new_track.value,
                            }),
                        ));
                    }
                })
            });
    }

    fn draw_timeline(
        &self,
        ui: &mut egui::Ui,
        current_time: Option<f32>,
        sister_window_id: Option<WindowId>,
        ctx: &mut LegacyEditorWindowContext,
    ) {
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
                format!("{indicator:.2}s"),
                egui::FontId {
                    size: 10.,
                    ..default()
                },
                egui::Color32::LIGHT_GRAY,
            );
        }

        // Draw current time
        if let Some(current_time) = current_time {
            let current_time_pixel_pos = self.time_to_pixel(current_time, area_rect);

            let points = vec![
                egui::Pos2::new(current_time_pixel_pos, area_rect.bottom()),
                egui::Pos2::new(current_time_pixel_pos - 8., area_rect.top()),
                egui::Pos2::new(current_time_pixel_pos + 8., area_rect.top()),
            ];

            let path_shape = egui::epaint::PathShape::convex_polygon(
                points,
                egui::Color32::RED, // fill color for the triangle
                egui::Stroke::new(0.0, egui::Color32::TRANSPARENT),
            );

            ui.painter().add(egui::Shape::Path(path_shape));
        }

        // Detect clicks on timeline to seek time
        if let Some(sister_window_id) = sister_window_id {
            if let Some(click_pos) = ui.input(|i| {
                if i.pointer.primary_down()
                    && i.pointer
                        .latest_pos()
                        .is_some_and(|p| area_rect.contains(p))
                {
                    i.pointer.latest_pos()
                } else {
                    None
                }
            }) {
                let time = self.pixel_to_time(click_pos.x, area_rect);
                ctx.editor_actions.window(
                    sister_window_id,
                    ClipPreviewAction::TimingOrder(TimingOrder::Seek { time }),
                );
            }
        }
    }

    fn draw_track_source_selector(
        &self,
        ui: &mut egui::Ui,
        world: &mut World,
        ctx: &mut LegacyEditorWindowContext,
    ) {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            if let Some(new_selection) = using_wrap_ui(world, |mut env| {
                env.mutable_buffered(
                    &self.target_tracks,
                    ui,
                    ui.id().with("track source selector"),
                    &(),
                )
            }) {
                ctx.editor_actions.window(
                    ctx.window_id,
                    EventTrackEditorAction::SetTrackSource(new_selection.clone()),
                );
                ctx.editor_actions.dynamic(TypeTargetedWindowAction {
                    target_window_type: TypeId::of::<ClipPreviewWindow>(),
                    action: Box::new(ClipPreviewAction::SelectTarget(new_selection)),
                });
            }
        });
    }

    fn draw_event_editor(
        &self,
        ui: &mut egui::Ui,
        world: &mut World,
        event: &TrackItemValue,
        ctx: &mut LegacyEditorWindowContext,
    ) {
        if let Some(edited_event) = using_wrap_ui(world, |mut env| {
            env.mutable_buffered(
                &Submittable {
                    value: event.clone(),
                },
                ui,
                ui.id().with("track event editor"),
                &(),
            )
        }) {
            ctx.editor_actions
                .push(EditorAction::EventTrack(EventTrackAction::EditEvent(
                    EditEventAction {
                        target_tracks: self.target_tracks.clone().unwrap(),
                        track_id: self.selected_track.clone().unwrap(),
                        event_id: self.selected_event.unwrap(),
                        item: edited_event.value,
                    },
                )));
        }
    }

    fn left_time(&self) -> f32 {
        self.scroll_config.center_time - self.scroll_config.time_range / 2.
    }

    fn right_time(&self) -> f32 {
        self.scroll_config.center_time + self.scroll_config.time_range / 2.
    }

    // Pretty naive time interval calculation. In the future we may want
    // to "snap" to more "rounder", human readable values
    fn timeline_interval(&self) -> f32 {
        self.scroll_config.time_range / 10.
    }

    /// Supports track fractional numbers
    fn track_to_pixel(&self, track_number: f32, viewport: egui::Rect) -> f32 {
        viewport.top() + (track_number - self.scroll_config.vertical_scroll) * 20.
    }

    fn time_to_pixel(&self, time: f32, viewport: egui::Rect) -> f32 {
        let time_to_pixel = viewport.width() / self.scroll_config.time_range;
        viewport.left() + (time - self.left_time()) * time_to_pixel
    }

    fn pixel_to_time(&self, pixel: f32, viewport: egui::Rect) -> f32 {
        let time_to_pixel = viewport.width() / self.scroll_config.time_range;
        (pixel - viewport.left()) / time_to_pixel + self.left_time()
    }

    /// Supports track fractional numbers
    fn pixel_to_track(&self, pixel: f32, viewport: egui::Rect) -> f32 {
        self.scroll_config.vertical_scroll + (pixel - viewport.top()) / 20.
    }

    fn with_event<F, G, T>(
        ui: &mut egui::Ui,
        target: &Option<TargetTracks>,
        track_id: &Option<String>,
        event_id: &Option<Uuid>,
        world: &mut World,
        f: F,
        g: G,
    ) -> T
    where
        F: FnOnce(&mut egui::Ui, &mut World, &TrackItem) -> T,
        G: FnOnce(&mut egui::Ui, &mut World) -> T + Clone,
    {
        let (Some(track_id), Some(event_id)) = (track_id, event_id) else {
            return g(ui, world);
        };

        let gg = g.clone();

        Self::with_tracks(
            ui,
            target,
            world,
            |ui, world, active_tracks| {
                if let Some(event) = active_tracks
                    .tracks
                    .get(track_id)
                    .and_then(|track| track.events.iter().find(|e| e.id == *event_id))
                {
                    f(ui, world, event)
                } else {
                    gg(ui, world)
                }
            },
            g,
        )
    }

    fn with_tracks<F, G, T>(
        ui: &mut egui::Ui,
        target: &Option<TargetTracks>,
        world: &mut World,
        f: F,
        g: G,
    ) -> T
    where
        F: FnOnce(&mut egui::Ui, &mut World, ActiveTracks) -> T,
        G: FnOnce(&mut egui::Ui, &mut World) -> T,
    {
        if let Some(target) = target {
            match target {
                TargetTracks::Clip(asset_id) => {
                    world.resource_scope::<Assets<GraphClip>, _>(|world, graph_clips| {
                        if let Some(tracks) =
                            graph_clips.get(asset_id.id()).map(|c| c.event_tracks())
                        {
                            f(ui, world, ActiveTracks { tracks, target })
                        } else {
                            g(ui, world)
                        }
                    })
                }
                TargetTracks::GraphNode { graph, node } => world
                    .resource_scope::<Assets<AnimationGraph>, _>(|world, anim_graphs| {
                        if let Some(tracks) = anim_graphs
                            .get(graph.id())
                            .and_then(|g| g.nodes.get(node))
                            .and_then(|n| n.inner.as_any().downcast_ref::<EventMarkupNode>())
                            .map(|n| &n.event_tracks)
                        {
                            f(ui, world, ActiveTracks { target, tracks })
                        } else {
                            g(ui, world)
                        }
                    }),
            }
        } else {
            g(ui, world)
        }
    }

    /// Returns the tracks sorted lexicographically
    fn sorted_tracks(tracks: ActiveTracks<'_>) -> impl Iterator<Item = &EventTrack> {
        let mut track_vec = tracks.tracks.iter().collect::<Vec<_>>();
        track_vec.sort_by_key(|(k, _)| *k);
        track_vec.into_iter().map(|t| t.1)
    }
}
