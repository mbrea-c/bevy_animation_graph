use bevy::{platform::collections::HashMap, reflect::Reflect};
use serde::{Deserialize, Serialize};

use crate::{
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

impl RagdollConfig {
    pub fn body_mode(&self, body: BodyId) -> Option<BodyMode> {
        self.mode_overrides
            .get(&body)
            .copied()
            .or(self.default_mode)
    }

    pub fn should_readback(&self, bone: BoneId) -> Option<bool> {
        self.readback_overrides
            .get(&bone)
            .copied()
            .or(self.default_readback)
    }
}
