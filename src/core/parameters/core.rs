use super::bone_mask::{BoneMask, BoneMaskSerial};
use crate::{core::animation_clip::EntityPath, utils::unwrap::Unwrap};
use bevy::{
    math::{Quat, Vec3},
    reflect::Reflect,
};
use serde::{Deserialize, Serialize};

#[derive(Reflect, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct OptParamSpec {
    pub spec: ParamSpec,
    pub optional: bool,
}

impl OptParamSpec {
    pub fn with_optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }
}

impl From<ParamSpec> for OptParamSpec {
    fn from(value: ParamSpec) -> Self {
        Self {
            spec: value,
            optional: false,
        }
    }
}

#[derive(Reflect, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ParamSpec {
    F32,
    Vec3,
    EntityPath,
    Quat,
    BoneMask,
}

#[derive(Reflect, Clone, Debug)]
pub enum ParamValue {
    F32(f32),
    Vec3(Vec3),
    EntityPath(EntityPath),
    Quat(Quat),
    BoneMask(BoneMask),
}

#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
pub enum ParamValueSerial {
    F32(f32),
    Vec3(Vec3),
    EntityPath(Vec<String>),
    Quat(Quat),
    BoneMask(BoneMaskSerial),
}

impl From<ParamValueSerial> for ParamValue {
    fn from(value: ParamValueSerial) -> Self {
        match value {
            ParamValueSerial::F32(v) => ParamValue::F32(v),
            ParamValueSerial::Vec3(v) => ParamValue::Vec3(v),
            ParamValueSerial::EntityPath(v) => ParamValue::EntityPath(v.into()),
            ParamValueSerial::Quat(v) => ParamValue::Quat(v),
            ParamValueSerial::BoneMask(v) => ParamValue::BoneMask(v.into()),
        }
    }
}

impl Unwrap<f32> for ParamValue {
    fn unwrap(self) -> f32 {
        match self {
            ParamValue::F32(f) => f,
            _ => panic!("Expected F32, found {:?}", ParamSpec::from(&self)),
        }
    }
}

impl Unwrap<EntityPath> for ParamValue {
    fn unwrap(self) -> EntityPath {
        match self {
            ParamValue::EntityPath(f) => f,
            _ => panic!("Expected EntityPath, found {:?}", ParamSpec::from(&self)),
        }
    }
}

impl Unwrap<BoneMask> for ParamValue {
    fn unwrap(self) -> BoneMask {
        match self {
            ParamValue::BoneMask(b) => b,
            _ => panic!("Expected BoneMask, found {:?}", ParamSpec::from(&self)),
        }
    }
}

impl Unwrap<Quat> for ParamValue {
    fn unwrap(self) -> Quat {
        match self {
            ParamValue::Quat(q) => q,
            _ => panic!("Expected Quat, found {:?}", ParamSpec::from(&self)),
        }
    }
}

impl Unwrap<Vec3> for ParamValue {
    fn unwrap(self) -> Vec3 {
        match self {
            ParamValue::Vec3(v) => v,
            _ => panic!("Expected Vec3, found {:?}", ParamSpec::from(&self)),
        }
    }
}

impl ParamValue {
    pub fn unwrap_f32(self) -> f32 {
        match self {
            ParamValue::F32(f) => f,
            _ => panic!("Expected F32, found {:?}", ParamSpec::from(&self)),
        }
    }
}

impl From<f32> for ParamValue {
    fn from(value: f32) -> Self {
        Self::F32(value)
    }
}

impl From<Vec3> for ParamValue {
    fn from(value: Vec3) -> Self {
        Self::Vec3(value)
    }
}

impl From<&ParamValue> for ParamSpec {
    fn from(value: &ParamValue) -> Self {
        match value {
            ParamValue::F32(_) => ParamSpec::F32,
            ParamValue::Vec3(_) => ParamSpec::Vec3,
            ParamValue::EntityPath(_) => ParamSpec::EntityPath,
            ParamValue::Quat(_) => ParamSpec::Quat,
            ParamValue::BoneMask(_) => ParamSpec::BoneMask,
        }
    }
}

impl From<&ParamValue> for OptParamSpec {
    fn from(value: &ParamValue) -> Self {
        Self {
            spec: value.into(),
            optional: false,
        }
    }
}
