use super::animation_clip::EntityPath;
use crate::{
    prelude::{InterpolateLinear, SampleLinearAt},
    utils::unwrap::Unwrap,
};
use bevy::{asset::prelude::*, math::prelude::*, reflect::prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};

#[derive(Asset, Reflect, Clone, Default, PartialEq)]
pub struct ValueFrame<T: FromReflect + TypePath> {
    pub(crate) prev: T,
    pub(crate) prev_timestamp: f32,
    pub(crate) next: T,
    pub(crate) next_timestamp: f32,
    pub(crate) prev_is_wrapped: bool,
    pub(crate) next_is_wrapped: bool,
}

impl<T: FromReflect + TypePath> ValueFrame<T> {
    pub fn map_ts<F>(&mut self, f: F)
    where
        F: Fn(f32) -> f32,
    {
        self.prev_timestamp = f(self.prev_timestamp);
        self.next_timestamp = f(self.next_timestamp);
    }

    /// Maps the `prev` and `next` values of the frame
    /// using the given function
    pub fn map<Q, F>(&self, f: F) -> ValueFrame<Q>
    where
        Q: FromReflect + TypePath,
        F: Fn(&T) -> Q,
    {
        ValueFrame {
            prev: f(&self.prev),
            prev_timestamp: self.prev_timestamp,
            next: f(&self.next),
            next_timestamp: self.next_timestamp,
            prev_is_wrapped: self.prev_is_wrapped,
            next_is_wrapped: self.next_is_wrapped,
        }
    }

    /// Returns a new frame where `prev_timestamp` is the maximum of `self.prev_timestamp`
    /// and `other.prev_timestamp`, and `next_timestamp` is the minimum of `self.next_timestamp`
    /// and `other.next_timestamp`. Both frames are sampled at the chosen timestamps for either
    /// end using the given sampler and combined using the given combiner function.
    pub fn merge<B, C, SLeft, SRight, F>(
        &self,
        other: &ValueFrame<B>,
        sampler_left: SLeft,
        sampler_right: SRight,
        combiner: F,
    ) -> ValueFrame<C>
    where
        B: FromReflect + TypePath,
        C: FromReflect + TypePath,
        SLeft: Fn(&Self, f32) -> T,
        SRight: Fn(&ValueFrame<B>, f32) -> B,
        F: Fn(&T, &B) -> C,
    {
        let (prev_timestamp, prev, prev_is_wrapped) = if self.prev_timestamp >= other.prev_timestamp
        {
            let ts = self.prev_timestamp;
            let other_prev = sampler_right(other, ts);
            (
                ts,
                combiner(&self.prev, &other_prev),
                self.prev_is_wrapped && other.prev_is_wrapped,
            )
        } else {
            let ts = other.prev_timestamp;
            let self_prev = sampler_left(self, ts);
            (
                ts,
                combiner(&self_prev, &other.prev),
                self.prev_is_wrapped && other.prev_is_wrapped,
            )
        };

        let (next_timestamp, next, next_is_wrapped) = if self.next_timestamp <= other.next_timestamp
        {
            let ts = self.next_timestamp;
            let other_next = sampler_right(other, ts);
            (
                ts,
                combiner(&self.next, &other_next),
                self.next_is_wrapped && other.next_is_wrapped,
            )
        } else {
            let ts = other.next_timestamp;
            let self_next = sampler_left(self, ts);
            (
                ts,
                combiner(&self_next, &other.next),
                self.next_is_wrapped && other.next_is_wrapped,
            )
        };

        ValueFrame {
            prev,
            prev_timestamp,
            next,
            next_timestamp,
            prev_is_wrapped,
            next_is_wrapped,
        }
    }

    /// Returns a new frame where `prev_timestamp` is the maximum of `self.prev_timestamp`
    /// and `other.prev_timestamp`, and `next_timestamp` is the minimum of `self.next_timestamp`
    /// and `other.next_timestamp`. Both frames are sampled linearly at the chosen timestamps for either
    /// end and combined using the given combiner function.
    pub fn merge_linear<B, C, F>(&self, other: &ValueFrame<B>, combiner: F) -> ValueFrame<C>
    where
        T: InterpolateLinear,
        B: FromReflect + TypePath + InterpolateLinear,
        C: FromReflect + TypePath,
        F: Fn(&T, &B) -> C,
    {
        self.merge(
            other,
            ValueFrame::<T>::sample_linear_at,
            ValueFrame::<B>::sample_linear_at,
            combiner,
        )
    }
}

#[derive(Asset, Reflect, Clone, Default)]
pub struct BoneFrame {
    pub(crate) rotation: Option<ValueFrame<Quat>>,
    pub(crate) translation: Option<ValueFrame<Vec3>>,
    pub(crate) scale: Option<ValueFrame<Vec3>>,
    pub(crate) weights: Option<ValueFrame<Vec<f32>>>,
}

