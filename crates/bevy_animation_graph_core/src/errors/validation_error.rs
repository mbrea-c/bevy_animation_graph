use thiserror::Error;

use crate::animation_graph::{SourcePin, TargetPin};

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum GraphValidationError {
    #[error("{0:?} and {1:?} have different types but are connected.")]
    InconsistentPinTypes(SourcePin, TargetPin),
    #[error("Catchall error: {0}")]
    UnknownError(String),
}
