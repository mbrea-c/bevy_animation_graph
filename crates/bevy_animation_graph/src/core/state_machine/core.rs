use super::{StateId, TransitionId};
use crate::{
    core::{
        animation_graph::{
            AnimationGraph, InputOverlay, PinMap, SourcePin, TargetPin, TimeUpdate,
            DEFAULT_OUTPUT_POSE,
        },
        context::{
            CacheReadFilter, CacheWriteFilter, FsmContext, PassContext, StateRole, StateStack,
        },
        duration_data::DurationData,
        edge_data::{DataValue, EventQueue},
        errors::GraphError,
    },
    utils::unwrap::UnwrapVal,
};
use bevy::{
    asset::{Asset, Handle},
    reflect::Reflect,
    utils::HashMap,
};

/// Stateful data associated with an FSM node
#[derive(Reflect, Debug, Default, Clone)]
pub struct FSMState {
    pub state: StateId,
    pub state_entered_time: f32,
}

#[derive(Reflect, Debug, Clone)]
pub struct TransitionData {
    pub source: StateId,
    pub target: StateId,
    pub hl_transition_id: TransitionId,
    pub duration: f32,
}

/// Specification of a state node in the low-level FSM
#[derive(Reflect, Debug, Clone)]
pub struct LowLevelState {
    pub id: StateId,
    pub graph: Handle<AnimationGraph>,
    pub transition: Option<TransitionData>,
}

/// It's a state machine in the mathematical sense (-ish). Transitions are immediate.
#[derive(Asset, Reflect, Debug, Clone, Default)]
pub struct LowLevelStateMachine {
    pub states: HashMap<StateId, LowLevelState>,
    pub transitions: HashMap<(StateId, TransitionId), StateId>,
    pub start_state: Option<StateId>,
    pub input_data: PinMap<DataValue>,
}

impl LowLevelStateMachine {
    pub const DRIVER_EVENT_QUEUE: &'static str = "driver events";
    pub const DRIVER_TIME: &'static str = "driver time";

