use bevy::{
    platform::collections::HashMap,
    reflect::{Reflect, std_traits::ReflectDefault},
};
use serde::{Deserialize, Serialize};

use crate::{animation_clip::EntityPath, pose::BoneId};

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub struct BoneMask {
    paths: HashMap<BoneId, EntityPath>,
    weights: HashMap<BoneId, f32>,
    base: BoneMaskType,
}

impl BoneMask {
    pub fn bone_weight(&self, bone_id: &BoneId) -> f32 {
        let default = match self.base {
            BoneMaskType::Positive => 0.,
            BoneMaskType::Negative => 1.,
        };
        self.weights.get(bone_id).copied().unwrap_or(default)
    }

    pub fn all() -> Self {
        Self {
            paths: Default::default(),
            weights: Default::default(),
            base: BoneMaskType::Negative,
        }
    }

    pub fn none() -> Self {
        Self {
            paths: Default::default(),
            weights: Default::default(),
            base: BoneMaskType::Positive,
        }
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

#[derive(Serialize, Deserialize)]
pub struct BoneMaskSerial {
    pub weights: HashMap<EntityPath, f32>,
    pub base: BoneMaskType,
}

impl BoneMaskSerial {
    pub fn from_value(value: &BoneMask) -> Self {
        Self {
            weights: value
                .weights
                .clone()
                .into_iter()
                .filter_map(|(bone_id, weight)| Some((value.paths.get(&bone_id)?.clone(), weight)))
                .collect(),
            base: value.base,
        }
    }

    pub fn to_value(&self) -> BoneMask {
        BoneMask {
            paths: self
                .weights
                .keys()
                .cloned()
                .map(|path| (path.id(), path))
                .collect(),
            weights: self
                .weights
                .clone()
                .into_iter()
                .map(|(path, weight)| (path.id(), weight))
                .collect(),
            base: self.base,
        }
    }
}

impl Serialize for BoneMask {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        BoneMaskSerial::from_value(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BoneMask {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(BoneMaskSerial::deserialize(deserializer)?.to_value())
    }
}
