use super::{EventQueue, bone_mask::BoneMask};
use crate::core::{animation_clip::EntityPath, pose::Pose};
use bevy::{
    math::{Quat, Vec2, Vec3},
    reflect::{Reflect, std_traits::ReflectDefault},
};
use bevy_animation_graph_proc_macros::ValueWrapper;
use serde::{Deserialize, Serialize};

#[derive(Reflect, Clone, Copy, Debug, Serialize, Deserialize, Default)]
#[reflect(Default)]
pub struct OptDataSpec {
    pub spec: DataSpec,
    pub optional: bool,
}

impl OptDataSpec {
    pub fn with_optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }
}

impl From<DataSpec> for OptDataSpec {
    fn from(value: DataSpec) -> Self {
        Self {
            spec: value,
            optional: false,
        }
    }
}

#[derive(Reflect, Default, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[reflect(Default)]
pub enum DataSpec {
    #[default]
    F32,
    Bool,
    Vec2,
    Vec3,
    EntityPath,
    Quat,
    BoneMask,
    Pose,
    EventQueue,
}

#[derive(Serialize, Deserialize, Reflect, Clone, Debug, ValueWrapper)]
#[unwrap_error(error(crate::core::errors::GraphError), variant(MismatchedDataType))]
pub enum DataValue {
    #[trivial_copy]
    F32(f32),
    #[trivial_copy]
    Bool(bool),
    #[trivial_copy]
    Vec2(Vec2),
    #[trivial_copy]
    Vec3(Vec3),
    #[trivial_copy]
    Quat(Quat),

    EntityPath(EntityPath),
    BoneMask(BoneMask),
    Pose(Pose),
    EventQueue(EventQueue),
}

impl Default for DataValue {
    fn default() -> Self {
        Self::F32(0.)
    }
}

impl From<&DataValue> for DataSpec {
    fn from(value: &DataValue) -> Self {
        match value {
            DataValue::F32(_) => DataSpec::F32,
            DataValue::Vec2(_) => DataSpec::Vec2,
            DataValue::Vec3(_) => DataSpec::Vec3,
            DataValue::EntityPath(_) => DataSpec::EntityPath,
            DataValue::Quat(_) => DataSpec::Quat,
            DataValue::BoneMask(_) => DataSpec::BoneMask,
            DataValue::Pose(_) => DataSpec::Pose,
            DataValue::EventQueue(_) => DataSpec::EventQueue,
            DataValue::Bool(_) => DataSpec::Bool,
        }
    }
}

impl From<&DataValue> for OptDataSpec {
    fn from(value: &DataValue) -> Self {
        Self {
            spec: value.into(),
            optional: false,
        }
    }
}
