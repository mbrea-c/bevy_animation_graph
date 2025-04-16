use crate::core::animation_graph::{NodeId, SourcePin, TargetPin};
use bevy::prelude::*;
use thiserror::Error;

/// Possible errors that can be produced by graph evaluation
#[non_exhaustive]
#[derive(Debug, Error, Reflect, Clone)]
pub enum GraphError {
    #[error("Expected an edge connected to the target {0:?}")]
    MissingEdgeToTarget(TargetPin),
    #[error("Expected an edge connected to the source {0:?}")]
    MissingEdgeToSource(SourcePin),
    #[error("Node update did not produce output for {0:?}")]
    OutputMissing(SourcePin),
    #[error("Time update requested is not cached: {0:?}")]
    TimeUpdateMissing(TargetPin),
    #[error("A parent graph was requested but none is present")]
    MissingParentGraph,
    #[error("Graph requested data from FSM transition, but it is assigned to a state")]
    FSMExpectedTransitionFoundState,
    #[error("FSM sub-graph requested data that isn't available")]
    FSMRequestedMissingData,
    #[error("The current state id does not match any known states, perhaps you deleted a state while the state machine was running?")]
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
}

pub type GraphResult<T> = Result<T, GraphError>;
