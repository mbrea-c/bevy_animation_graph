use core::f32;
use std::cmp::Ordering;

use bevy::{
    asset::{Assets, Handle},
    ecs::{component::Component, entity::Entity, event::EntityEvent, observer::On, system::Query},
    math::Vec2,
    platform::collections::{HashMap, HashSet},
    prelude::World,
};
use bevy_animation_graph::core::state_machine::{
    high_level::{
        DirectTransition, DirectTransitionId, State, StateId, StateMachine, TransitionId,
    },
    low_level::{FsmState, LowLevelStateId},
};
use egui_dock::egui;
use uuid::Uuid;

use crate::ui::{
    generic_widgets::fsm::{direct_transition::DirectTransitionWidget, state::StateWidget},
    native_windows::{
        EditorWindowContext, EditorWindowRegistrationContext, NativeEditorWindowExtension,
        OwnedQueue,
    },
    state_management::global::{
        RegisterStateComponent,
        active_fsm::ActiveFsm,
        active_fsm_state::{ActiveFsmState, SetActiveFsmState},
        active_fsm_transition::{ActiveFsmTransition, SetActiveFsmTransition},
        active_graph_context::{ActiveContexts, ensure_active_context_selected_if_available},
        fsm::{
            CreateDirectTransition, CreateState, DeleteDirectTransitions, DeleteStates, MoveStates,
        },
        get_global_state,
        inspector_selection::{InspectorSelection, SetInspectorSelection},
        register_if_missing,
    },
    style::{StyleEngine, StyleModifiers, StyleObject, StyleRule, path_stroke, rgb, rgba},
    utils::{self, popup::CustomPopup},
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

        ctx.queue_context(|_, queue| {
            ensure_active_context_selected_if_available(
                world,
                active_fsm.handle.id().untyped(),
                |st| {
                    utils::find_fsm_node_in_graph(world, st.get_graph_id(), active_fsm.handle.id())
                        .is_some()
                },
                queue,
            );
        });

        let maybe_fsm_state = get_global_state::<ActiveContexts>(world)
            .and_then(|s| s.by_asset.get(&active_fsm.handle.id().untyped()))
            .and_then(|(entity, id)| {
                Some((
                    id,
                    utils::get_specific_animation_graph_player(world, *entity)?,
                ))
            })
            .and_then(|(id, p)| Some(id).zip(p.get_context_arena()))
            .and_then(|(id, ca)| ca.get_context(*id))
            .and_then(|st| {
                let fsm_node_id = utils::find_fsm_node_in_graph(
                    world,
                    st.get_graph_id(),
                    active_fsm.handle.id(),
                )?;

                st.node_states
                    .get_all_upcoming_states::<FsmState>(fsm_node_id)
                    .ok()
            })
            .and_then(|mut iter| iter.next())
            .cloned();

        let outer_rect = ui.available_rect_before_wrap();

        let mut queue = ctx.make_queue();
        let window = ctx.window_entity;

        let buffer_id = ui.id().with("Fsm editor nodes context buffer");
        let buffer = ctx.buffers.get_mut_or_default(buffer_id);

        world.resource_scope::<Assets<StateMachine>, _>(|_, fsm_assets| {
            let fsm = fsm_assets.get(&active_fsm.handle)?;

            window_state.draw_fsm(
                ui,
                fsm,
                buffer,
                &mut queue,
                window,
                &active_fsm.handle,
                maybe_fsm_state.as_ref(),
            );

            Some(())
        });

        ctx.consume_queue(queue);

        // State deleteions
        if ui.input(|i| {
            i.pointer
                .hover_pos()
                .is_some_and(|p| outer_rect.contains(p))
                && (i.key_pressed(egui::Key::Backspace) || i.key_pressed(egui::Key::Delete))
        }) {
            ctx.trigger(DeleteStates {
                fsm: active_fsm.handle.clone(),
                states: window_state.selected_states.clone(),
            });

            ctx.trigger(DeleteDirectTransitions {
                fsm: active_fsm.handle.clone(),
                transitions: window_state.selected_transitions.clone(),
            });
        }

        CustomPopup::new()
            .with_salt(ui.id().with("Graph editor right click popup"))
            .with_sense_rect(outer_rect)
            .with_allow_opening(true)
            .with_save_on_click(Some(()))
            .with_default_size(egui::Vec2::new(500., 300.))
            .show_if_saved(ui, |ui, ()| {
                creation_popup(ui, world, ctx);
            });
    }

    fn display_name(&self) -> String {
        "FSM Editor".to_string()
    }
}

