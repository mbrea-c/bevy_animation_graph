pub mod loader;
pub mod serial;

use super::{LowLevelStateMachine, StateId, TransitionId};
use crate::core::{animation_graph::AnimationGraph, errors::GraphValidationError};
use bevy::{
    asset::{Asset, Handle},
    math::Vec2,
    reflect::Reflect,
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

/// Specification of a state node in the low-level FSM
#[derive(Reflect, Debug, Clone)]
pub struct State {
    pub id: StateId,
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
    pub fn node_added(&mut self, node_id: impl Into<StateId>) {
        let node_id = node_id.into();
        if !self.states.contains_key(&node_id) {
            self.states.insert(node_id, Vec2::ZERO);
        }
    }

    /// Rename node if node exists and new name is available, otherwise return false.
    pub fn rename_node(&mut self, old_id: impl Into<StateId>, new_id: impl Into<StateId>) -> bool {
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

#[derive(Reflect, Debug, Clone)]
pub struct Transition {
    pub id: TransitionId,
    #[reflect(ignore)]
    pub source: StateId,
    #[reflect(ignore)]
    pub target: StateId,
    pub duration: f32,
    pub graph: Handle<AnimationGraph>,
}

/// Stateful data associated with an FSM node
#[derive(Asset, Reflect, Debug, Default, Clone)]
pub struct StateMachine {
    pub start_state: StateId,
    pub states: HashMap<StateId, State>,
    pub transitions: HashMap<TransitionId, Transition>,
    low_level_fsm: LowLevelStateMachine,
    pub extra: Extra,
}

impl StateMachine {
    pub fn add_state(&mut self, state: State) {
        self.extra.node_added(&state.id);
        self.states.insert(state.id.clone(), state);
    }

    pub fn add_transition(&mut self, transition: Transition) {
        self.transitions.insert(transition.id.clone(), transition);
    }

    pub fn set_start_state(&mut self, start_state: StateId) {
        self.start_state = start_state;
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

            self.extra.rename_node(&old_state_name, &new_state.id);

            // If old node exists, perform rename
            for transition in self.transitions.values_mut() {
                if transition.source == old_state_name {
                    transition.source = new_state.id.clone();
                }
                if transition.target == old_state_name {
                    transition.target = new_state.id.clone();
                }
            }
        }

        self.states.remove(&old_state_name);
        self.states.insert(new_state.id.clone(), new_state);

        self.update_low_level_fsm();

        Ok(())
    }

    /// Update the low-level FSM with the current high-level FSM data. This should be called after
    /// mutating the high-level FSM, otherwise the execution will not reflect the changes.
    pub fn update_low_level_fsm(&mut self) {
        let mut llfsm = LowLevelStateMachine::new();

        llfsm.start_state = Some(self.start_state.clone());

        for state in self.states.values() {
            llfsm.add_state(super::core::LowLevelState {
                id: state.id.clone(),
                graph: state.graph.clone(),
                transition: None,
            });
        }

        for transition in self.transitions.values() {
            llfsm.add_state(super::core::LowLevelState {
                id: format!("{}_state", transition.id),
                graph: transition.graph.clone(),
                transition: Some(super::core::TransitionData {
                    source: transition.source.clone(),
                    target: transition.target.clone(),
                    hl_transition_id: transition.id.clone(),
                    duration: transition.duration,
                }),
            });

            llfsm.add_transition(
                transition.source.clone(),
                transition.id.clone(),
                format!("{}_state", transition.id),
            );

            llfsm.add_transition(
                format!("{}_state", transition.id),
                "end_transition".into(),
                transition.target.clone(),
            );
        }

        self.low_level_fsm = llfsm;
    }

    pub fn get_low_level_fsm(&self) -> &LowLevelStateMachine {
        &self.low_level_fsm
    }
}
