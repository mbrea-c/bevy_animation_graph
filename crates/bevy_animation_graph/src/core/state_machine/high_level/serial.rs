use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};

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
}