#[derive(Component, Clone)]
pub struct FsmEditorWindowState {
    pub selected_states: HashSet<StateId>,
    pub selected_transitions: HashSet<DirectTransitionId>,
    pub style_engine: StyleEngine,
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

impl FsmEditorWindowState {
    #[allow(clippy::too_many_arguments)]
    fn draw_fsm(
        &self,
        ui: &mut egui::Ui,
        fsm: &StateMachine,
        buffer: &mut FsmEditBuffer,
        queue: &mut OwnedQueue,
        window: Entity,
        fsm_handle: &Handle<StateMachine>,
        maybe_state: Option<&FsmState>,
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

                self.draw_direct_transitions(ui, fsm, queue, fsm_handle, maybe_state);
                self.draw_states(ui, fsm, queue, fsm_handle, maybe_state);
            })
            .response
    }

    fn draw_editor_background(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        let style = self
            .style_engine
            .evaluate::<EditorGridStyle>(StyleModifiers::empty(), &HashSet::new());

        let separation_int = 40;
        let separation = separation_int as f32;

        ui.painter()
            .rect_filled(rect, 0., style.bg.unwrap_or_default());

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
                    color: style.grid.unwrap_or_default(),
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
                    color: style.grid.unwrap_or_default(),
                },
            );
        }
    }

    fn draw_selection_box(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        let style = self
            .style_engine
            .evaluate::<SelectionBoxStyle>(StyleModifiers::empty(), &HashSet::new());

        ui.painter().rect(
            rect,
            0.,
            style.fill.unwrap_or_default(),
            egui::Stroke {
                width: 1.,
                color: style.border.unwrap_or_default(),
            },
            egui::StrokeKind::Middle,
        );
    }

    fn compute_updated_selections(
        &self,
        fsm: &StateMachine,
        rect: egui::Rect,
    ) -> (HashSet<StateId>, HashSet<DirectTransitionId>) {
        let mut selected_states = HashSet::new();

        for state in fsm.states.values() {
            let pos = fsm
                .editor_metadata
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
                let source_pos = fsm
                    .editor_metadata
                    .states
                    .get(&transition.source)
                    .copied()
                    .unwrap_or(Vec2::ZERO);
                let target_pos = fsm
                    .editor_metadata
                    .states
                    .get(&transition.target)
                    .copied()
                    .unwrap_or(Vec2::ZERO);
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
        maybe_state: Option<&FsmState>,
    ) {
        for state in fsm.states.values() {
            let pos = fsm
                .editor_metadata
                .states
                .get(&state.id)
                .copied()
                .unwrap_or(Vec2::ZERO);
            self.draw_state(
                ui,
                state,
                pos,
                fsm.start_state == state.id,
                queue,
                fsm_handle,
                maybe_state,
            );
        }
    }

    fn collapse_direct_transitions(
        fsm: &StateMachine,
    ) -> HashMap<(StateId, StateId), Vec<DirectTransitionId>> {
        let mut direct_transitions: HashMap<(StateId, StateId), Vec<DirectTransitionId>> =
            HashMap::new();
        for transition in fsm.transitions.values() {
            let key = match transition.source.cmp(&transition.target) {
                Ordering::Less => (transition.source, transition.target),
                _ => (transition.target, transition.source),
            };
            direct_transitions
                .entry(key)
                .or_insert(vec![])
                .push(transition.id);
        }

        direct_transitions
    }

    fn draw_direct_transitions(
        &self,
        ui: &mut egui::Ui,
        fsm: &StateMachine,
        queue: &mut OwnedQueue,
        fsm_handle: &Handle<StateMachine>,
        maybe_state: Option<&FsmState>,
    ) {
        let direct_transitions = Self::collapse_direct_transitions(fsm);
        for transitions in direct_transitions.values() {
            for (i, transition_id) in transitions.iter().enumerate() {
                let Some(transition) = fsm.transitions.get(transition_id) else {
                    continue;
                };
                let source_pos = fsm
                    .editor_metadata
                    .states
                    .get(&transition.source)
                    .copied()
                    .unwrap_or(Vec2::ZERO);
                let target_pos = fsm
                    .editor_metadata
                    .states
                    .get(&transition.target)
                    .copied()
                    .unwrap_or(Vec2::ZERO);
                self.draw_direct_transition(
                    ui,
                    *transition_id,
                    source_pos,
                    target_pos,
                    i,
                    transitions.len(),
                    queue,
                    fsm_handle,
                    maybe_state,
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

    #[allow(clippy::too_many_arguments)]
    fn draw_state(
        &self,
        ui: &mut egui::Ui,
        state: &State,
        pos: Vec2,
        is_start_state: bool,
        queue: &mut OwnedQueue,
        fsm: &Handle<StateMachine>,
        maybe_state: Option<&FsmState>,
    ) -> egui::Response {
        let rect = Self::state_rect(pos);

        let state_response = ui.allocate_rect(rect, egui::Sense::click_and_drag());

        if state_response.clicked() {
            queue.trigger(ClearSelections {
                entity: queue.window_entity,
            });

            queue.trigger(SelectStates {
                entity: queue.window_entity,
                states: [state.id].into(),
            });
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
                queue.trigger(ClearSelections {
                    entity: queue.window_entity,
                });
                queue.trigger(SelectStates {
                    entity: queue.window_entity,
                    states: [state.id].into(),
                });
                queue.trigger(MoveStates {
                    fsm: fsm.clone(),
                    states: [state.id].into(),
                    delta: bevy_vec2_from_vec2(state_response.drag_delta()),
                });
            }
        }
        let mut modifiers = StyleModifiers::empty();
        let mut classes = HashSet::new();
        if state_response.hovered() {
            modifiers |= StyleModifiers::HOVERED;
        }
        if self.selected_states.contains(&state.id) {
            modifiers |= StyleModifiers::SELECTED;
        }
        if let Some(stateful_data) = maybe_state
            && let LowLevelStateId::HlState(active_state_id) = &stateful_data.state
            && *active_state_id == state.id
        {
            classes.insert("active".into());
        }

        let style = self
            .style_engine
            .evaluate::<FsmStateStyle>(modifiers, &classes);

        ui.painter().rect(
            rect,
            10.,
            style.bg.unwrap_or_default(),
            style.border.unwrap_or_default(),
            egui::StrokeKind::Outside,
        );
        if is_start_state {
            ui.painter().rect(
                rect.shrink(5.),
                10.,
                egui::Color32::TRANSPARENT,
                style.border.unwrap_or_default(),
                egui::StrokeKind::Outside,
            );
        }

        if state.state_transition.is_some() {
            ui.painter().text(
                egui_pos2(
                    pos - Vec2::new(Self::STATE_WIDTH / 2. - 10., Self::STATE_HEIGHT / 2. - 10.),
                ),
                egui::Align2::CENTER_CENTER,
                "⚡",
                egui::FontId::proportional(18.),
                egui::Color32::YELLOW,
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

    #[allow(clippy::too_many_arguments)]
    fn draw_direct_transition(
        &self,
        ui: &mut egui::Ui,
        transition_id: DirectTransitionId,
        source_pos: Vec2,
        target_pos: Vec2,
        offset: usize,
        total_transitions_for_state_pair: usize,
        queue: &mut OwnedQueue,
        fsm: &Handle<StateMachine>,
        maybe_state: Option<&FsmState>,
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
            queue.trigger(ClearSelections {
                entity: queue.window_entity,
            });
            queue.trigger(SelectTransitions {
                entity: queue.window_entity,
                transitions: [transition_id].into(),
            });
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

        let mut modifiers = StyleModifiers::empty();
        let mut classes = HashSet::new();

        if hovered {
            modifiers |= StyleModifiers::HOVERED;
        }
        if selected {
            modifiers |= StyleModifiers::SELECTED;
        }

        if let Some(LowLevelStateId::HlTransition(TransitionId::Direct(
            active_direct_transition_id,
        ))) = maybe_state.map(|s| &s.state)
            && *active_direct_transition_id == transition_id
        {
            classes.insert("active".into());
        }

        let style = self
            .style_engine
            .evaluate::<FsmTransitionStyle>(modifiers, &classes);

        ui.painter().add(egui::epaint::PathShape {
            points: vec![
                egui_pos2(start + normal * 2.),
                egui_pos2(start - normal * 2.),
                egui_pos2(shaft_end - normal * 2.),
                egui_pos2(shaft_end + normal * 2.),
            ],
            closed: true,
            fill: style.bg.unwrap_or_default(),
            stroke: egui::epaint::PathStroke::new(0., egui::Color32::TRANSPARENT),
        });

        ui.painter().add(egui::epaint::PathShape {
            points: vec![
                egui_pos2(shaft_end - normal * 5.),
                egui_pos2(end),
                egui_pos2(shaft_end + normal * 5.),
            ],
            closed: true,
            fill: style.bg.unwrap_or_default(),
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
                style.border.unwrap_or_default(),
                egui::epaint::StrokeKind::Middle,
            ),
        });
    }
}

fn creation_popup(ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
    let toggle_mem_id = ui.id().with("state or transition crate mode");
    let mut create_transition: bool =
        ui.memory_mut(|mem| *mem.data.get_temp_mut_or_default(toggle_mem_id));
    let button_text = if create_transition {
        "Switch to state creation"
    } else {
        "Switch to transition creation"
    };
    if ui.button(button_text).clicked() {
        create_transition = !create_transition;
    }
    egui::Frame::new().outer_margin(3).show(ui, |ui| {
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                if create_transition {
                    transition_creation(ui, world, ctx);
                } else {
                    state_creation(ui, world, ctx);
                }
            });
    });

    ui.memory_mut(|mem| mem.data.insert_temp(toggle_mem_id, create_transition));
}

fn state_creation(ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
    let buffer_id = ui.id().with("state creator popup");
    let buffer = ctx.buffers.get_mut_or_default::<State>(buffer_id);

    ui.add(StateWidget::new_salted(
        buffer,
        world,
        "create fsm state widget",
    ));

    let submit_response = ui
        .with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
            ui.button("Create state")
        })
        .inner;

    if submit_response.clicked()
        && let Some(active_fsm) = get_global_state::<ActiveFsm>(world)
    {
        let mut state = buffer.clone();
        state.id = StateId::from(Uuid::new_v4());
        ctx.trigger(CreateState {
            fsm: active_fsm.handle.clone(),
            state,
        });
    }
}

fn transition_creation(ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
    world.resource_scope::<Assets<StateMachine>, _>(|world, fsm_assets| {
        let Some(active_fsm) = get_global_state::<ActiveFsm>(world) else {
            return;
        };
        let buffer_id = ui.id().with("transition creator popup");
        let buffer = ctx
            .buffers
            .get_mut_or_default::<DirectTransition>(buffer_id);

        let fsm = fsm_assets.get(&active_fsm.handle);
        ui.add(
            DirectTransitionWidget::new(buffer, world)
                .salted("create fsm state widget")
                .with_fsm(fsm),
        );

        let submit_response = ui
            .with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                ui.button("Create state")
            })
            .inner;

        if submit_response.clicked()
            && let Some(active_fsm) = get_global_state::<ActiveFsm>(world)
        {
            let mut transition = buffer.clone();
            transition.id = DirectTransitionId::from(Uuid::new_v4());
            ctx.trigger(CreateDirectTransition {
                fsm: active_fsm.handle.clone(),
                transition,
            });
        }
    });
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
    transitions: HashSet<DirectTransitionId>,
}

