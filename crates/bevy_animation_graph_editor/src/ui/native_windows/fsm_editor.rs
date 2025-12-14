use core::f32;

use bevy::{
    asset::{Assets, Handle},
    ecs::{component::Component, entity::Entity, event::EntityEvent, observer::On, system::Query},
    math::Vec2,
    platform::collections::{HashMap, HashSet},
    prelude::World,
};
use bevy_animation_graph::core::state_machine::high_level::{
    State, StateId, StateMachine, TransitionId, TransitionVariant,
};
use egui_dock::egui;

use crate::ui::{
    native_windows::{
        EditorWindowContext, EditorWindowRegistrationContext, NativeEditorWindowExtension,
        OwnedQueue,
    },
    state_management::global::{
        RegisterStateComponent,
        active_fsm::ActiveFsm,
        active_fsm_state::{ActiveFsmState, SetActiveFsmState},
        active_fsm_transition::{ActiveFsmTransition, SetActiveFsmTransition},
        fsm::MoveStates,
        get_global_state,
        inspector_selection::{InspectorSelection, SetInspectorSelection},
        register_if_missing,
    },
};

pub struct FsmEditBuffer {
    pub scene_rect: egui::Rect,
    pub drag_start: Vec2,
}

impl Default for FsmEditBuffer {
    fn default() -> Self {
        Self {
            scene_rect: egui::Rect::ZERO,
            drag_start: Vec2::ZERO,
        }
    }
}

#[derive(Debug)]
pub struct FsmEditorWindow;

impl NativeEditorWindowExtension for FsmEditorWindow {
    fn init(&self, world: &mut World, ctx: &EditorWindowRegistrationContext) {
        register_if_missing::<FsmEditorWindowState>(world, ctx.window);
    }

    fn ui(&self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        let (Some(active_fsm), Some(window_state)) = (
            get_global_state::<ActiveFsm>(world).cloned(),
            ctx.get_window_state::<FsmEditorWindowState>(world).cloned(),
        ) else {
            ui.centered_and_justified(|ui| ui.label("Select a state machine to edit!"));
            return;
        };

        let mut queue = ctx.make_queue();
        let window = ctx.window_entity;

        let buffer_id = ui.id().with("Fsm editor nodes context buffer");
        let buffer = ctx.buffers.get_mut_or_default(buffer_id);

        world.resource_scope::<Assets<StateMachine>, _>(|_, fsm_assets| {
            let fsm = fsm_assets.get(&active_fsm.handle)?;

            window_state.draw_fsm(ui, fsm, buffer, &mut queue, window, &active_fsm.handle);

            Some(())
        });

        ctx.consume_queue(queue);
    }

    fn display_name(&self) -> String {
        "FSM Editor".to_string()
    }
}

#[derive(Component, Clone)]
pub struct FsmEditorWindowState {
    pub selected_states: HashSet<StateId>,
    pub selected_transitions: HashSet<TransitionId>,
    pub style: FsmEditorStyle,
}

impl RegisterStateComponent for FsmEditorWindowState {
    fn register(world: &mut World, state_entity: Entity) {
        world
            .entity_mut(state_entity)
            .insert(FsmEditorWindowState::default())
            .observe(ClearSelections::observe)
            .observe(SelectStates::observe)
            .observe(SelectTransitions::observe);
    }
}

impl Default for FsmEditorWindowState {
    fn default() -> Self {
        Self {
            selected_states: Default::default(),
            selected_transitions: Default::default(),
            style: Default::default(),
        }
    }
}

