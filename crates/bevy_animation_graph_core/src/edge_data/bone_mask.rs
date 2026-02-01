use bevy::{
    platform::collections::HashMap,
    reflect::{Reflect, std_traits::ReflectDefault},
};
use serde::{Deserialize, Serialize};

use crate::pose::BoneId;

#[derive(Reflect, Clone, Debug, Serialize, Deserialize, Default)]
#[reflect(Default)]
pub struct BoneMask {
    pub weights: HashMap<BoneId, f32>,
    pub base: BoneMaskType,
}

impl BoneMask {
    pub fn bone_weight(&self, bone_id: &BoneId) -> f32 {
        let default = match self.base {
            BoneMaskType::Positive => 0.,
            BoneMaskType::Negative => 1.,
        };
        self.weights.get(bone_id).copied().unwrap_or(default)
    }
}

#[derive(Reflect, Clone, Copy, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
#[reflect(Default)]
pub enum BoneMaskType {
    /// If a bone is in the bones map, weight is given. Otherwise, weight is zero
    #[default]
    Positive,
    /// If a bone is not in bones map, weight is 1. Otherwise, weight is as given
    Negative,
}
