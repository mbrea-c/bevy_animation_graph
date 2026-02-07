use bevy::asset::{AssetPath, LoadContext};
use serde::{Deserialize, Serialize};

use super::{DirectTransitionId, FsmEditorMetadata, State, StateId, StateMachine};
use crate::{
    context::spec_context::NodeSpec,
    errors::{AssetLoaderError, SavingError},
    state_machine::high_level::{DirectTransition, TransitionData, TransitionKind},
    utils::loading::TryLoad,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct StateSerial {
    pub id: StateId,
    pub label: String,
    pub graph: AssetPath<'static>,
    pub state_transition: Option<TransitionDataSerial>,
}

impl TryFrom<&State> for StateSerial {
    type Error = SavingError;

    fn try_from(value: &State) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            label: value.label.clone(),
            graph: value
                .graph
                .path()
                .ok_or(SavingError::MissingAssetPath(value.graph.id().untyped()))?
                .to_owned(),
            state_transition: if let Some(t) = &value.state_transition {
                Some(TransitionDataSerial::try_from(t)?)
            } else {
                None
            },
        })
    }
}

impl TryLoad<State> for StateSerial {
    type Error = AssetLoaderError;

    fn try_load<'a, 'b>(
        &self,
        load_context: &'a mut LoadContext<'b>,
    ) -> Result<State, Self::Error> {
        Ok(State {
            id: self.id,
            label: self.label.clone(),
            graph: load_context.load(&self.graph),
            state_transition: if let Some(t) = &self.state_transition {
                Some(t.try_load(load_context)?)
            } else {
                None
            },
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransitionDataSerial {
    pub kind: TransitionKindSerial,
    #[serde(default)]
    pub ignore_external_events: bool,
    #[serde(default)]
    pub reset_target_state: bool,
}

impl TryFrom<&TransitionData> for TransitionDataSerial {
    type Error = SavingError;

    fn try_from(value: &TransitionData) -> Result<Self, Self::Error> {
        Ok(Self {
            kind: TransitionKindSerial::try_from(&value.kind)?,
            ignore_external_events: value.ignore_external_events,
            reset_target_state: value.reset_target_state,
        })
    }
}

impl TryLoad<TransitionData> for TransitionDataSerial {
    type Error = AssetLoaderError;

    fn try_load<'a, 'b>(
        &self,
        load_context: &'a mut LoadContext<'b>,
    ) -> Result<TransitionData, Self::Error> {
        Ok(TransitionData {
            kind: self.kind.try_load(load_context)?,
            ignore_external_events: self.ignore_external_events,
            reset_target_state: self.reset_target_state,
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum TransitionKindSerial {
    Immediate,
    Graph {
        graph: AssetPath<'static>,
        /// If set, will automatically end the transition after the given time has elapsed
        timed: Option<f32>,
    },
}

impl TryFrom<&TransitionKind> for TransitionKindSerial {
    type Error = SavingError;

    fn try_from(value: &TransitionKind) -> Result<Self, Self::Error> {
        Ok(match value {
            TransitionKind::Immediate => Self::Immediate,
            TransitionKind::Graph { graph, timed } => Self::Graph {
                graph: graph
                    .path()
                    .ok_or(SavingError::MissingAssetPath(graph.id().untyped()))?
                    .to_owned(),
                timed: *timed,
            },
        })
    }
}

impl TryLoad<TransitionKind> for TransitionKindSerial {
    type Error = AssetLoaderError;

    fn try_load<'a, 'b>(
        &self,
        load_context: &'a mut LoadContext<'b>,
    ) -> Result<TransitionKind, Self::Error> {
        Ok(match self {
            TransitionKindSerial::Immediate => TransitionKind::Immediate,
            TransitionKindSerial::Graph { graph, timed } => TransitionKind::Graph {
                graph: load_context.load(graph),
                timed: *timed,
            },
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DirectTransitionSerial {
    pub id: DirectTransitionId,
    pub source: StateId,
    pub target: StateId,
    pub data: TransitionDataSerial,
}

impl TryFrom<&DirectTransition> for DirectTransitionSerial {
    type Error = SavingError;

    fn try_from(value: &DirectTransition) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            source: value.source,
            target: value.target,
            data: TransitionDataSerial::try_from(&value.data)?,
        })
    }
}

impl TryLoad<DirectTransition> for DirectTransitionSerial {
    type Error = AssetLoaderError;

    fn try_load<'a, 'b>(
        &self,
        load_context: &'a mut LoadContext<'b>,
    ) -> Result<DirectTransition, Self::Error> {
        Ok(DirectTransition {
            id: self.id,
            source: self.source,
            target: self.target,
            data: self.data.try_load(load_context)?,
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StateMachineSerial {
    pub states: Vec<StateSerial>,
    pub transitions: Vec<DirectTransitionSerial>,
    pub start_state: StateId,
    #[serde(default)]
    pub node_spec: NodeSpec,
    #[serde(default)]
    pub editor_metadata: FsmEditorMetadata,
}

impl TryFrom<&StateMachine> for StateMachineSerial {
    type Error = SavingError;
    fn try_from(value: &StateMachine) -> Result<Self, Self::Error> {
        let mut states = Vec::new();
        for s in value.states.values() {
            states.push(StateSerial::try_from(s)?);
        }

        let mut transitions = Vec::new();
        for t in value.transitions.values() {
            transitions.push(DirectTransitionSerial::try_from(t)?);
        }

        Ok(Self {
            states,
            transitions,
            start_state: value.start_state,
            editor_metadata: value.editor_metadata.clone(),
            node_spec: value.node_spec.clone(),
        })
    }
}
