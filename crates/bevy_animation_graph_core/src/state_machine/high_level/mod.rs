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
    LowLevelState, LowLevelStateId, LowLevelStateMachine, LowLevelTransition, LowLevelTransitionId,
    LowLevelTransitionType,
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
pub struct TransitionId(#[uuid] pub(crate) Uuid);

/// Specification of a state node in the low-level FSM
#[derive(Reflect, Debug, Clone, Default)]
pub struct State {
    pub id: StateId,
    pub label: String,
    pub graph: Handle<AnimationGraph>,
}

#[derive(Reflect, Debug, Clone, Default)]
pub struct Transition {
    pub id: TransitionId,
    pub variant: TransitionVariant,
    pub duration: f32,
    pub graph: Handle<AnimationGraph>,
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
pub enum TransitionVariant {
    // From state A to state B
    Direct { source: StateId, target: StateId },
    // From any state to state B
    State { target: StateId },
}

impl Default for TransitionVariant {
    fn default() -> Self {
        Self::Direct {
            source: Default::default(),
            target: Default::default(),
        }
    }
}

/// Stateful data associated with an FSM node
#[derive(Asset, Reflect, Debug, Default, Clone)]
#[reflect(Asset)]
pub struct StateMachine {
    pub start_state: StateId,

    #[reflect(ignore)]
    pub states: HashMap<StateId, State>,
    #[reflect(ignore)]
    pub transitions: HashMap<TransitionId, Transition>,

    #[reflect(ignore)]
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

    pub fn add_transition_unchecked(&mut self, transition: Transition) {
        self.transitions.insert(transition.id.clone(), transition);
    }

    // TODO: REMOVE THIS AND UPDATE `add_transition`
    pub fn add_transition_from_ui(
        &mut self,
        transition: Transition,
    ) -> Result<(), GraphValidationError> {
        match &transition.variant {
            TransitionVariant::Direct { source, target } => {
                if !self.states.contains_key(source) || !self.states.contains_key(target) {
                    return Err(GraphValidationError::UnknownError(
                        "Transition connects states that don't exist!".into(),
                    ));
                }
            }
            TransitionVariant::State { target } => {
                if !self.states.contains_key(target) {
                    return Err(GraphValidationError::UnknownError(
                        "Transition connects states that don't exist!".into(),
                    ));
                }
            }
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

        self.transitions.retain(|_, v| match &v.variant {
            TransitionVariant::Direct { source, target } => {
                *source != state_name && *target != state_name
            }
            TransitionVariant::State { target } => *target != state_name,
        });

        self.states.remove(&state_name);
        self.update_low_level_fsm();

        Ok(())
    }

    pub fn update_transition(
        &mut self,
        old_transition_name: TransitionId,
        new_transition: Transition,
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

        match &new_transition.variant {
            TransitionVariant::Direct { source, target } => {
                if !self.states.contains_key(source) || !self.states.contains_key(target) {
                    return Err(GraphValidationError::UnknownError(
                        "Transition connects states that don't exist!".into(),
                    ));
                }
            }
            TransitionVariant::State { target } => {
                if !self.states.contains_key(target) {
                    return Err(GraphValidationError::UnknownError(
                        "Transition connects states that don't exist!".into(),
                    ));
                }
            }
        }

        self.transitions
            .insert(new_transition.id.clone(), new_transition);

        self.update_low_level_fsm();
        Ok(())
    }

    pub fn delete_transition(
        &mut self,
        transition_name: TransitionId,
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
            llfsm.add_state(super::low_level::LowLevelState {
                id: LowLevelStateId::HlState(state.id.clone()),
                graph: state.graph.clone(),
                hl_transition: None,
            });
        }

        for transition in self.transitions.values() {
            match &transition.variant {
                TransitionVariant::Direct { source, target } => {
                    llfsm.add_state(LowLevelState {
                        id: LowLevelStateId::DirectTransition(transition.id),
                        graph: transition.graph.clone(),
                        hl_transition: Some(super::low_level::TransitionData {
                            source: *source,
                            target: *target,
                            hl_transition_id: transition.id.clone(),
                            duration: transition.duration,
                        }),
                    });

                    llfsm.add_transition(LowLevelTransition {
                        id: LowLevelTransitionId::Start(transition.id),
                        source: LowLevelStateId::HlState(*source),
                        target: LowLevelStateId::DirectTransition(transition.id),
                        transition_type: LowLevelTransitionType::Direct,
                        hl_source: *source,
                        hl_target: *target,
                    });

                    llfsm.add_transition(LowLevelTransition {
                        id: LowLevelTransitionId::End(transition.id),
                        source: LowLevelStateId::DirectTransition(transition.id),
                        target: LowLevelStateId::HlState(*target),
                        transition_type: LowLevelTransitionType::Direct,
                        hl_source: *source,
                        hl_target: *target,
                    });
                }
                TransitionVariant::State { target } => {
                    for source_state in self.states.values() {
                        if source_state.id != *target {
                            llfsm.add_state(LowLevelState {
                                id: LowLevelStateId::GlobalTransition(source_state.id, *target),
                                graph: transition.graph.clone(),
                                hl_transition: Some(super::low_level::TransitionData {
                                    source: source_state.id,
                                    target: *target,
                                    hl_transition_id: transition.id,
                                    duration: transition.duration,
                                }),
                            });
                            llfsm.add_transition(LowLevelTransition {
                                id: LowLevelTransitionId::Start(transition.id),
                                source: LowLevelStateId::HlState(source_state.id),
                                target: LowLevelStateId::GlobalTransition(source_state.id, *target),
                                transition_type: LowLevelTransitionType::Global,
                                hl_source: source_state.id,
                                hl_target: *target,
                            });
                            llfsm.add_transition(LowLevelTransition {
                                id: LowLevelTransitionId::End(transition.id),
                                source: LowLevelStateId::GlobalTransition(
                                    source_state.id.clone(),
                                    *target,
                                ),
                                target: LowLevelStateId::HlState(*target),
                                transition_type: LowLevelTransitionType::Global,
                                hl_source: source_state.id,
                                hl_target: *target,
                            });
                        }
                    }
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
