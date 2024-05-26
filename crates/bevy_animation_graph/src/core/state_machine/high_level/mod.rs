pub mod loader;
pub mod serial;

use super::{LowLevelStateMachine, StateId, TransitionId};
use crate::core::animation_graph::AnimationGraph;
use bevy::{
    asset::{Asset, Handle},
    reflect::Reflect,
    utils::HashMap,
};

/// Specification of a state node in the low-level FSM
#[derive(Reflect, Debug, Clone)]
pub struct State {
    pub id: StateId,
    pub graph: Handle<AnimationGraph>,
}

#[derive(Reflect, Debug, Clone)]
pub struct Transition {
    pub id: TransitionId,
    pub source: StateId,
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
}

impl StateMachine {
    pub fn add_state(&mut self, state: State) {
        self.states.insert(state.id.clone(), state);
    }

    pub fn add_transition(&mut self, transition: Transition) {
        self.transitions.insert(transition.id.clone(), transition);
    }

    pub fn set_start_state(&mut self, start_state: StateId) {
        self.start_state = start_state;
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