impl FsmEditorWindowState {
    fn draw_fsm(
        &self,
        ui: &mut egui::Ui,
        fsm: &StateMachine,
        buffer: &mut FsmEditBuffer,
        queue: &mut OwnedQueue,
        window: Entity,
        fsm_handle: &Handle<StateMachine>,
    ) -> egui::Response {
        let FsmEditBuffer {
            scene_rect,
            drag_start,
            ..
        } = buffer;
        let last_rect = *scene_rect;
        egui::Scene::new()
            .drag_pan_buttons(egui::DragPanButtons::MIDDLE)
            .show(ui, scene_rect, |ui| {
                self.draw_editor_background(ui, last_rect.expand(500.));

                let global_drag = ui.allocate_rect(last_rect, egui::Sense::drag());

                if global_drag.dragged_by(egui::PointerButton::Primary)
                    && let Some(pos) = global_drag.hover_pos()
                {
                    if global_drag.drag_started() {
                        *drag_start = bevy_vec2_from_pos2(pos - global_drag.drag_delta())
                    }

                    let drag_rect = egui::Rect::from_two_pos(egui_pos2(*drag_start), pos);

                    self.draw_selection_box(ui, drag_rect);

                    let (new_selected_states, new_selected_transitions) =
                        self.compute_updated_selections(fsm, drag_rect);

                    queue.trigger(ClearSelections { entity: window });
                    queue.trigger(SelectStates {
                        entity: window,
                        states: new_selected_states,
                    });
                    queue.trigger(SelectTransitions {
                        entity: window,
                        transitions: new_selected_transitions,
                    });
                }

                self.draw_direct_transitions(ui, fsm, queue, fsm_handle);
                self.draw_states(ui, fsm, queue, fsm_handle);
            })
            .response
    }

    fn draw_editor_background(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        let separation_int = 40;
        let separation = separation_int as f32;

        ui.painter().rect_filled(rect, 0., self.style.bg);

        let start_x = rect.left().div_euclid(separation) as i32;
        let end_x = (rect.right().div_euclid(separation) + separation) as i32;
        for i in start_x..=end_x {
            let x = (i * separation_int) as f32;
            ui.painter().line_segment(
                [
                    egui::Pos2::new(x, rect.top()),
                    egui::Pos2::new(x, rect.bottom()),
                ],
                egui::Stroke {
                    width: 1.,
                    color: self.style.bg_grid,
                },
            );
        }

        let start_y = rect.top().div_euclid(separation) as i32;
        let end_y = (rect.bottom().div_euclid(separation) + separation) as i32;
        for i in start_y..=end_y {
            let y = (i * separation_int) as f32;
            ui.painter().line_segment(
                [
                    egui::Pos2::new(rect.left(), y),
                    egui::Pos2::new(rect.right(), y),
                ],
                egui::Stroke {
                    width: 2.,
                    color: self.style.bg_grid,
                },
            );
        }
    }

    fn draw_selection_box(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        ui.painter().rect(
            rect,
            0.,
            self.style.selection_box_fill,
            egui::Stroke {
                width: 1.,
                color: self.style.selection_box_border,
            },
            egui::StrokeKind::Middle,
        );
    }

    fn compute_updated_selections(
        &self,
        fsm: &StateMachine,
        rect: egui::Rect,
    ) -> (HashSet<StateId>, HashSet<TransitionId>) {
        let mut selected_states = HashSet::new();

        for state in fsm.states.values() {
            let pos = fsm
                .extra
                .states
                .get(&state.id)
                .copied()
                .unwrap_or(Vec2::ZERO);
            let state_rect = Self::state_rect(pos);
            if rect.intersects(state_rect) {
                selected_states.insert(state.id);
            }
        }

        let mut selected_transitions = HashSet::new();

        let direct_transitions = Self::collapse_direct_transitions(fsm);
        for transitions in direct_transitions.values() {
            for (i, transition_id) in transitions.iter().enumerate() {
                let Some(transition) = fsm.transitions.get(transition_id) else {
                    continue;
                };
                let TransitionVariant::Direct { source, target } = &transition.variant else {
                    continue;
                };
                let source_pos = fsm.extra.states.get(source).copied().unwrap_or(Vec2::ZERO);
                let target_pos = fsm.extra.states.get(target).copied().unwrap_or(Vec2::ZERO);
                let data =
                    Self::transition_arrow_endpoints(source_pos, target_pos, i, transitions.len());
                if rect.intersects_ray(egui_pos2(data.start), egui_vec2(data.dir))
                    && rect.intersects_ray(egui_pos2(data.end), egui_vec2(-data.dir))
                {
                    selected_transitions.insert(*transition_id);
                }
            }
        }

        (selected_states, selected_transitions)
    }

