use std::{borrow::Cow, cmp::Ordering, collections::VecDeque};

use bevy::{
    asset::{Asset, Handle},
    log::warn,
    platform::collections::HashMap,
    reflect::Reflect,
};
use serde::{Deserialize, Serialize};

use super::high_level;
use crate::{
    animation_graph::{AnimationGraph, GraphInputPin, PinId, SourcePin, TargetPin, TimeUpdate},
    context::{
        io_env::{GraphIoEnv, IoOverrides, LayeredIoEnv},
        new_context::{GraphContext, NodeContext},
        spec_context::NodeSpec,
    },
    duration_data::DurationData,
    edge_data::{
        DataValue,
        events::{AnimationEvent, EventQueue},
    },
    errors::GraphError,
};

#[derive(Reflect, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LowLevelTransitionId {
    Start(high_level::TransitionId),
    End(high_level::TransitionId),
}

#[derive(Reflect, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LowLevelStateId {
    HlState(high_level::StateId),
    DirectTransition(high_level::TransitionId),
    GlobalTransition(
        /// source
        high_level::StateId,
        /// target (state with global transition enabled)
        high_level::StateId,
    ),
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LowLevelTransitionType {
    Direct,
    Global,
    Fallback,
}

impl PartialOrd for LowLevelTransitionType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LowLevelTransitionType {
    fn cmp(&self, other: &Self) -> Ordering {
        use LowLevelTransitionType::*;
        match (self, other) {
            (Direct, Direct) => Ordering::Equal,
            (Direct, Global) => Ordering::Less,
            (Direct, Fallback) => Ordering::Less,
            (Global, Direct) => Ordering::Greater,
            (Global, Global) => Ordering::Equal,
            (Global, Fallback) => Ordering::Less,
            (Fallback, Direct) => Ordering::Greater,
            (Fallback, Global) => Ordering::Greater,
            (Fallback, Fallback) => Ordering::Equal,
        }
    }
}

#[derive(Reflect, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FsmBuiltinPin {
    PercentThroughDuration,
    TimeElapsed,
}

/// Stateful data associated with an FSM node
#[derive(Reflect, Debug, Clone)]
pub struct FSMState {
    pub state: LowLevelStateId,
    pub state_entered_time: f32,
}

#[derive(Reflect, Debug, Clone)]
pub struct TransitionData {
    pub source: high_level::StateId,
    pub target: high_level::StateId,
    pub hl_transition_id: high_level::TransitionId,
    pub duration: f32,
}

/// Specification of a state node in the low-level FSM
#[derive(Reflect, Debug, Clone)]
pub struct LowLevelState {
    pub id: LowLevelStateId,
    pub graph: Handle<AnimationGraph>,
    pub hl_transition: Option<TransitionData>,
}

/// Specification of a transition in the low-level FSM
#[derive(Reflect, Debug, Clone)]
pub struct LowLevelTransition {
    pub id: LowLevelTransitionId,
    pub source: LowLevelStateId,
    pub target: LowLevelStateId,
    pub transition_type: LowLevelTransitionType,
    pub hl_source: high_level::StateId,
    pub hl_target: high_level::StateId,
}

/// It's a state machine in the mathematical sense (-ish). Transitions are immediate.
#[derive(Asset, Reflect, Debug, Clone, Default)]
pub struct LowLevelStateMachine {
    pub states: HashMap<LowLevelStateId, LowLevelState>,

    pub transitions: HashMap<LowLevelTransitionId, LowLevelTransition>,
    pub transitions_by_hl_state_pair:
        HashMap<(high_level::StateId, high_level::StateId), Vec<LowLevelTransitionId>>,

    pub start_state: Option<LowLevelStateId>,
    pub node_spec: NodeSpec,
}

