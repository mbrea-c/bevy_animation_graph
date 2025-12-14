use bevy::asset::AssetPath;
use serde::{Deserialize, Serialize};

use super::{FsmEditorMetadata, State, StateId, StateMachine, Transition, TransitionId};
use crate::{context::spec_context::NodeSpec, state_machine::high_level::TransitionVariant};

#[derive(Serialize, Deserialize, Clone)]
pub struct StateSerial {
    pub id: StateId,
    pub label: String,
    pub graph: AssetPath<'static>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransitionSerial {
    pub id: TransitionId,
    pub variant: TransitionVariant,
    pub duration: f32,
    pub graph: AssetPath<'static>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StateMachineSerial {
    pub states: Vec<StateSerial>,
    pub transitions: Vec<TransitionSerial>,
    pub start_state: StateId,
    #[serde(default)]
    pub node_spec: NodeSpec,
    #[serde(default)]
    pub extra: FsmEditorMetadata,
}

impl From<&Transition> for TransitionSerial {
    fn from(value: &Transition) -> Self {
        Self {
            id: value.id,
            variant: value.variant.clone(),
            duration: value.duration,
            graph: value.graph.path().unwrap().clone_owned(),
        }
    }
}

impl From<&State> for StateSerial {
    fn from(value: &State) -> Self {
        Self {
            id: value.id.clone(),
            label: value.label.clone(),
            graph: value.graph.path().unwrap().clone_owned(),
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
            node_spec: value.node_spec.clone(),
        }
    }
}