    // -----------------------------------------------------
    // --- Reserved FSM input parameter names
    const SOURCE_POSE: &'static str = "source pose";
    const TARGET_POSE: &'static str = "target pose";
    const SOURCE_TIME: &'static str = "source time";
    const TARGET_TIME: &'static str = "target time";
    const PERCENT_THROUGH_DURATION: &'static str = "elapsed percent";
    // -----------------------------------------------------

    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            transitions: HashMap::new(),
            start_state: None,
            input_data: PinMap::new(),
        }
    }

    pub fn add_state(&mut self, state: LowLevelState) {
        self.states.insert(state.id.clone(), state);
    }

    pub fn add_transition(&mut self, from: StateId, transition: TransitionId, to: StateId) {
        self.transitions.insert((from, transition), to);
    }

    fn handle_event_queue(
        &self,
        fsm_state: Option<FSMState>,
        event_queue: EventQueue,
        mut ctx: PassContext,
    ) -> Result<FSMState, GraphError> {
        let time = ctx.time();
        let mut fsm_state = fsm_state.unwrap_or_else(|| {
            ctx.caches()
                .get(
                    |c| {
                        c.get_fsm_state(ctx.node_context.as_ref().unwrap().node_id)
                            .cloned()
                    },
                    CacheReadFilter::FULL,
                )
                .unwrap_or(FSMState {
                    state: self.start_state.clone().unwrap(),
                    state_entered_time: time,
                })
        });

        for event in event_queue.events {
            let transition = event.event.id;
            if let Some(state) = self.transitions.get(&(fsm_state.state.clone(), transition)) {
                fsm_state = FSMState {
                    state: state.clone(),
                    state_entered_time: time,
                };
            }
        }

        let is_temp = ctx.temp_cache;
        let node_id = ctx.node_context.as_ref().unwrap().node_id.clone();

        ctx.caches_mut().set(
            |c| c.set_fsm_state(node_id, fsm_state.clone()),
            CacheWriteFilter::for_temp(is_temp),
        );

        Ok(fsm_state)
    }

    pub fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let input = ctx.time_update_fwd()?;
        let prev_time = ctx.prev_time();
        let pred_time = input.apply(prev_time);

        ctx.set_time(pred_time);

        ctx.set_time_update_back(Self::DRIVER_TIME, input);
        let event_queue: EventQueue = ctx.data_back(Self::DRIVER_EVENT_QUEUE)?.val();
        let fsm_state = self.handle_event_queue(None, event_queue, ctx.clone())?;
        let inner_eq = self.update_graph(&fsm_state, ctx.clone())?;
        self.handle_event_queue(Some(fsm_state), inner_eq, ctx)?;

        Ok(())
    }

    pub fn update_graph(
        &self,
        fsm_state: &FSMState,
        mut ctx: PassContext,
    ) -> Result<EventQueue, GraphError> {
        // TODO: Replace panic with `GraphError`
        let state = self.states.get(&fsm_state.state).unwrap();
        let graph = ctx
            .resources
            .animation_graph_assets
            .get(&state.graph)
            .unwrap();

        let mut input_overlay = InputOverlay::default();

        let time = ctx.time();
        let elapsed_time = time - fsm_state.state_entered_time;
        let percent_through_duration = state
            .transition
            .as_ref()
            .map(|t| elapsed_time / t.duration)
            .unwrap_or(0.);

        input_overlay.parameters.insert(
            Self::PERCENT_THROUGH_DURATION.into(),
            percent_through_duration.into(),
        );

        let fsm_ctx = FsmContext {
            state_stack: StateStack {
                stack: vec![(fsm_state.state.clone(), StateRole::Root)],
            },
            fsm: self,
        };

        if graph.output_time.is_some() {
            let input = ctx.time_update_fwd();
            if let Ok(time_update) = input {
                let target_pin = TargetPin::OutputTime;

                let mut ctx = ctx.child_with_state(Some(fsm_ctx.clone()), &input_overlay);
                let is_temp = ctx.temp_cache;

                ctx.caches_mut().set(
                    |c| c.set_time_update_back(target_pin, time_update),
                    CacheWriteFilter::for_temp(is_temp),
                );
            }
        }

        let mut driver_event_queue = EventQueue::default();

        for id in graph.output_parameters.keys() {
            let target_pin = TargetPin::OutputData(id.clone());
            let value = graph.get_data(
                target_pin,
                ctx.child_with_state(Some(fsm_ctx.clone()), &input_overlay),
            )?;
            if id == Self::DRIVER_EVENT_QUEUE {
                driver_event_queue = value.val();
            } else {
                ctx.set_data_fwd(id, value);
            }
        }

        Ok(driver_event_queue)
    }

    pub fn get_data(
        &self,
        mut state_stack: StateStack,
        target_pin: TargetPin,
        mut ctx: PassContext,
    ) -> Result<DataValue, GraphError> {
        let state_id = state_stack.last_state();
        let state = self
            .states
            .get(&state_id)
            .ok_or(GraphError::FSMCurrentStateMissing)?;
        let out = match &target_pin {
            TargetPin::OutputData(s) => {
                if let Some(default) = self.input_data.get(s) {
                    ctx.data_back(s).or_else(|_| Ok(default.clone()))
                } else {
                    let (queried_state, queried_role) = if s == Self::SOURCE_POSE {
                        (
                            state
                                .transition
                                .as_ref()
                                .ok_or(GraphError::FSMExpectedTransitionFoundState)?
                                .source
                                .clone(),
                            StateRole::Source,
                        )
                    } else if s == Self::TARGET_POSE {
                        (
                            state
                                .transition
                                .as_ref()
                                .ok_or(GraphError::FSMExpectedTransitionFoundState)?
                                .target
                                .clone(),
                            StateRole::Target,
                        )
                    } else {
                        return Err(GraphError::FSMRequestedMissingData);
                    };

                    let queried_graph = &self
                        .states
                        .get(&queried_state)
                        .expect("Queried state does not exist. This is an internal error, please submit a GitHub issue.")
                        .graph;

                    let graph = ctx
                        .resources
                        .animation_graph_assets
                        .get(queried_graph)
                        .ok_or(GraphError::FSMGraphAssetMissing)?;

                    let target_pin = TargetPin::OutputData(DEFAULT_OUTPUT_POSE.to_string());

                    let i = InputOverlay::default();
                    state_stack.stack.push((queried_state, queried_role));
                    graph.get_data(
                        target_pin,
                        ctx.child_with_state(
                            Some(FsmContext {
                                state_stack,
                                fsm: self,
                            }),
                            &i,
                        ),
                    )
                }
            }
            _ => panic!("State machine received data query without `OutputData` target"),
        };

        out
    }
    pub fn get_time_update(
        &self,
        mut state_stack: StateStack,
        target_pin: TargetPin,
        ctx: PassContext,
    ) -> Result<TimeUpdate, GraphError> {
        match target_pin {
            TargetPin::OutputTime => match state_stack.last_role() {
                StateRole::Source => {
                    state_stack.stack.pop();
                    let state = self.states.get(&state_stack.last_state()).unwrap();
                    let graph = ctx
                        .resources
                        .animation_graph_assets
                        .get(&state.graph)
                        .unwrap();
                    let overlay = InputOverlay::default();
                    graph.get_time_update(
                        SourcePin::InputTime(Self::SOURCE_TIME.to_string()),
                        ctx.child_with_state(
                            Some(FsmContext {
                                state_stack,
                                fsm: self,
                            }),
                            &overlay,
                        ),
                    )
                }
                StateRole::Target => {
                    state_stack.stack.pop();
                    let state = self.states.get(&state_stack.last_state()).unwrap();
                    let graph = ctx
                        .resources
                        .animation_graph_assets
                        .get(&state.graph)
                        .unwrap();
                    let overlay = InputOverlay::default();
                    graph.get_time_update(
                        SourcePin::InputTime(Self::TARGET_TIME.to_string()),
                        ctx.child_with_state(
                            Some(FsmContext {
                                state_stack,
                                fsm: self,
                            }),
                            &overlay,
                        ),
                    )
                }
                StateRole::Root => ctx.time_update_fwd(),
            },
            _ => panic!("State machine received time query without `OutputTime` target"),
        }
    }

    pub fn get_duration(
        &self,
        mut state_stack: StateStack,
        source_pin: SourcePin,
        ctx: PassContext,
    ) -> Result<DurationData, GraphError> {
        let state_id = state_stack.last_state();
        let state = self
            .states
            .get(&state_id)
            .ok_or(GraphError::FSMCurrentStateMissing)?;

        let out = match &source_pin {
            SourcePin::InputTime(p) => {
                let (queried_state, queried_role) = if p == Self::SOURCE_TIME {
                    (
                        state
                            .transition
                            .as_ref()
                            .ok_or(GraphError::FSMExpectedTransitionFoundState)?
                            .source
                            .clone(),
                        StateRole::Source,
                    )
                } else if p == Self::TARGET_TIME {
                    (
                        state
                            .transition
                            .as_ref()
                            .ok_or(GraphError::FSMExpectedTransitionFoundState)?
                            .target
                            .clone(),
                        StateRole::Source,
                    )
                } else {
                    return Err(GraphError::FSMRequestedMissingData);
                };

                let queried_graph = &self
                    .states
                    .get(&queried_state)
                    .expect("Queried state does not exist. This is an internal error, please submit a GitHub issue.")
                    .graph;

                let graph = ctx
                    .resources
                    .animation_graph_assets
                    .get(queried_graph)
                    .ok_or(GraphError::FSMGraphAssetMissing)?;

                let i = InputOverlay::default();
                let target_pin = TargetPin::OutputTime;

                state_stack.stack.push((queried_state, queried_role));

                graph.get_duration(
                    target_pin,
                    ctx.child_with_state(
                        Some(FsmContext {
                            state_stack,
                            fsm: self,
                        }),
                        &i,
                    ),
                )
            }
            _ => panic!("State machine received data query without `OutputData` target"),
        };

        out
    }
}
