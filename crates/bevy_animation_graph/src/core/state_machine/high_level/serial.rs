use serde::{Deserialize, Serialize};

use super::{Extra, State, StateMachine, Transition};

pub type StateIdSerial = String;
pub type TransitionIdSerial = String;

#[derive(Serialize, Deserialize, Clone)]
pub struct StateSerial {
    pub id: StateIdSerial,
    pub graph: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransitionSerial {
    pub id: TransitionIdSerial,
    pub source: StateIdSerial,
    pub target: StateIdSerial,
    pub duration: f32,
    pub graph: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StateMachineSerial {
    pub states: Vec<StateSerial>,
    pub transitions: Vec<TransitionSerial>,
    pub start_state: String,
    #[serde(default)]
    pub extra: Extra,
}

impl From<&Transition> for TransitionSerial {
    fn from(value: &Transition) -> Self {
        Self {
            id: value.id.clone(),
            source: value.source.clone(),
            target: value.target.clone(),
            duration: value.duration,
            graph: value.graph.path().unwrap().to_string(),
        }
    }
}

impl From<&State> for StateSerial {
    fn from(value: &State) -> Self {
        Self {
            id: value.id.clone(),
            graph: value.graph.path().unwrap().to_string(),
        }
    }
}

impl From<&StateMachine> for StateMachineSerial {
    fn from(value: &StateMachine) -> Self {
        Self {
            states: value.states.values().map(StateSerial::from).collect(),
            transitions: value
                .transitions
                .values()
                .map(TransitionSerial::from)
                .collect(),
            start_state: value.start_state.clone(),
            extra: value.extra.clone(),
        }
    }
}
