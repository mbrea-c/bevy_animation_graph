pub mod loader;
pub mod serial;

use bevy::{
    asset::{Asset, Handle, ReflectAsset},
    math::Vec2,
    platform::collections::HashMap,
    prelude::ReflectDefault,
    reflect::Reflect,
};
use serde::{Deserialize, Serialize};

use super::low_level::{
    LowLevelState, LowLevelStateId, LowLevelStateMachine, LowLevelTransition, LowLevelTransitionId,
    LowLevelTransitionType,
};
use crate::core::{
    animation_graph::{AnimationGraph, PinMap},
    edge_data::DataValue,
    errors::GraphValidationError,
};

/// Unique within a high-level FSM
pub type StateId = String;

/// Unique within a high-level FSM
#[derive(Reflect, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransitionId {
    /// Direct transitions are still indexed by String Id
    Direct(String),
    /// There can only be a single global transition between any state pair
    Global(StateId, StateId),
}

impl Default for TransitionId {
    fn default() -> Self {
        Self::Direct("".to_string())
    }
}

/// Specification of a state node in the low-level FSM
#[derive(Reflect, Debug, Clone, Default)]
pub struct State {
    pub id: StateId,
    pub graph: Handle<AnimationGraph>,
    pub global_transition: Option<GlobalTransition>,
}

#[derive(Reflect, Debug, Clone, Default)]
#[reflect(Default)]
pub struct GlobalTransition {
    pub duration: f32,
    pub graph: Handle<AnimationGraph>,
}

/// Stores the positions of nodes in the canvas for the editor
#[derive(Reflect, Debug, Clone, Serialize, Deserialize, Default)]
pub struct Extra {
    pub states: HashMap<StateId, Vec2>,
}

impl Extra {
    /// Set node position (for editor)
    pub fn set_node_position(&mut self, node_id: impl Into<StateId>, position: Vec2) {
        self.states.insert(node_id.into(), position);
    }

    /// Add default position for new node if not already there
    pub fn state_added(&mut self, node_id: impl Into<StateId>) {
        let node_id = node_id.into();
        if !self.states.contains_key(&node_id) {
            self.states.insert(node_id, Vec2::ZERO);
        }
    }

    /// Rename node if node exists and new name is available, otherwise return false.
    pub fn rename_state(&mut self, old_id: impl Into<StateId>, new_id: impl Into<StateId>) -> bool {
        let old_id = old_id.into();
        let new_id = new_id.into();

        if !self.states.contains_key(&old_id) || self.states.contains_key(&new_id) {
            return false;
        }

        let pos = self.states.remove(&old_id).unwrap();
        self.states.insert(new_id, pos);

        true
    }
}

#[derive(Reflect, Debug, Clone, Default)]
pub struct Transition {
    pub id: TransitionId,
    pub source: StateId,
    pub target: StateId,
    pub duration: f32,
    pub graph: Handle<AnimationGraph>,
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

    pub input_data: PinMap<DataValue>,

    #[reflect(ignore)]
    pub extra: Extra,
    #[reflect(ignore)]
    low_level_fsm: LowLevelStateMachine,
}

impl StateMachine {
    pub fn add_state(&mut self, state: State) {
        self.extra.state_added(&state.id);
        self.states.insert(state.id.clone(), state);
    }

    pub fn add_transition(&mut self, transition: Transition) {
        self.transitions.insert(transition.id.clone(), transition);
    }

    // TODO: REMOVE THIS AND UPDATE `add_transition`
    pub fn add_transition_from_ui(
        &mut self,
        transition: Transition,
    ) -> Result<(), GraphValidationError> {
        if !self.states.contains_key(&transition.source)
            || !self.states.contains_key(&transition.target)
        {
            return Err(GraphValidationError::UnknownError(
                "Transition connects states that don't exist!".into(),
            ));
        }

        self.transitions.insert(transition.id.clone(), transition);
        self.update_low_level_fsm();

        Ok(())
    }

    pub fn set_start_state(&mut self, start_state: StateId) {
        self.start_state = start_state;
    }

    pub fn set_input_data(&mut self, input_data: PinMap<DataValue>) {
        self.input_data = input_data;
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
            if self.states.contains_key(&new_state.id) {
                return Err(GraphValidationError::UnknownError(
                    "State id already exists!".into(),
                ));
            }

            self.extra.rename_state(&old_state_name, &new_state.id);

            // If old node exists, perform rename
            for transition in self.transitions.values_mut() {
                if transition.source == old_state_name {
                    transition.source.clone_from(&new_state.id);
                }
                if transition.target == old_state_name {
                    transition.target.clone_from(&new_state.id);
                }
            }
        }

