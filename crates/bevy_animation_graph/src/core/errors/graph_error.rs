use crate::core::animation_graph::TargetPin;
use bevy::prelude::*;
use thiserror::Error;

/// Possible errors that can be produced by graph evaluation
#[non_exhaustive]
#[derive(Debug, Error, Reflect, Clone)]
pub enum GraphError {
    #[error("Expected an edge connected to {0:?}")]
    MissingInputEdge(TargetPin),
}
