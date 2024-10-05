use super::{bone_mask::BoneMask, EventQueue};
use crate::{
    core::{animation_clip::EntityPath, pose::Pose},
    utils::unwrap::UnwrapVal,
};
use bevy::{
    math::{Quat, Vec3},
    reflect::{std_traits::ReflectDefault, Reflect},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    Vec3,
    EntityPath,
    Quat,
    BoneMask,
    Pose,
    EventQueue,
}

#[derive(Serialize, Deserialize, Reflect, Clone, Debug)]
pub enum DataValue {
    // trivial copy
    F32(f32),
    Bool(bool),
    Vec3(Vec3),
    Quat(Quat),
    // non-trivial copy
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

impl DataValue {
    // trivial copy

    #[must_use]
    pub const fn as_f32(&self) -> Option<f32> {
        match self {
            &Self::F32(x) => Some(x),
            _ => None,
        }
    }

    #[must_use]
    pub fn into_f32(self) -> Option<f32> {
        self.as_f32()
    }

    #[must_use]
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            &Self::Bool(x) => Some(x),
            _ => None,
        }
    }

    #[must_use]
    pub fn into_bool(self) -> Option<bool> {
        self.as_bool()
    }

    #[must_use]
    pub const fn as_vec3(&self) -> Option<Vec3> {
        match self {
            &Self::Vec3(x) => Some(x),
            _ => None,
        }
    }

    #[must_use]
    pub fn into_vec3(self) -> Option<Vec3> {
        self.as_vec3()
    }

    #[must_use]
    pub const fn as_quat(&self) -> Option<Quat> {
        match self {
            &Self::Quat(x) => Some(x),
            _ => None,
        }
    }

    #[must_use]
    pub fn into_quat(self) -> Option<Quat> {
        self.as_quat()
    }

    // non-trivial copy

    #[must_use]
    pub fn into_entity_path(self) -> Option<EntityPath> {
        match self {
            Self::EntityPath(x) => Some(x),
            _ => None,
        }
    }

    #[must_use]
    pub fn into_bone_mask(self) -> Option<BoneMask> {
        match self {
            Self::BoneMask(x) => Some(x),
            _ => None,
        }
    }

    #[must_use]
    pub fn into_pose(self) -> Option<Pose> {
        match self {
            Self::Pose(x) => Some(x),
            _ => None,
        }
    }

    #[must_use]
    pub fn into_event_queue(self) -> Option<EventQueue> {
        match self {
            Self::EventQueue(x) => Some(x),
            _ => None,
        }
    }
}

impl UnwrapVal<f32> for DataValue {
    fn val(self) -> f32 {
        match self {
            DataValue::F32(f) => f,
            _ => panic!("Expected F32, found {:?}", DataSpec::from(&self)),
        }
    }
}

impl UnwrapVal<bool> for DataValue {
    fn val(self) -> bool {
        match self {
            DataValue::Bool(b) => b,
            _ => panic!("Expected Bool, found {:?}", DataSpec::from(&self)),
        }
    }
}

impl UnwrapVal<EntityPath> for DataValue {
    fn val(self) -> EntityPath {
        match self {
            DataValue::EntityPath(f) => f,
            _ => panic!("Expected EntityPath, found {:?}", DataSpec::from(&self)),
        }
    }
}

impl UnwrapVal<BoneMask> for DataValue {
    fn val(self) -> BoneMask {
        match self {
            DataValue::BoneMask(b) => b,
            _ => panic!("Expected BoneMask, found {:?}", DataSpec::from(&self)),
        }
    }
}

impl UnwrapVal<Quat> for DataValue {
    fn val(self) -> Quat {
        match self {
            DataValue::Quat(q) => q,
            _ => panic!("Expected Quat, found {:?}", DataSpec::from(&self)),
        }
    }
}

impl UnwrapVal<Vec3> for DataValue {
    fn val(self) -> Vec3 {
        match self {
            DataValue::Vec3(v) => v,
            _ => panic!("Expected Vec3, found {:?}", DataSpec::from(&self)),
        }
    }
}

impl UnwrapVal<Pose> for DataValue {
    fn val(self) -> Pose {
        match self {
            DataValue::Pose(v) => v,
            _ => panic!("Expected Pose, found {:?}", DataSpec::from(&self)),
        }
    }
}

impl UnwrapVal<EventQueue> for DataValue {
    fn val(self) -> EventQueue {
        match self {
            DataValue::EventQueue(v) => v,
            _ => panic!("Expected EventQueue, found {:?}", DataSpec::from(&self)),
        }
    }
}

impl DataValue {
    pub fn unwrap_f32(self) -> f32 {
        match self {
            DataValue::F32(f) => f,
            _ => panic!("Expected F32, found {:?}", DataSpec::from(&self)),
        }
    }
}

#[derive(Debug, Clone, Copy, Error)]
#[error("wrong data value kind")]
pub struct WrongDataValueKind;

macro_rules! impl_variant {
    ($unwrapped:ident, $wrapped:ident) => {
        impl From<$unwrapped> for DataValue {
            fn from(value: $unwrapped) -> Self {
                Self::$wrapped(value)
            }
        }

        impl TryFrom<DataValue> for $unwrapped {
            type Error = WrongDataValueKind;

            fn try_from(value: DataValue) -> Result<Self, Self::Error> {
                match value {
                    DataValue::$wrapped(v) => Ok(v),
                    _ => Err(WrongDataValueKind),
                }
            }
        }
    };
}

impl_variant!(bool, Bool);
impl_variant!(f32, F32);
impl_variant!(Vec3, Vec3);
impl_variant!(Quat, Quat);
impl_variant!(EntityPath, EntityPath);
impl_variant!(BoneMask, BoneMask);
impl_variant!(Pose, Pose);
impl_variant!(EventQueue, EventQueue);

impl From<&DataValue> for DataSpec {
    fn from(value: &DataValue) -> Self {
        match value {
            DataValue::F32(_) => DataSpec::F32,
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
