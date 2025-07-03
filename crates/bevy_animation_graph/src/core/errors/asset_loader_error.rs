use bevy::prelude::*;
use thiserror::Error;

use super::GraphValidationError;

/// Possible errors that can be produced by a custom asset loader
// TODO: clean this up
// https://rust-lang.github.io/api-guidelines/interoperability.html?highlight=error#examples-of-error-messages
// - lowercase error messages
// - don't print sources exclusively
// - avoid mega error enums
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum AssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could load shader: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
    #[error("Could not load Gltf: {0}")]
    GltfError(#[from] bevy::gltf::GltfError),
    #[error("Could not find gltf named label: {0}")]
    GltfMissingLabel(String),
    #[error("Could not complete direct asset load: {0}")]
    LoadDirectError(#[from] bevy::asset::LoadDirectError),
    #[error("Animated scene path is incorrect: {0}")]
    AnimatedSceneMissingName(String),
    #[error(
        "Animated scene missing a root (an exsiting AnimationPlayer). Does your scene root have a `Name`?"
    )]
    AnimatedSceneMissingRoot,
    #[error("Graph does not satisfy constraints: {0}")]
    InconsistentGraphError(#[from] GraphValidationError),
    #[error("Failed to load skeleton colliders object")]
    SkeletonColliderLoadError,
}