    fn draw_states(
        &self,
        ui: &mut egui::Ui,
        fsm: &StateMachine,
        queue: &mut OwnedQueue,
        fsm_handle: &Handle<StateMachine>,
    ) {
        for state in fsm.states.values() {
            let pos = fsm
                .extra
                .states
                .get(&state.id)
                .copied()
                .unwrap_or(Vec2::ZERO);
            self.draw_state(
                ui,
                state,
                pos,
                fsm.start_state == state.id,
                None, // TODO: get correct one
                queue,
                fsm_handle,
            );
        }
    }

    fn collapse_direct_transitions(
        fsm: &StateMachine,
    ) -> HashMap<(StateId, StateId), Vec<TransitionId>> {
        let mut direct_transitions: HashMap<(StateId, StateId), Vec<TransitionId>> = HashMap::new();
        for transition in fsm.transitions.values() {
            match &transition.variant {
                TransitionVariant::Direct { source, target } => {
                    let key = match source.cmp(target) {
                        std::cmp::Ordering::Less => (*source, *target),
                        _ => (*target, *source),
                    };
                    direct_transitions
                        .entry(key)
                        .or_insert(vec![])
                        .push(transition.id);
                }
                TransitionVariant::State { .. } => {}
            }
        }

        direct_transitions
    }

    fn draw_direct_transitions(
        &self,
        ui: &mut egui::Ui,
        fsm: &StateMachine,
        queue: &mut OwnedQueue,
        fsm_handle: &Handle<StateMachine>,
    ) {
        let direct_transitions = Self::collapse_direct_transitions(fsm);
        for transitions in direct_transitions.values() {
            for (i, transition_id) in transitions.iter().enumerate() {
                let Some(transition) = fsm.transitions.get(transition_id) else {
                    continue;
                };
                let TransitionVariant::Direct { source, target } = &transition.variant else {
                    continue;
                };
                let source_pos = fsm.extra.states.get(source).copied().unwrap_or(Vec2::ZERO);
                let target_pos = fsm.extra.states.get(target).copied().unwrap_or(Vec2::ZERO);
                self.draw_direct_transition(
                    ui,
                    *transition_id,
                    source_pos,
                    target_pos,
                    i,
                    transitions.len(),
                    queue,
                    fsm_handle,
                );
            }
        }
    }

    const STATE_WIDTH: f32 = 100.;
    const STATE_HEIGHT: f32 = 50.;

    fn state_rect(pos: Vec2) -> egui::Rect {
        egui::Rect {
            min: egui::Pos2::new(
                pos.x - Self::STATE_WIDTH / 2.,
                pos.y - Self::STATE_HEIGHT / 2.,
            ),
            max: egui::Pos2::new(
                pos.x + Self::STATE_WIDTH / 2.,
                pos.y + Self::STATE_HEIGHT / 2.,
            ),
        }
    }

