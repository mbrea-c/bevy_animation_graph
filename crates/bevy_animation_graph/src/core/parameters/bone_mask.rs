use crate::core::pose::BoneId;
use bevy::{
    reflect::{std_traits::ReflectDefault, Reflect},
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

#[derive(Reflect, Clone, Debug)]
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

#[derive(Reflect, Clone, Serialize, Deserialize, Debug)]
pub enum BoneMaskSerial {
    Positive { bones: HashMap<Vec<String>, f32> },
    Negative { bones: HashMap<Vec<String>, f32> },
}

fn deserialize_bone_map(map: HashMap<Vec<String>, f32>) -> HashMap<BoneId, f32> {
    map.into_iter().map(|(k, v)| (k.into(), v)).collect()
}

fn serialize_bone_map(map: HashMap<BoneId, f32>) -> HashMap<Vec<String>, f32> {
    map.into_iter().map(|(k, v)| (k.into(), v)).collect()
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

impl From<BoneMask> for BoneMaskSerial {
    fn from(value: BoneMask) -> Self {
        match value {
            BoneMask::Positive { bones } => BoneMaskSerial::Positive {
                bones: serialize_bone_map(bones),
            },
            BoneMask::Negative { bones } => BoneMaskSerial::Negative {
                bones: serialize_bone_map(bones),
            },
        }
    }
}

impl Serialize for BoneMask {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        BoneMaskSerial::from(self.clone()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BoneMask {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        BoneMaskSerial::deserialize(deserializer).map(BoneMask::from)
    }
}