impl SelectTransitions {
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

#[derive(Default, Clone)]
pub struct EditorGridStyle {
    pub bg: Option<egui::Color32>,
    pub grid: Option<egui::Color32>,
}

impl StyleObject for EditorGridStyle {
    fn merge(&self, other: &Self) -> Self {
        Self {
            bg: other.bg.or(self.bg),
            grid: other.grid.or(self.grid),
        }
    }

    fn base() -> Self {
        Self {
            bg: Some(rgb(38, 38, 46)),
            grid: Some(rgba(89, 89, 89, 128)),
        }
    }
}

#[derive(Default, Clone)]
pub struct SelectionBoxStyle {
    pub fill: Option<egui::Color32>,
    pub border: Option<egui::Color32>,
}

impl StyleObject for SelectionBoxStyle {
    fn merge(&self, other: &Self) -> Self {
        Self {
            fill: other.fill.or(self.fill),
            border: other.border.or(self.border),
        }
    }

    fn base() -> Self {
        Self {
            fill: Some(rgba(40, 70, 200, 90)),
            border: Some(rgb(40, 70, 200)),
        }
    }
}

#[derive(Default, Clone)]
pub struct FsmStateStyle {
    pub bg: Option<egui::Color32>,
    pub border: Option<egui::Stroke>,
}

impl StyleObject for FsmStateStyle {
    fn merge(&self, other: &Self) -> Self {
        Self {
            bg: other.bg.or(self.bg),
            border: other.border.or(self.border),
        }
    }