    fn draw_state(
        &self,
        ui: &mut egui::Ui,
        state: &State,
        pos: Vec2,
        is_start_state: bool,
        state_transition: Option<TransitionId>,
        queue: &mut OwnedQueue,
        fsm: &Handle<StateMachine>,
    ) -> egui::Response {
        let rect = Self::state_rect(pos);

        let state_response = ui.allocate_rect(rect, egui::Sense::click_and_drag());

        if state_response.clicked() {
            queue.trigger_window(ClearSelections {
                entity: Entity::PLACEHOLDER,
            });

            queue.trigger_window(SelectStates::placeholder([state.id].into()));
            queue.trigger(SetActiveFsmState {
                new: ActiveFsmState {
                    handle: fsm.clone(),
                    state: state.id,
                },
            });
            queue.trigger(SetInspectorSelection {
                selection: InspectorSelection::ActiveFsmState,
            });
        }

        if state_response.dragged_by(egui::PointerButton::Primary) {
            if self.selected_states.contains(&state.id) {
                queue.trigger(MoveStates {
                    fsm: fsm.clone(),
                    states: self.selected_states.clone(),
                    delta: bevy_vec2_from_vec2(state_response.drag_delta()),
                });
            } else {
                queue.trigger_window(ClearSelections {
                    entity: Entity::PLACEHOLDER,
                });
                queue.trigger_window(SelectStates::placeholder([state.id].into()));
                queue.trigger(MoveStates {
                    fsm: fsm.clone(),
                    states: [state.id].into(),
                    delta: bevy_vec2_from_vec2(state_response.drag_delta()),
                });
            }
        }

        let hovered = state_response.hovered();
        let selected = self.selected_states.contains(&state.id);

        ui.painter().rect(
            rect,
            10.,
            self.style.state_bg.get(hovered, selected),
            self.style.state_border.get(hovered, selected),
            egui::StrokeKind::Outside,
        );
        if is_start_state {
            ui.painter().rect(
                rect.shrink(5.),
                10.,
                egui::Color32::TRANSPARENT,
                self.style.state_border.get(false, false),
                egui::StrokeKind::Outside,
            );
        }

        ui.painter().text(
            egui::Pos2::new(pos.x, pos.y),
            egui::Align2::CENTER_CENTER,
            &state.label,
            egui::FontId::proportional(13.),
            ui.visuals().text_color(),
        );

        state_response
    }

    fn transition_arrow_endpoints(
        source_pos: Vec2,
        target_pos: Vec2,
        offset: usize,
        total_transitions_for_state_pair: usize,
    ) -> TransitionDrawData {
        let mut direction = target_pos - source_pos;
        let flip = direction.dot(Vec2::Y) < 0.
            || (direction.dot(Vec2::Y) == 0. && direction.dot(Vec2::X) < 0.);
        if flip {
            direction *= -1.;
        }
        let normal = direction.perp().normalize();

        const SEPARATION: f32 = 10.;

        let total_width = (total_transitions_for_state_pair - 1) as f32 * SEPARATION;
        let vec_offset = (-total_width / 2. + SEPARATION * offset as f32) * normal;
        let dir = (target_pos - source_pos).normalize();
        let start = source_pos + dir * 50. + vec_offset;
        let end = target_pos - dir * 50. + vec_offset;
        let shaft_end = end - 10. * dir;

        TransitionDrawData {
            start,
            end,
            normal,
            dir,
            shaft_end,
        }
    }