impl BoneFrame {
    pub fn map_ts<F>(&mut self, f: F)
    where
        F: Fn(f32) -> f32,
    {
        if let Some(v) = self.rotation.as_mut() {
            v.map_ts(&f)
        };
        if let Some(v) = self.translation.as_mut() {
            v.map_ts(&f)
        };
        if let Some(v) = self.scale.as_mut() {
            v.map_ts(&f)
        };
        if let Some(v) = self.weights.as_mut() {
            v.map_ts(&f)
        };
    }
}

#[derive(Asset, Reflect, Clone, Default)]
pub struct InnerPoseFrame {
    pub(crate) bones: Vec<BoneFrame>,
    pub(crate) paths: HashMap<EntityPath, usize>,
}

/// Pose frame where each transform is local with respect to the parent bone
// TODO: Verify that transforms are wrt parent bone
#[derive(Reflect, Clone, Default, Debug)]
pub struct BonePoseFrame(pub(crate) InnerPoseFrame);
/// Pose frame where each transform is relative to the root of the skeleton
#[derive(Reflect, Clone, Default, Debug)]
pub struct CharacterPoseFrame(pub(crate) InnerPoseFrame);
/// Pose frame where each transform is in world/global space
#[derive(Reflect, Clone, Default, Debug)]
pub struct GlobalPoseFrame(pub(crate) InnerPoseFrame);

impl From<InnerPoseFrame> for BonePoseFrame {
    fn from(value: InnerPoseFrame) -> Self {
        Self(value)
    }
}

impl From<InnerPoseFrame> for CharacterPoseFrame {
    fn from(value: InnerPoseFrame) -> Self {
        Self(value)
    }
}

impl From<InnerPoseFrame> for GlobalPoseFrame {
    fn from(value: InnerPoseFrame) -> Self {
        Self(value)
    }
}

impl BonePoseFrame {
    pub fn inner_ref(&self) -> &InnerPoseFrame {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut InnerPoseFrame {
        &mut self.0
    }

    pub fn inner(self) -> InnerPoseFrame {
        self.0
    }

    pub fn map_ts<F>(&mut self, f: F)
    where
        F: Fn(f32) -> f32,
    {
        self.inner_mut().map_ts(f)
    }
}

impl CharacterPoseFrame {
    pub fn inner_ref(&self) -> &InnerPoseFrame {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut InnerPoseFrame {
        &mut self.0
    }

    pub fn inner(self) -> InnerPoseFrame {
        self.0
    }

    pub fn map_ts<F>(&mut self, f: F)
    where
        F: Fn(f32) -> f32,
    {
        self.inner_mut().map_ts(f)
    }
}

impl GlobalPoseFrame {
    pub fn inner_ref(&self) -> &InnerPoseFrame {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut InnerPoseFrame {
        &mut self.0
    }

    pub fn inner(self) -> InnerPoseFrame {
        self.0
    }

    pub fn map_ts<F>(&mut self, f: F)
    where
        F: Fn(f32) -> f32,
    {
        self.inner_mut().map_ts(f)
    }
}

impl InnerPoseFrame {
    pub(crate) fn add_bone(&mut self, frame: BoneFrame, path: EntityPath) {
        let id = self.bones.len();
        self.bones.insert(id, frame);
        self.paths.insert(path, id);
    }

    pub fn map_ts<F>(&mut self, f: F)
    where
        F: Fn(f32) -> f32,
    {
        self.bones.iter_mut().for_each(|v| v.map_ts(&f));
    }

    pub(crate) fn verify_timestamp_in_range(&self, timestamp: f32) -> bool {
        let mut failed = false;

        for bone in self.bones.iter() {
            if let Some(v) = &bone.translation {
                if !(v.prev_timestamp <= timestamp && timestamp <= v.next_timestamp) {
                    failed = true;
                }
            }
            if let Some(v) = &bone.rotation {
                if !(v.prev_timestamp <= timestamp && timestamp <= v.next_timestamp) {
                    failed = true;
                }
            }
            if let Some(v) = &bone.scale {
                if !(v.prev_timestamp <= timestamp && timestamp <= v.next_timestamp) {
                    failed = true;
                }
            }
            if let Some(v) = &bone.weights {
                if !(v.prev_timestamp <= timestamp && timestamp <= v.next_timestamp) {
                    failed = true;
                }
            }
        }

        failed
    }

    pub(crate) fn verify_timestamps_in_order(&self) -> bool {
        let mut failed = false;

        for bone in self.bones.iter() {
            if let Some(v) = &bone.translation {
                if v.prev_timestamp > v.next_timestamp {
                    failed = true;
                }
            }
            if let Some(v) = &bone.rotation {
                if v.prev_timestamp > v.next_timestamp {
                    failed = true;
                }
            }
            if let Some(v) = &bone.scale {
                if v.prev_timestamp > v.next_timestamp {
                    failed = true;
                }
            }
            if let Some(v) = &bone.weights {
                if v.prev_timestamp > v.next_timestamp {
                    failed = true;
                }
            }
        }

        failed
    }
}

impl<T: FromReflect + TypePath> std::fmt::Debug for ValueFrame<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} <--> {:?}",
            self.prev_timestamp, self.next_timestamp
        )
    }
}

