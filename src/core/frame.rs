use super::animation_clip::EntityPath;
use crate::utils::unwrap::Unwrap;
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