    fn base() -> Self {
        Self {
            bg: Some(rgb(50, 50, 50)),
            border: Some(egui::Stroke {
                width: 1.,
                color: rgb(89, 89, 89),
            }),
        }
    }
}

#[derive(Default, Clone)]
pub struct FsmTransitionStyle {
    pub bg: Option<egui::Color32>,
    pub border: Option<egui::Stroke>,
}

impl StyleObject for FsmTransitionStyle {
    fn merge(&self, other: &Self) -> Self {
        Self {
            bg: other.bg.or(self.bg),
            border: other.border.or(self.border),
        }
    }

    fn base() -> Self {
        Self {
            bg: Some(rgba(160, 160, 160, 128)),
            border: Some(egui::Stroke {
                width: 1.,
                color: rgb(89, 89, 89),
            }),
        }
    }
}

impl Default for FsmEditorWindowState {
    fn default() -> Self {
        let mut style_engine = StyleEngine::default();

        let active_bg = rgb(30, 110, 20);

        style_engine
            .add_rule(
                StyleRule::val(FsmStateStyle {
                    bg: Some(rgb(75, 75, 75)),
                    ..Default::default()
                })
                .with_modifiers(StyleModifiers::HOVERED),
            )
            .add_rule(
                StyleRule::val(FsmStateStyle {
                    border: Some(egui::Stroke {
                        width: 4.,
                        color: rgb(100, 100, 200),
                    }),
                    ..Default::default()
                })
                .with_modifiers(StyleModifiers::SELECTED),
            )
            .add_rule(
                StyleRule::val(FsmStateStyle {
                    bg: Some(active_bg),
                    ..Default::default()
                })
                .with_class("active"),
            );

        style_engine
            .add_rule(
                StyleRule::val(FsmTransitionStyle {
                    bg: Some(rgba(160, 160, 160, 255)),
                    ..Default::default()
                })
                .with_modifiers(StyleModifiers::HOVERED),
            )
            .add_rule(
                StyleRule::val(FsmTransitionStyle {
                    border: Some(egui::Stroke {
                        width: 2.,
                        color: rgb(100, 100, 200),
                    }),
                    ..Default::default()
                })
                .with_modifiers(StyleModifiers::SELECTED),
            )
            .add_rule(
                StyleRule::val(FsmTransitionStyle {
                    bg: Some(active_bg),
                    ..Default::default()
                })
                .with_class("active"),
            );

        Self {
            selected_states: Default::default(),
            selected_transitions: Default::default(),
            style_engine,
        }
    }
}