    fn draw_direct_transition(
        &self,
        ui: &mut egui::Ui,
        transition_id: TransitionId,
        source_pos: Vec2,
        target_pos: Vec2,
        offset: usize,
        total_transitions_for_state_pair: usize,
        queue: &mut OwnedQueue,
        fsm: &Handle<StateMachine>,
    ) {
        let TransitionDrawData {
            start,
            end,
            normal,
            shaft_end,
            ..
        } = Self::transition_arrow_endpoints(
            source_pos,
            target_pos,
            offset,
            total_transitions_for_state_pair,
        );
        let response = ui.allocate_rect(
            egui::Rect::from_two_pos(egui_pos2(start), egui_pos2(end)),
            egui::Sense::empty(),
        );
        let hovered = response.hover_pos().is_some_and(|pos| {
            let dist = distance_to_segment(bevy_vec2_from_pos2(pos), start, end);
            dist < 4.
        });
        let selected = self.selected_transitions.contains(&transition_id);

        if hovered && ui.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary)) {
            queue.trigger_window(ClearSelections {
                entity: Entity::PLACEHOLDER,
            });
            queue.trigger_window(SelectTransitions::placeholder([transition_id].into()));
            queue.trigger(SetActiveFsmTransition {
                new: ActiveFsmTransition {
                    handle: fsm.clone(),
                    transition: transition_id,
                },
            });
            queue.trigger(SetInspectorSelection {
                selection: InspectorSelection::ActiveFsmTransition,
            });
        }

        ui.painter().add(egui::epaint::PathShape {
            points: vec![
                egui_pos2(start + normal * 2.),
                egui_pos2(start - normal * 2.),
                egui_pos2(shaft_end - normal * 2.),
                egui_pos2(shaft_end + normal * 2.),
            ],
            closed: true,
            fill: self.style.transition_bg.get(hovered, selected),
            stroke: egui::epaint::PathStroke::new(0., egui::Color32::TRANSPARENT),
        });

        ui.painter().add(egui::epaint::PathShape {
            points: vec![
                egui_pos2(shaft_end - normal * 5.),
                egui_pos2(end),
                egui_pos2(shaft_end + normal * 5.),
            ],
            closed: true,
            fill: self.style.transition_bg.get(hovered, selected),
            stroke: egui::epaint::PathStroke::new(0., egui::Color32::TRANSPARENT),
        });

        // arrow outline
        ui.painter().add(egui::epaint::PathShape {
            points: vec![
                egui_pos2(start + normal * 2.),
                egui_pos2(start - normal * 2.),
                egui_pos2(shaft_end - normal * 2.),
                egui_pos2(shaft_end - normal * 5.),
                egui_pos2(end),
                egui_pos2(shaft_end + normal * 5.),
                egui_pos2(shaft_end + normal * 2.),
            ],
            closed: true,
            fill: egui::Color32::TRANSPARENT,
            stroke: path_stroke(
                self.style.transition_border.get(hovered, selected),
                egui::epaint::StrokeKind::Middle,
            ),
        });
    }
}

fn egui_pos2(vec2: Vec2) -> egui::Pos2 {
    egui::Pos2::new(vec2.x, vec2.y)
}

fn egui_vec2(vec2: Vec2) -> egui::Vec2 {
    egui::Vec2::new(vec2.x, vec2.y)
}

fn bevy_vec2_from_pos2(pos2: egui::Pos2) -> Vec2 {
    Vec2::new(pos2.x, pos2.y)
}

fn bevy_vec2_from_vec2(vec2: egui::Vec2) -> Vec2 {
    Vec2::new(vec2.x, vec2.y)
}

fn distance_to_segment(point: Vec2, segment_start: Vec2, segment_end: Vec2) -> f32 {
    let n = segment_end - segment_start;
    let p = point - segment_start;
    let proj = p.dot(n) / n.dot(n);
    if proj < 0. {
        point.distance(segment_start)
    } else if proj > 1. {
        point.distance(segment_end)
    } else {
        p.distance(n * proj)
    }
}

struct TransitionDrawData {
    start: Vec2,
    end: Vec2,
    normal: Vec2,
    dir: Vec2,
    shaft_end: Vec2,
}

#[derive(EntityEvent)]
struct ClearSelections {
    entity: Entity,
}

impl ClearSelections {
    pub fn observe(clear: On<ClearSelections>, mut state_query: Query<&mut FsmEditorWindowState>) {
        let Ok(mut state) = state_query.get_mut(clear.entity) else {
            return;
        };

        state.selected_states.clear();
        state.selected_transitions.clear();
    }
}

#[derive(EntityEvent)]
struct SelectStates {
    entity: Entity,
    states: HashSet<StateId>,
}

impl SelectStates {
    pub fn placeholder(states: HashSet<StateId>) -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
            states,
        }
    }

    pub fn observe(clear: On<SelectStates>, mut state_query: Query<&mut FsmEditorWindowState>) {
        let Ok(mut state) = state_query.get_mut(clear.entity) else {
            return;
        };

        state.selected_states.extend(&clear.states);
    }
}

