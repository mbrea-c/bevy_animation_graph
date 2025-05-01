use crate::core::pose::BoneId;
use bevy::{
    platform::collections::HashMap,
    reflect::{Reflect, std_traits::ReflectDefault},
};
use serde::{Deserialize, Serialize};

#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Default)]
pub enum BoneMask {
    /// If a bone is in the bones map, weight is given. Otherwise, weight is zero
    Positive { bones: HashMap<BoneId, f32> },
    /// If a bone is not in bones map, weight is 1. Otherwise, weight is as given
    Negative { bones: HashMap<BoneId, f32> },
}

impl Default for BoneMask {
    fn default() -> Self {
        Self::Positive {
            bones: Default::default(),
        }
    }
}

impl BoneMask {
    pub fn bone_weight(&self, bone_id: &BoneId) -> f32 {
        match self {
            BoneMask::Positive { bones } => bones.get(bone_id).copied().unwrap_or(0.),
            BoneMask::Negative { bones } => bones.get(bone_id).copied().unwrap_or(1.),
        }
    }
}
