use super::animation_clip::EntityPath;
use bevy::{asset::prelude::*, math::prelude::*, reflect::prelude::*, utils::HashMap};

#[derive(Asset, Reflect, Clone, Debug, Default, PartialEq)]
pub struct ValueFrame<T: FromReflect + TypePath> {
    pub(crate) timestamp: f32,
    pub(crate) prev: T,
    pub(crate) prev_timestamp: f32,
    pub(crate) next: T,
    pub(crate) next_timestamp: f32,
    pub(crate) next_is_wrapped: bool,
}

impl<T: FromReflect + TypePath> ValueFrame<T> {
    pub fn map_ts<F>(&mut self, f: F)
    where
        F: Fn(f32) -> f32,
    {
        self.prev_timestamp = f(self.prev_timestamp);
        self.timestamp = f(self.timestamp);
        self.next_timestamp = f(self.next_timestamp);
    }
}

#[derive(Asset, Reflect, Clone, Debug, Default)]
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

#[derive(Asset, Reflect, Clone, Debug, Default)]
pub struct PoseFrame {
    pub(crate) bones: Vec<BoneFrame>,
    pub(crate) paths: HashMap<EntityPath, usize>,
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
    }
}
