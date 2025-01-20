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
    pub rotation: Option<Quat>,
    pub translation: Option<Vec3>,
    pub scale: Option<Vec3>,
    pub weights: Option<Vec<f32>>,
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

    pub fn additive_blend(&self, other: &BonePose, alpha: f32) -> Self {
        Self {
            rotation: either_or_mix(self.rotation, other.rotation, |a, b| {
                additive_blend_quat(a, b, alpha)
            }),
            translation: either_or_mix(self.translation, other.translation, |a, b| a + alpha * b),
            scale: either_or_mix(self.scale, other.scale, |a, b| a + alpha * b),
            weights: either_or_mix(self.weights.clone(), other.weights.clone(), |a, b| {
                a.into_iter().zip(b).map(|(a, b)| a + alpha * b).collect()
            }),
        }
    }

    pub fn difference(&self, other: &BonePose) -> Self {
        Self {
            rotation: either_or_mix(self.rotation, other.rotation, |a, b| b * a.inverse()),
            translation: either_or_mix(self.translation, other.translation, |a, b| b - a),
            scale: either_or_mix(self.scale, other.scale, |a, b| b - a),
            weights: either_or_mix(self.weights.clone(), other.weights.clone(), |a, b| {
                a.into_iter().zip(b).map(|(a, b)| b - a).collect()
            }),
        }
    }
}

/// Vertical slice of an [`GraphClip`]
///
/// [`GraphClip`]: crate::prelude::GraphClip
#[derive(Asset, Reflect, Clone, Debug, Default, Serialize, Deserialize)]
#[reflect(Default)]
pub struct Pose {
    pub bones: Vec<BonePose>,
    pub paths: HashMap<BoneId, usize>,
    pub timestamp: f32,
    #[serde(skip)]
    pub skeleton: Handle<Skeleton>,
}

impl Pose {
    pub fn add_bone(&mut self, pose: BonePose, path: BoneId) {
        let id = self.bones.len();
        self.bones.insert(id, pose);
        self.paths.insert(path, id);
    }

    pub fn additive_blend(&self, other: &Pose, alpha: f32) -> Self {
        self.combine(other, |ba, bb| ba.additive_blend(bb, alpha))
    }

    pub fn difference(&self, other: &Pose) -> Self {
        self.combine(other, |l, r| l.difference(r))
    }

    pub fn combine(&self, other: &Self, func: impl Fn(&BonePose, &BonePose) -> BonePose) -> Self {
        let mut result = Pose::default();

        for (path, bone_id) in self.paths.iter() {
            if let Some(other_bone_id) = other.paths.get(path) {
                result.add_bone(
                    func(&self.bones[*bone_id], &other.bones[*other_bone_id]),
                    *path,
                );
            } else {
                result.add_bone(self.bones[*bone_id].clone(), *path);
            }
        }

        for (path, bone_id) in other.paths.iter() {
            if self.paths.contains_key(path) {
                continue;
            }
            result.add_bone(other.bones[*bone_id].clone(), *path);
        }

        result.timestamp = self.timestamp;
        result.skeleton = self.skeleton.clone();

        result
    }
}

fn additive_blend_quat(left: Quat, right: Quat, alpha: f32) -> Quat {
    left.slerp(right * left, alpha)
}

fn either_or_mix<T>(a: Option<T>, b: Option<T>, mix: impl Fn(T, T) -> T) -> Option<T> {
    match (a, b) {
        (Some(a), Some(b)) => Some(mix(a, b)),
        (None, None) => None,
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
    }
}
