use crate::core::animation_graph::{SourcePin, TargetPin};
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
}