impl LowLevelStateMachine {
    pub const DRIVER_EVENT_QUEUE: &'static str = "driver events";
    pub const DRIVER_TIME: &'static str = "driver time";

    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            transitions: HashMap::new(),
            transitions_by_hl_state_pair: HashMap::new(),
            start_state: None,
            node_spec: NodeSpec::default(),
        }
    }

    pub fn add_state(&mut self, state: LowLevelState) {
        self.states.insert(state.id.clone(), state);
    }

    pub fn add_transition(&mut self, transition: LowLevelTransition) {
        self.transitions
            .insert(transition.id.clone(), transition.clone());
        if matches!(transition.id, LowLevelTransitionId::Start(_)) {
            let vec = self
                .transitions_by_hl_state_pair
                .entry((transition.hl_source.clone(), transition.hl_target.clone()))
                .or_default();
            vec.push(transition.id.clone());
            // Direct transitions should come first
            vec.sort_by_key(|id| self.transitions.get(id).unwrap().transition_type);
        }
    }

    fn handle_event_queue(
        &self,
        event_queue: EventQueue,
        mut ctx: NodeContext,
    ) -> Result<(), GraphError> {
        let time = ctx.time();
        let fsm_state = ctx.state_mut_or_else(|| FSMState {
            state: self.start_state.clone().unwrap(),
            state_entered_time: time,
        })?;

        for event in event_queue.events {
            match event.event {
                AnimationEvent::TransitionToState(hl_target_state_id) => {
                    if let LowLevelStateId::HlState(hl_curr_state_id) = fsm_state.state.clone()
                        && let Some(transition) = self
                            .transitions_by_hl_state_pair
                            .get(&(hl_curr_state_id, hl_target_state_id))
                            .and_then(|ids| ids.iter().next())
                            .and_then(|id| self.transitions.get(id))
                    {
                        *fsm_state = FSMState {
                            state: transition.target.clone(),
                            state_entered_time: time,
                        };
                    }
                }
                AnimationEvent::Transition(transition_id) => {
                    if let Some(transition) = self
                        .transitions
                        .get(&LowLevelTransitionId::Start(transition_id))
                        && fsm_state.state == transition.source
                    {
                        *fsm_state = FSMState {
                            state: transition.target.clone(),
                            state_entered_time: time,
                        };
                    }
                }
                AnimationEvent::EndTransition => {
                    if let Some(hl_transition_data) = self
                        .states
                        .get(&fsm_state.state)
                        .and_then(|s| s.hl_transition.as_ref())
                        && let Some(transition) = self.transitions.get(&LowLevelTransitionId::End(
                            hl_transition_data.hl_transition_id.clone(),
                        ))
                    {
                        *fsm_state = FSMState {
                            state: transition.target.clone(),
                            state_entered_time: time,
                        };
                    }
                }
                AnimationEvent::StringId(_) => {}
            }
        }

        Ok(())
    }

    /// Performs a node update
    pub fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let input = ctx.time_update_fwd()?;

        let prev_time = ctx.prev_time();
        let pred_time = input.partial_update_basic(prev_time).unwrap_or_else(|| {
            warn!(
                "State machine node received unsupported time update: {:?}",
                input
            );
            prev_time
        });
        ctx.set_time(pred_time);

        ctx.set_time_update_back(Self::DRIVER_TIME, input);
        let event_queue = ctx
            .data_back(Self::DRIVER_EVENT_QUEUE)?
            .into_event_queue()
            .unwrap();

        self.handle_event_queue(event_queue, ctx.clone())?;
        let inner_eq = self.update_graph(ctx.clone())?;
        self.handle_event_queue(inner_eq, ctx)?;

        Ok(())
    }

    /// Updates underlying animation graphs for active states.
    pub fn update_graph(&self, mut ctx: NodeContext) -> Result<EventQueue, GraphError> {
        let time = ctx.time();
        let fsm_state = ctx.state::<FSMState>()?;
        // TODO: Replace panic with `GraphError`
        let state = self.states.get(&fsm_state.state).unwrap();
        let graph = ctx
            .graph_context
            .resources
            .animation_graph_assets
            .get(&state.graph)
            .unwrap();

        let mut io_overrides = IoOverrides::default();

        let elapsed_time = time - fsm_state.state_entered_time;
        let percent_through_duration = state
            .hl_transition
            .as_ref()
            .map(|t| elapsed_time / t.duration)
            .unwrap_or(0.);

        io_overrides.data.insert(
            FsmBuiltinPin::PercentThroughDuration.into(),
            percent_through_duration.into(),
        );

        let sub_io_env = LayeredIoEnv(
            Cow::<IoOverrides>::Owned(io_overrides),
            Cow::<FsmIoEnv>::Owned(FsmIoEnv {
                node_context: ctx.clone(),
                state_machine: self,
                current_state: state.id.clone(),
                current_state_role: StateRole::Root,
                state_stack: [].into(),
            }),
        );

        let sub_ctx = ctx
            .create_child_context(state.graph.id(), Some(state.id.clone()))
            .with_io(&sub_io_env);

        let mut driver_event_queue = EventQueue::default();

        for (id, _) in graph.node_spec.iter_output_data() {
            let target_pin = TargetPin::OutputData(id.clone());
            let value = graph.get_data(target_pin, sub_ctx.clone())?;
            if id == Self::DRIVER_EVENT_QUEUE {
                driver_event_queue = value.into_event_queue().unwrap();
            } else {
                ctx.set_data_fwd(id, value);
            }
        }

        Ok(driver_event_queue)
    }

    fn get_source(&self, state: &LowLevelStateId) -> Result<LowLevelStateId, GraphError> {
        self.states
            .get(state)
            .and_then(|s| s.hl_transition.as_ref())
            .map(|t| LowLevelStateId::HlState(t.source.clone()))
            .ok_or_else(|| GraphError::FSMCurrentStateMissing)
    }

    fn get_target(&self, state: &LowLevelStateId) -> Result<LowLevelStateId, GraphError> {
        self.states
            .get(state)
            .and_then(|s| s.hl_transition.as_ref())
            .map(|t| LowLevelStateId::HlState(t.target.clone()))
            .ok_or_else(|| GraphError::FSMCurrentStateMissing)
    }
}