impl std::fmt::Debug for BoneFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\tBone:")?;
        if let Some(v) = &self.translation {
            writeln!(f, "\t\ttranslation: {:?}", v)?;
        }
        if let Some(v) = &self.rotation {
            writeln!(f, "\t\trotation: {:?}", v)?;
        }
        if let Some(v) = &self.scale {
            writeln!(f, "\t\tscale: {:?}", v)?;
        }
        if let Some(v) = &self.weights {
            writeln!(f, "\t\tweight: {:?}", v)?;
        }

        Ok(())
    }
}

impl std::fmt::Debug for InnerPoseFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for bone in self.bones.iter() {
            write!(f, "{:?}", bone)?;
        }
        Ok(())
    }
}

#[derive(Clone, Reflect, Debug)]
pub struct PoseFrame {
    pub data: PoseFrameData,
    pub timestamp: f32,
}

#[derive(Clone, Reflect, Debug)]
pub enum PoseFrameData {
    BoneSpace(BonePoseFrame),
    CharacterSpace(CharacterPoseFrame),
    GlobalSpace(GlobalPoseFrame),
}

impl Default for PoseFrameData {
    fn default() -> Self {
        Self::BoneSpace(BonePoseFrame::default())
    }
}

impl PoseFrameData {
    pub fn map_ts<F>(&mut self, f: F)
    where
        F: Fn(f32) -> f32,
    {
        match self {
            PoseFrameData::BoneSpace(data) => data.map_ts(f),
            PoseFrameData::CharacterSpace(data) => data.map_ts(f),
            PoseFrameData::GlobalSpace(data) => data.map_ts(f),
        }
    }
}

impl PoseFrame {
    pub fn map_ts<F>(&mut self, f: F)
    where
        F: Fn(f32) -> f32,
    {
        self.data.map_ts(&f);
        self.timestamp = f(self.timestamp);
    }

    pub(crate) fn verify_timestamp_in_range(&self) -> bool {
        let inner = match &self.data {
            PoseFrameData::BoneSpace(data) => data.inner_ref(),
            PoseFrameData::CharacterSpace(data) => data.inner_ref(),
            PoseFrameData::GlobalSpace(data) => data.inner_ref(),
        };

        inner.verify_timestamp_in_range(self.timestamp)
    }

    pub(crate) fn verify_timestamps_in_order(&self) -> bool {
        let inner = match &self.data {
            PoseFrameData::BoneSpace(data) => data.inner_ref(),
            PoseFrameData::CharacterSpace(data) => data.inner_ref(),
            PoseFrameData::GlobalSpace(data) => data.inner_ref(),
        };

        inner.verify_timestamps_in_order()
    }
}

#[derive(Clone, Copy, Debug, Reflect, Default, Serialize, Deserialize)]
pub enum PoseSpec {
    #[default]
    BoneSpace,
    CharacterSpace,
    GlobalSpace,
    Any,
}

impl From<&PoseFrameData> for PoseSpec {
    fn from(value: &PoseFrameData) -> Self {
        match value {
            PoseFrameData::BoneSpace(_) => PoseSpec::BoneSpace,
            PoseFrameData::CharacterSpace(_) => PoseSpec::CharacterSpace,
            PoseFrameData::GlobalSpace(_) => PoseSpec::GlobalSpace,
        }
    }
}

impl From<&PoseFrame> for PoseSpec {
    fn from(value: &PoseFrame) -> Self {
        (&value.data).into()
    }
}

impl Unwrap<BonePoseFrame> for PoseFrameData {
    fn unwrap(self) -> BonePoseFrame {
        match self {
            PoseFrameData::BoneSpace(b) => b,
            x => panic!(
                "Found {:?}, expected pose in bone space",
                PoseSpec::from(&x)
            ),
        }
    }
}

impl Unwrap<CharacterPoseFrame> for PoseFrameData {
    fn unwrap(self) -> CharacterPoseFrame {
        match self {
            PoseFrameData::CharacterSpace(b) => b,
            x => panic!(
                "Found {:?}, expected pose in character space",
                PoseSpec::from(&x)
            ),
        }
    }
}

impl Unwrap<GlobalPoseFrame> for PoseFrameData {
    fn unwrap(self) -> GlobalPoseFrame {
        match self {
            PoseFrameData::GlobalSpace(b) => b,
            x => panic!(
                "Found {:?}, expected pose in character space",
                PoseSpec::from(&x)
            ),
        }
    }
}