#[derive(EntityEvent)]
struct SelectTransitions {
    entity: Entity,
    transitions: HashSet<TransitionId>,
}

impl SelectTransitions {
    pub fn placeholder(transitions: HashSet<TransitionId>) -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
            transitions,
        }
    }

    pub fn observe(
        clear: On<SelectTransitions>,
        mut state_query: Query<&mut FsmEditorWindowState>,
    ) {
        let Ok(mut state) = state_query.get_mut(clear.entity) else {
            return;
        };

        state.selected_transitions.extend(&clear.transitions);
    }
}

#[derive(Clone)]
pub struct StyleProp<T> {
    base: T,
    hovered: Option<T>,
    selected: Option<T>,
    hovered_and_selected: Option<T>,
}

impl<T: Clone> StyleProp<T> {
    pub fn get(&self, hovered: bool, selected: bool) -> T {
        if hovered && selected {
            self.hovered_and_selected
                .as_ref()
                .or(self.selected.as_ref())
                .or(self.hovered.as_ref())
                .unwrap_or(&self.base)
                .clone()
        } else if hovered {
            self.hovered.as_ref().unwrap_or(&self.base).clone()
        } else if selected {
            self.selected.as_ref().unwrap_or(&self.base).clone()
        } else {
            self.base.clone()
        }
    }
}

#[derive(Clone)]
pub struct FsmEditorStyle {
    pub bg: egui::Color32,
    pub bg_grid: egui::Color32,

    pub selection_box_fill: egui::Color32,
    pub selection_box_border: egui::Color32,

    pub state_bg: StyleProp<egui::Color32>,
    pub state_border: StyleProp<egui::Stroke>,
    pub transition_bg: StyleProp<egui::Color32>,
    pub transition_border: StyleProp<egui::Stroke>,
}

impl Default for FsmEditorStyle {
    fn default() -> Self {
        let state_bg = rgb(50, 50, 50);
        let state_hovered_bg = rgb(75, 75, 75);
        let state_border = egui::Stroke {
            width: 1.,
            color: rgb(89, 89, 89),
        };
        let state_selected_border = egui::Stroke {
            width: 4.,
            color: rgb(100, 100, 200),
        };

        let transition_bg = rgba(160, 160, 160, 128);
        let transition_hovered_bg = rgba(160, 160, 160, 255);

        let transition_border = egui::Stroke {
            width: 0.,
            color: rgb(89, 89, 89),
        };
        let transition_selected_border = egui::Stroke {
            width: 2.,
            color: rgb(100, 100, 200),
        };

        Self {
            bg: rgb(38, 38, 46),
            bg_grid: rgba(89, 89, 89, 128),

            selection_box_fill: rgba(40, 70, 200, 90),
            selection_box_border: rgb(40, 70, 200),

            state_bg: StyleProp {
                base: state_bg,
                hovered: Some(state_hovered_bg),
                selected: None,
                hovered_and_selected: None,
            },

            state_border: StyleProp {
                base: state_border,
                hovered: None,
                selected: Some(state_selected_border),
                hovered_and_selected: None,
            },

            transition_bg: StyleProp {
                base: transition_bg,
                hovered: Some(transition_hovered_bg),
                selected: None,
                hovered_and_selected: None,
            },

            transition_border: StyleProp {
                base: transition_border,
                hovered: None,
                selected: Some(transition_selected_border),
                hovered_and_selected: None,
            },
        }
    }
}

fn rgb(r: u8, g: u8, b: u8) -> egui::Color32 {
    egui::Color32::from_rgb(r, g, b)
}

fn rgba(r: u8, g: u8, b: u8, a: u8) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(r, g, b, a)
}

fn path_stroke(stroke: egui::Stroke, kind: egui::epaint::StrokeKind) -> egui::epaint::PathStroke {
    egui::epaint::PathStroke {
        width: stroke.width,
        color: egui::epaint::ColorMode::Solid(stroke.color),
        kind,
    }
}