#[derive(Clone)]
pub struct FsmIoEnv<'a> {
    /// The context at the FSM node
    node_context: NodeContext<'a>,
    state_machine: &'a LowLevelStateMachine,

    current_state: LowLevelStateId,
    current_state_role: StateRole,
    state_stack: VecDeque<(LowLevelStateId, StateRole)>,
}

impl<'a> FsmIoEnv<'a> {
    const MISSING_STATE_ERROR_MESSAGE: &'static str =
        "Queried state does not exist. This is an internal error, please submit a GitHub issue.";
    fn parent_graph_data_back(
        &self,
        pin_id: &PinId,
        ctx: &GraphContext,
    ) -> Result<DataValue, GraphError> {
        self.node_context
            .clone()
            .with_state_key(ctx.state_key)
            .data_back(pin_id)
    }

    fn state_data_back(
        &self,
        forward_pin_id: PinId,
        ctx: &GraphContext,
        next_state: LowLevelStateId,
        next_state_role: StateRole,
    ) -> Result<DataValue, GraphError> {
        let graph_handle = &self
            .state_machine
            .states
            .get(&next_state)
            .expect(Self::MISSING_STATE_ERROR_MESSAGE)
            .graph;

        let graph = self
            .node_context
            .graph_context
            .resources
            .animation_graph_assets
            .get(graph_handle)
            .ok_or(GraphError::FSMGraphAssetMissing)?;

        let mut next_state_stack = self.state_stack.clone();
        next_state_stack.push_back((self.current_state.clone(), self.current_state_role));

        let sub_graph_io = FsmIoEnv {
            node_context: self.node_context.clone(),
            state_machine: self.state_machine,
            current_state: next_state.clone(),
            state_stack: next_state_stack,
            current_state_role: next_state_role,
        };

        let sub_ctx = self
            .node_context
            .create_child_context(graph_handle.id(), Some(next_state))
            .with_state_key(ctx.state_key)
            .with_io(&sub_graph_io);

        graph.get_data(TargetPin::OutputData(forward_pin_id), sub_ctx)
    }

    fn parent_graph_duration_back(
        &self,
        pin_id: &PinId,
        ctx: &GraphContext,
    ) -> Result<DurationData, GraphError> {
        self.node_context
            .clone()
            .with_state_key(ctx.state_key)
            .duration_back(pin_id)
    }

    fn state_duration_back(
        &self,
        ctx: &GraphContext,
        state: LowLevelStateId,
        next_state_role: StateRole,
    ) -> Result<DurationData, GraphError> {
        let graph_handle = &self
            .state_machine
            .states
            .get(&state)
            .expect(Self::MISSING_STATE_ERROR_MESSAGE)
            .graph;

        let graph = self
            .node_context
            .graph_context
            .resources
            .animation_graph_assets
            .get(graph_handle)
            .ok_or(GraphError::FSMGraphAssetMissing)?;

        let mut next_state_stack = self.state_stack.clone();
        next_state_stack.push_back((self.current_state.clone(), self.current_state_role));

        let sub_graph_io = FsmIoEnv {
            node_context: self.node_context.clone(),
            state_machine: self.state_machine,
            current_state: state.clone(),
            state_stack: next_state_stack,
            current_state_role: next_state_role,
        };

        let sub_ctx = self
            .node_context
            .create_child_context(graph_handle.id(), Some(state))
            .with_state_key(ctx.state_key)
            .with_io(&sub_graph_io);

        graph.get_duration(TargetPin::OutputTime, sub_ctx)
    }

