use bevy::{asset::prelude::*, math::prelude::*, reflect::prelude::*, utils::HashMap};

use super::animation_clip::EntityPath;

// PERF: Bone ids should become integers eventually
pub type BoneId = EntityPath;

/// Vertical slice of a [`Keyframes`] that represents an instant in an animation [`Transform`].
#[derive(Asset, Reflect, Clone, Debug, Default)]
pub struct BonePose {
    pub(crate) rotation: Option<Quat>,
    pub(crate) translation: Option<Vec3>,
    pub(crate) scale: Option<Vec3>,
    pub(crate) weights: Option<Vec<f32>>,
}

/// Vertical slice of an [`AnimationClip`]
#[derive(Asset, Reflect, Clone, Debug, Default)]
pub struct Pose {
    pub(crate) bones: Vec<BonePose>,
    pub(crate) paths: HashMap<BoneId, usize>,
}

impl Pose {
    pub fn add_bone(&mut self, pose: BonePose, path: BoneId) {
        let id = self.bones.len();
        self.bones.insert(id, pose);
        self.paths.insert(path, id);
    }
}
