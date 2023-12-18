use bevy::{math::Quat, reflect::Reflect};
use serde::{Deserialize, Serialize};

use crate::utils::unwrap::Unwrap;

use super::bone_mask::{BoneMask, BoneMaskSerial};

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
    BoneMask,
    Quat,
}

#[derive(Reflect, Clone, Debug)]
pub enum ParamValue {
    F32(f32),
    Quat(Quat),
    BoneMask(BoneMask),
}

#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
pub enum ParamValueSerial {
    F32(f32),
    Quat(Quat),
    BoneMask(BoneMaskSerial),
}

impl From<ParamValueSerial> for ParamValue {
    fn from(value: ParamValueSerial) -> Self {
        match value {
            ParamValueSerial::F32(v) => ParamValue::F32(v),
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

impl From<&ParamValue> for ParamSpec {
    fn from(value: &ParamValue) -> Self {
        match value {
            ParamValue::F32(_) => ParamSpec::F32,
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
