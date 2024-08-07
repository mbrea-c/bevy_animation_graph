pub use super::id::BoneId;
use super::skeleton::Skeleton;
use bevy::{
    asset::prelude::*, math::prelude::*, reflect::prelude::*, transform::prelude::*, utils::HashMap,
};
use serde::{Deserialize, Serialize};

/// Vertical slice of a [`Keyframes`] that represents an instant in an animation [`Transform`].
///
/// [`Keyframes`]: crate::core::animation_clip::Keyframes
/// [`Transform`]: bevy::transform::prelude::Transform
#[derive(Asset, Reflect, Clone, Debug, Default, Serialize, Deserialize)]
pub struct BonePose {
    pub(crate) rotation: Option<Quat>,
    pub(crate) translation: Option<Vec3>,
    pub(crate) scale: Option<Vec3>,
    pub(crate) weights: Option<Vec<f32>>,
}

impl BonePose {
    pub fn to_transform(&self) -> Transform {
        self.to_transform_with_base(Transform::default())
    }

    pub fn to_transform_with_base(&self, mut base: Transform) -> Transform {
        if let Some(translation) = &self.translation {
            base.translation = *translation;
        }

        if let Some(rotation) = &self.rotation {
            base.rotation = *rotation;
        }

        if let Some(scale) = &self.scale {
            base.scale = *scale;
        }

        base
    }
}

/// Vertical slice of an [`GraphClip`]
///
/// [`GraphClip`]: crate::prelude::GraphClip
#[derive(Asset, Reflect, Clone, Debug, Default, Serialize, Deserialize)]
#[reflect(Default)]
pub struct Pose {
    pub(crate) bones: Vec<BonePose>,
    pub(crate) paths: HashMap<BoneId, usize>,
    pub(crate) timestamp: f32,
    #[serde(skip)]
    pub(crate) skeleton: Handle<Skeleton>,
}

impl Pose {
    pub fn add_bone(&mut self, pose: BonePose, path: BoneId) {
        let id = self.bones.len();
        self.bones.insert(id, pose);
        self.paths.insert(path, id);
    }
}