    fn parent_graph_time_update_fwd(&self, ctx: &GraphContext) -> Result<TimeUpdate, GraphError> {
        self.node_context
            .clone()
            .with_state_key(ctx.state_key)
            .time_update_fwd()
    }

    fn state_time_update_fwd(&self, ctx: &GraphContext) -> Result<TimeUpdate, GraphError> {
        let mut next_state_stack = self.state_stack.clone();
        if let Some((next_state, next_state_role)) = next_state_stack.pop_back() {
            let graph_handle = &self
                .state_machine
                .states
                .get(&next_state)
                .expect(Self::MISSING_STATE_ERROR_MESSAGE)
                .graph;

            let graph = self
                .node_context
                .graph_context
                .resources
                .animation_graph_assets
                .get(graph_handle)
                .ok_or(GraphError::FSMGraphAssetMissing)?;

            let sub_graph_io = FsmIoEnv {
                node_context: self.node_context.clone(),
                state_machine: self.state_machine,
                current_state: next_state.clone(),
                state_stack: next_state_stack,
                current_state_role: next_state_role,
            };

            let sub_ctx = self
                .node_context
                .create_child_context(graph_handle.id(), Some(next_state))
                .with_state_key(ctx.state_key)
                .with_io(&sub_graph_io);

            graph.get_time_update(
                SourcePin::InputTime(match self.current_state_role {
                    StateRole::Source => GraphInputPin::FromFsmSource("".into()),
                    StateRole::Target => GraphInputPin::FromFsmTarget("".into()),
                    StateRole::Root => unreachable!(),
                }),
                sub_ctx,
            )
        } else {
            Err(GraphError::FSMRequestedMissingData)
        }
    }
}

impl<'a> GraphIoEnv for FsmIoEnv<'a> {
    fn get_data_back(
        &self,
        graph_input_pin: GraphInputPin,
        ctx: GraphContext,
    ) -> Result<DataValue, GraphError> {
        match graph_input_pin {
            GraphInputPin::Default(pin_id) => self.parent_graph_data_back(&pin_id, &ctx),
            GraphInputPin::FromFsmSource(pin_id) => self
                .state_machine
                .get_source(&self.current_state)
                .and_then(|state| self.state_data_back(pin_id, &ctx, state, StateRole::Source)),
            GraphInputPin::FromFsmTarget(pin_id) => self
                .state_machine
                .get_source(&self.current_state)
                .and_then(|state| self.state_data_back(pin_id, &ctx, state, StateRole::Target)),
            // This will get handled by the next IO layer in update_graph (defaults and overrides)
            GraphInputPin::FsmBuiltin(_) => Err(GraphError::FSMRequestedMissingData),
        }
    }

    // TODO: We can forward duration queries to the parent graph, but there will never be anything
    // connected there! We need to allow arbitrary time inputs to the FSM as well
    fn get_duration_back(
        &self,
        graph_input_pin: GraphInputPin,
        ctx: GraphContext,
    ) -> Result<DurationData, GraphError> {
        match graph_input_pin {
            GraphInputPin::Default(pin_id) => self.parent_graph_duration_back(&pin_id, &ctx),
            GraphInputPin::FromFsmSource(_) => self
                .state_machine
                .get_source(&self.current_state)
                .and_then(|state| self.state_duration_back(&ctx, state, StateRole::Source)),
            GraphInputPin::FromFsmTarget(_) => self
                .state_machine
                .get_target(&self.current_state)
                .and_then(|state| self.state_duration_back(&ctx, state, StateRole::Target)),
            // This will get handled by the next IO layer in update_graph (defaults and overrides)
            GraphInputPin::FsmBuiltin(_) => Err(GraphError::FSMRequestedMissingData),
        }
    }

    fn get_time_fwd(&self, ctx: GraphContext) -> Result<TimeUpdate, GraphError> {
        self.state_time_update_fwd(&ctx)
            .or_else(|_| self.parent_graph_time_update_fwd(&ctx))
    }
}

#[derive(Clone, Copy)]
pub enum StateRole {
    Source,
    Target,
    Root,
}
