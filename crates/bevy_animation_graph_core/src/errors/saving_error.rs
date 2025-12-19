use bevy::{asset::UntypedAssetId, prelude::*};
use thiserror::Error;

/// Possible errors that can be produced by graph evaluation
#[non_exhaustive]
#[derive(Debug, Error, Reflect, Clone)]
pub enum SavingError {
    #[error("Assets without an asset path cannot be serialized: {0:?}")]
    MissingAssetPath(UntypedAssetId),
}
