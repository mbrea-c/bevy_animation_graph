pub mod loader;
pub mod serial;

use bevy::{
    asset::{Asset, Handle, ReflectAsset},
    math::Vec2,
    platform::collections::HashMap,
    reflect::Reflect,
};
use bevy_animation_graph_proc_macros::UuidWrapper;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::low_level::{
    self, LowLevelState, LowLevelStateId, LowLevelStateMachine, LowLevelTransition,
    LowLevelTransitionId, LowLevelTransitionType,
};
use crate::{
    animation_graph::AnimationGraph, context::spec_context::NodeSpec, errors::GraphValidationError,
};

/// Unique within a high-level FSM
#[derive(
    UuidWrapper, Clone, Copy, Debug, Reflect, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
pub struct StateId(#[uuid] pub(crate) Uuid);

/// Unique within a high-level FSM
#[derive(
    UuidWrapper, Clone, Copy, Debug, Reflect, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
pub struct DirectTransitionId(#[uuid] pub(crate) Uuid);

/// It's convenient to have a way to refer to a given transition in a state machine, of any kind,
/// uniquely.
#[derive(
    Clone,
    Copy,
    Debug,
    Reflect,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Serialize,
    Deserialize,
)]
pub enum TransitionId {
    Direct(DirectTransitionId),
    State(StateId),
    #[default]
    Fallback,
}

/// Specification of a state node in the low-level FSM
#[derive(Reflect, Debug, Clone, Default)]
pub struct State {
    pub id: StateId,
    pub label: String,
    pub graph: Handle<AnimationGraph>,
    pub state_transition: Option<TransitionData>,
}

#[derive(Reflect, Debug, Clone, Default)]
pub struct DirectTransition {
    pub id: DirectTransitionId,
    pub source: StateId,
    pub target: StateId,
    pub data: TransitionData,
}

#[derive(Reflect, Debug, Clone, Default)]
pub struct TransitionData {
    pub kind: TransitionKind,
}

#[derive(Reflect, Debug, Clone, Default)]
pub enum TransitionKind {
    #[default]
    Immediate,
    Graph {
        graph: Handle<AnimationGraph>,
        /// If set, will automatically end the transition after the given time has elapsed
        timed: Option<f32>,
    },
}

/// Stateful data associated with an FSM node
#[derive(Asset, Reflect, Debug, Default, Clone)]
#[reflect(Asset)]
pub struct StateMachine {
    pub start_state: StateId,

    pub states: HashMap<StateId, State>,
    pub transitions: HashMap<DirectTransitionId, DirectTransition>,
    pub node_spec: NodeSpec,

    #[reflect(ignore)]
    pub extra: FsmEditorMetadata,

    #[reflect(ignore)]
    low_level_fsm: LowLevelStateMachine,
}

impl StateMachine {
    pub fn add_state(&mut self, state: State) {
        self.extra.state_added(state.id);
        self.states.insert(state.id.clone(), state);
    }

    pub fn add_transition_unchecked(&mut self, transition: DirectTransition) {
        self.transitions.insert(transition.id.clone(), transition);
    }

    pub fn add_transition_from_ui(
        &mut self,
        transition: DirectTransition,
    ) -> Result<(), GraphValidationError> {
        if !self.states.contains_key(&transition.source)
            || !self.states.contains_key(&transition.target)
        {
            return Err(GraphValidationError::UnknownError(
                "Transition connects states that don't exist!".into(),
            ));
        }

        self.add_transition_unchecked(transition);
        self.update_low_level_fsm();

        Ok(())
    }

    pub fn set_start_state(&mut self, start_state: StateId) {
        self.start_state = start_state;
    }

    pub fn set_input_spec(&mut self, spec: NodeSpec) {
        self.node_spec = spec;
        self.update_low_level_fsm();
    }

    pub fn update_state(
        &mut self,
        old_state_name: StateId,
        new_state: State,
    ) -> Result<(), GraphValidationError> {
        if !self.states.contains_key(&old_state_name) {
            return Err(GraphValidationError::UnknownError(
                "Old state id does not exist!".into(),
            ));
        }

        // first, verify that if the name of the node changed, we can perform the rename
        if old_state_name != new_state.id {
            return Err(GraphValidationError::UnknownError(
                "Cannot change the ID of a state".into(),
            ));
        }

        self.states.insert(new_state.id.clone(), new_state);

        self.update_low_level_fsm();

        Ok(())
    }

    pub fn delete_state(&mut self, state_name: StateId) -> Result<(), GraphValidationError> {
        if !self.states.contains_key(&state_name) {
            return Err(GraphValidationError::UnknownError(
                "State id to be delted does not exist!".into(),
            ));
        }

        self.transitions
            .retain(|_, v| v.source != state_name && v.target != state_name);

        self.states.remove(&state_name);
        self.update_low_level_fsm();

        Ok(())
    }

    pub fn update_transition(
        &mut self,
        old_transition_name: DirectTransitionId,
        new_transition: DirectTransition,
    ) -> Result<(), GraphValidationError> {
        if !self.transitions.contains_key(&old_transition_name) {
            return Err(GraphValidationError::UnknownError(
                "Old transition id does not exist!".into(),
            ));
        }
        if old_transition_name != new_transition.id {
            return Err(GraphValidationError::UnknownError(
                "Transition id can't be updated!".into(),
            ));
        }

        if !self.states.contains_key(&new_transition.source)
            || !self.states.contains_key(&new_transition.target)
        {
            return Err(GraphValidationError::UnknownError(
                "Transition connects states that don't exist!".into(),
            ));
        }

        self.transitions
            .insert(new_transition.id.clone(), new_transition);

        self.update_low_level_fsm();
        Ok(())
    }

