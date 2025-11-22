use bevy::prelude::*;
use thiserror::Error;

use crate::core::animation_graph::{GraphInputPin, NodeId, SourcePin, TargetPin};

/// Possible errors that can be produced by graph evaluation
#[non_exhaustive]
#[derive(Debug, Error, Reflect, Clone)]
pub enum GraphError {
    #[error("The graph input pin type used was not appropriate in this context: {0:?}")]
    InvalidGraphInputPinType(GraphInputPin),
    #[error("Graph input data could not be retrieved {0:?}")]
    MissingGraphInputData(GraphInputPin),
    #[error("Graph input duration could not be retrieved {0:?}")]
    MissingGraphInputDuration(GraphInputPin),

    #[error("Expected an edge connected to the target {0:?}")]
    MissingEdgeToTarget(TargetPin),
    #[error("Expected an edge connected to the source {0:?}")]
    MissingEdgeToSource(SourcePin),
    #[error("Node update did not produce output for {0:?}")]
    OutputMissing(SourcePin),
    #[error("Node update did not produce output for {0:?}")]
    DurationMissing(SourcePin),
    #[error("Time update requested is not cached: {0:?}")]
    TimeUpdateMissingBack(TargetPin),
    #[error("Time update requested is not cached: {0:?}")]
    TimeUpdateMissingFwd(SourcePin),
    #[error("A parent graph was requested but none is present")]
    MissingParentGraph,
    #[error("Graph requested data from FSM transition, but it is assigned to a state")]
    FSMExpectedTransitionFoundState,
    #[error("FSM sub-graph requested data that isn't available")]
    FSMRequestedMissingData,
    #[error(
        "The current state id does not match any known states, perhaps you deleted a state while the state machine was running?"
    )]
    FSMCurrentStateMissing,
    #[error("The asset for a state's graph is missing.")]
    FSMGraphAssetMissing,
    #[error("We're missing a skeleton asset at: {0:?}")]
    SkeletonMissing(NodeId),
    #[error("The requested animation clip is not found.")]
    ClipMissing,
    #[error(
        "Failed to update time. Possibly the requested event track does not exist in a given clip."
    )]
    TimeUpdateFailed,
    #[error("Tried to convert to incorrect data type: expected {0}, got {1}")]
    MismatchedDataType(String, String),
    #[error("Tried to get node state of the wrong type")]
    MismatchedStateType,
    #[error("State value not found for the given parameters")]
    MissingStateValue,
}

pub type GraphResult<T> = Result<T, GraphError>;
