use bevy::{platform::collections::HashMap, reflect::Reflect};
use serde::{Deserialize, Serialize};

use crate::core::{
    id::BoneId,
    ragdoll::definition::{BodyId, BodyMode},
};

/// Determines:
/// * Default rigidbody modes for ragdoll bodies, and per-body overrides.
/// * Default readback configuration for skeleton bones (whether bone position is read back from
///   the ragdoll), and per-bone overrides.
#[derive(Reflect, Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RagdollConfig {
    pub default_mode: Option<BodyMode>,
    pub mode_overrides: HashMap<BodyId, BodyMode>,
    pub default_readback: Option<bool>,
    pub readback_overrides: HashMap<BoneId, bool>,
}