    pub fn delete_transition(
        &mut self,
        transition_name: DirectTransitionId,
    ) -> Result<(), GraphValidationError> {
        if !self.transitions.contains_key(&transition_name) {
            return Err(GraphValidationError::UnknownError(
                "Transition id to be deleted does not exist!".into(),
            ));
        }
        self.transitions.remove(&transition_name);
        self.update_low_level_fsm();
        Ok(())
    }

    /// Update the low-level FSM with the current high-level FSM data. This should be called after
    /// mutating the high-level FSM, otherwise the execution will not reflect the changes.
    pub fn update_low_level_fsm(&mut self) {
        let mut llfsm = LowLevelStateMachine::new();

        llfsm.start_state = Some(LowLevelStateId::HlState(self.start_state.clone()));
        llfsm.node_spec = self.node_spec.clone();

        for state in self.states.values() {
            llfsm.add_state(low_level::LowLevelState {
                id: LowLevelStateId::HlState(state.id.clone()),
                graph: state.graph.clone(),
                hl_transition: None,
            });
            if let Some(state_transition) = &state.state_transition {
                let transition_id = TransitionId::State(state.id);
                for source_state in self.states.values() {
                    if source_state.id != state.id {
                        match &state_transition.kind {
                            TransitionKind::Immediate => {
                                llfsm.add_transition(LowLevelTransition {
                                    id: LowLevelTransitionId::Immediate(transition_id),
                                    source: LowLevelStateId::HlState(source_state.id),
                                    target: LowLevelStateId::HlState(state.id),
                                    transition_type: LowLevelTransitionType::State,
                                    hl_source: source_state.id,
                                    hl_target: state.id,
                                });
                            }
                            TransitionKind::Graph { graph, timed } => {
                                llfsm.add_state(LowLevelState {
                                    id: LowLevelStateId::HlTransition(transition_id),
                                    graph: graph.clone(),
                                    hl_transition: Some(low_level::LlTransitionData {
                                        source: source_state.id,
                                        target: state.id,
                                        hl_transition_id: transition_id,
                                        timed: *timed,
                                    }),
                                });
                                llfsm.add_transition(LowLevelTransition {
                                    id: LowLevelTransitionId::Start(transition_id),
                                    source: LowLevelStateId::HlState(source_state.id),
                                    target: LowLevelStateId::HlTransition(transition_id),
                                    transition_type: LowLevelTransitionType::State,
                                    hl_source: source_state.id,
                                    hl_target: state.id,
                                });
                                llfsm.add_transition(LowLevelTransition {
                                    id: LowLevelTransitionId::End(transition_id),
                                    source: LowLevelStateId::HlTransition(transition_id),
                                    target: LowLevelStateId::HlState(state.id),
                                    transition_type: LowLevelTransitionType::State,
                                    hl_source: source_state.id,
                                    hl_target: state.id,
                                });
                            }
                        }
                    }
                }
            }
        }

        for transition in self.transitions.values() {
            let transition_id = TransitionId::Direct(transition.id);
            match &transition.data.kind {
                TransitionKind::Immediate => {
                    llfsm.add_transition(LowLevelTransition {
                        id: LowLevelTransitionId::Immediate(transition_id),
                        source: LowLevelStateId::HlState(transition.source),
                        target: LowLevelStateId::HlTransition(transition_id),
                        transition_type: LowLevelTransitionType::Direct,
                        hl_source: transition.source,
                        hl_target: transition.target,
                    });
                }
                TransitionKind::Graph { graph, timed } => {
                    llfsm.add_state(LowLevelState {
                        id: LowLevelStateId::HlTransition(transition_id),
                        graph: graph.clone(),
                        hl_transition: Some(low_level::LlTransitionData {
                            source: transition.source,
                            target: transition.target,
                            hl_transition_id: transition_id,
                            timed: *timed,
                        }),
                    });

                    llfsm.add_transition(LowLevelTransition {
                        id: LowLevelTransitionId::Start(transition_id),
                        source: LowLevelStateId::HlState(transition.source),
                        target: LowLevelStateId::HlTransition(transition_id),
                        transition_type: LowLevelTransitionType::Direct,
                        hl_source: transition.source,
                        hl_target: transition.target,
                    });

                    llfsm.add_transition(LowLevelTransition {
                        id: LowLevelTransitionId::End(transition_id),
                        source: LowLevelStateId::HlTransition(transition_id),
                        target: LowLevelStateId::HlState(transition.target),
                        transition_type: LowLevelTransitionType::Direct,
                        hl_source: transition.source,
                        hl_target: transition.target,
                    });
                }
            }
        }

        self.low_level_fsm = llfsm;
    }

    pub fn get_low_level_fsm(&self) -> &LowLevelStateMachine {
        &self.low_level_fsm
    }
}

/// Stores the positions of nodes in the canvas for the editor
#[derive(Reflect, Debug, Clone, Serialize, Deserialize, Default)]
pub struct FsmEditorMetadata {
    pub states: HashMap<StateId, Vec2>,
}

impl FsmEditorMetadata {
    /// Set node position (for editor)
    pub fn set_state_position(&mut self, node_id: impl Into<StateId>, position: Vec2) {
        self.states.insert(node_id.into(), position);
    }

    /// Set node position (for editor)
    pub fn move_state(&mut self, node_id: impl Into<StateId>, delta: Vec2) {
        let id = node_id.into();
        let prev_pos = self.states.get(&id).copied().unwrap_or(Vec2::ZERO);
        self.states.insert(id, prev_pos + delta);
    }

    /// Add default position for new node if not already there
    pub fn state_added(&mut self, node_id: StateId) {
        let node_id = node_id.into();
        if !self.states.contains_key(&node_id) {
            self.states.insert(node_id, Vec2::ZERO);
        }
    }
}
