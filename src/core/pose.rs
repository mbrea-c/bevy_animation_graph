use bevy::{asset::prelude::*, math::prelude::*, reflect::prelude::*, utils::HashMap};

use super::animation_clip::EntityPath;

/// Vertical slice of a [`Keyframes`] that represents an instant in an animation [`Transform`].
///
/// [`Keyframes`]: crate::core::animation_clip::Keyframes
/// [`Transform`]: bevy::transform::prelude::Transform
#[derive(Asset, Reflect, Clone, Debug, Default)]
pub struct BonePose {
    pub(crate) rotation: Option<Quat>,
    pub(crate) translation: Option<Vec3>,
    pub(crate) scale: Option<Vec3>,
    pub(crate) weights: Option<Vec<f32>>,
}

/// Vertical slice of an [`GraphClip`]
///
/// [`GraphClip`]: crate::prelude::GraphClip
#[derive(Asset, Reflect, Clone, Debug, Default)]
pub struct Pose {
    pub(crate) bones: Vec<BonePose>,
    pub(crate) paths: HashMap<EntityPath, usize>,
}

impl Pose {
    pub fn add_bone(&mut self, pose: BonePose, path: EntityPath) {
        let id = self.bones.len();
        self.bones.insert(id, pose);
        self.paths.insert(path, id);
    }
}
