use super::animation_clip::EntityPath;
use bevy::{asset::prelude::*, math::prelude::*, reflect::prelude::*, utils::HashMap};

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
pub struct PoseFrame {
    pub(crate) bones: Vec<BoneFrame>,
    pub(crate) paths: HashMap<EntityPath, usize>,
    pub(crate) timestamp: f32,
}

impl PoseFrame {
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
        self.timestamp = f(self.timestamp);
    }

    pub(crate) fn verify_timestamp_in_range(&self) -> bool {
        let mut failed = false;

        for bone in self.bones.iter() {
            if let Some(v) = &bone.translation {
                if !(v.prev_timestamp <= self.timestamp && self.timestamp <= v.next_timestamp) {
                    failed = true;
                }
            }
            if let Some(v) = &bone.rotation {
                if !(v.prev_timestamp <= self.timestamp && self.timestamp <= v.next_timestamp) {
                    failed = true;
                }
            }
            if let Some(v) = &bone.scale {
                if !(v.prev_timestamp <= self.timestamp && self.timestamp <= v.next_timestamp) {
                    failed = true;
                }
            }
            if let Some(v) = &bone.weights {
                if !(v.prev_timestamp <= self.timestamp && self.timestamp <= v.next_timestamp) {
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

impl std::fmt::Debug for PoseFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Frame with t={:?}", self.timestamp)?;
        for bone in self.bones.iter() {
            write!(f, "{:?}", bone)?;
        }
        Ok(())
    }
}