        self.states.remove(&old_state_name);
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
        old_transition_name: TransitionId,
        new_transition: Transition,
    ) -> Result<(), GraphValidationError> {
        if !self.transitions.contains_key(&old_transition_name) {
            return Err(GraphValidationError::UnknownError(
                "Old transition id does not exist!".into(),
            ));
        }
        if old_transition_name != new_transition.id
            && self.transitions.contains_key(&new_transition.id)
        {
            return Err(GraphValidationError::UnknownError(
                "Transition id already exists!".into(),
            ));
        }
        if !self.states.contains_key(&new_transition.source)
            || !self.states.contains_key(&new_transition.target)
        {
            return Err(GraphValidationError::UnknownError(
                "Transition connects states that don't exist!".into(),
            ));
        }
        self.transitions.remove(&old_transition_name);
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
        llfsm.input_data = self.input_data.clone();

        for state in self.states.values() {
            llfsm.add_state(super::low_level::LowLevelState {
                id: LowLevelStateId::HlState(state.id.clone()),
                graph: state.graph.clone(),
                hl_transition: None,
            });

            if state.global_transition.is_some() {
                // TODO: the source state is inaccurate since it will come from several places
                for source_state in self.states.values() {
                    if source_state.id != state.id {
                        let transition_id =
                            TransitionId::Global(source_state.id.clone(), state.id.clone());

                        llfsm.add_state(LowLevelState {
                            id: LowLevelStateId::GlobalTransition(
                                source_state.id.clone(),
                                state.id.clone(),
                            ),
                            graph: state.global_transition.as_ref().unwrap().graph.clone(),
                            hl_transition: Some(super::low_level::TransitionData {
                                source: source_state.id.clone(),
                                target: state.id.clone(),
                                hl_transition_id: transition_id.clone(),
                                duration: state.global_transition.as_ref().unwrap().duration,
                            }),
                        });
                        llfsm.add_transition(LowLevelTransition {
                            id: LowLevelTransitionId::Start(transition_id.clone()),
                            source: LowLevelStateId::HlState(source_state.id.clone()),
                            target: LowLevelStateId::GlobalTransition(
                                source_state.id.clone(),
                                state.id.clone(),
                            ),
                            transition_type: LowLevelTransitionType::Global,
                            hl_source: source_state.id.clone(),
                            hl_target: state.id.clone(),
                        });
                        llfsm.add_transition(LowLevelTransition {
                            id: LowLevelTransitionId::End(transition_id.clone()),
                            source: LowLevelStateId::GlobalTransition(
                                source_state.id.clone(),
                                state.id.clone(),
                            ),
                            target: LowLevelStateId::HlState(state.id.clone()),
                            transition_type: LowLevelTransitionType::Global,
                            hl_source: source_state.id.clone(),
                            hl_target: state.id.clone(),
                        });
                    }
                }
            }
        }

        for transition in self.transitions.values() {
            llfsm.add_state(LowLevelState {
                id: LowLevelStateId::DirectTransition(transition.id.clone()),
                graph: transition.graph.clone(),
                hl_transition: Some(super::low_level::TransitionData {
                    source: transition.source.clone(),
                    target: transition.target.clone(),
                    hl_transition_id: transition.id.clone(),
                    duration: transition.duration,
                }),
            });

            llfsm.add_transition(LowLevelTransition {
                id: LowLevelTransitionId::Start(transition.id.clone()),
                source: LowLevelStateId::HlState(transition.source.clone()),
                target: LowLevelStateId::DirectTransition(transition.id.clone()),
                transition_type: LowLevelTransitionType::Direct,
                hl_source: transition.source.clone(),
                hl_target: transition.target.clone(),
            });

            llfsm.add_transition(LowLevelTransition {
                id: LowLevelTransitionId::End(transition.id.clone()),
                source: LowLevelStateId::DirectTransition(transition.id.clone()),
                target: LowLevelStateId::HlState(transition.target.clone()),
                transition_type: LowLevelTransitionType::Direct,
                hl_source: transition.source.clone(),
                hl_target: transition.target.clone(),
            });
        }

        self.low_level_fsm = llfsm;
    }

    pub fn get_low_level_fsm(&self) -> &LowLevelStateMachine {
        &self.low_level_fsm
    }
}
