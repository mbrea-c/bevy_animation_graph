use crate::core::{animation_clip::EntityPath, pose::BoneId};
use bevy::{core::Name, reflect::Reflect, utils::HashMap};
use serde::{Deserialize, Serialize};

#[derive(Reflect, Clone, Debug)]
pub enum BoneMask {
    /// If a bone is in the bones map, weight is given. Otherwise, weight is zero
    Positive { bones: HashMap<BoneId, f32> },
    /// If a bone is not in bones map, weight is 1. Otherwise, weight is as given
    Negative { bones: HashMap<BoneId, f32> },
}

impl BoneMask {
    pub fn bone_weight(&self, bone_id: &BoneId) -> f32 {
        match self {
            BoneMask::Positive { bones } => bones.get(bone_id).copied().unwrap_or(0.),
            BoneMask::Negative { bones } => bones.get(bone_id).copied().unwrap_or(1.),
        }
    }
}

#[derive(Reflect, Clone, Serialize, Deserialize, Debug)]
pub enum BoneMaskSerial {
    Positive { bones: HashMap<Vec<String>, f32> },
    Negative { bones: HashMap<Vec<String>, f32> },
}

fn deserialize_bone_map(map: HashMap<Vec<String>, f32>) -> HashMap<BoneId, f32> {
    map.into_iter()
        .map(|(k, v)| {
            let k = EntityPath {
                parts: k.into_iter().map(|s| Name::new(s)).collect(),
            };

            (k, v)
        })
        .collect()
}

impl From<BoneMaskSerial> for BoneMask {
    fn from(value: BoneMaskSerial) -> Self {
        match value {
            BoneMaskSerial::Positive { bones } => BoneMask::Positive {
                bones: deserialize_bone_map(bones),
            },
            BoneMaskSerial::Negative { bones } => BoneMask::Negative {
                bones: deserialize_bone_map(bones),
            },
        }
    }
}
