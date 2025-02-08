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
#[derive(Asset, Reflect, Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
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

    pub fn linear_add(&self, other: &BonePose) -> Self {
        Self {
            rotation: either_or_mix(self.rotation, other.rotation, |a, b| a + b),
            translation: either_or_mix(self.translation, other.translation, |a, b| a + b),
            scale: either_or_mix(self.scale, other.scale, |a, b| a + b),
            weights: either_or_mix(self.weights.clone(), other.weights.clone(), |a, b| {
                a.into_iter().zip(b).map(|(a, b)| a + b).collect()
            }),
        }
    }

    pub fn scalar_mult(&self, alpha: f32) -> Self {
        Self {
            rotation: self.rotation.map(|r| r * alpha),
            translation: self.translation.map(|t| alpha * t),
            scale: self.scale.map(|s| alpha * s),
            weights: self
                .weights
                .clone()
                .map(|w| w.into_iter().map(|a| alpha * a).collect()),
        }
    }

    pub fn normalize_quat(&self) -> Self {
        Self {
            rotation: self.rotation.map(|r| r.normalize()),
            translation: self.translation,
            scale: self.scale,
            weights: self.weights.clone(),
        }
    }
}

/// Vertical slice of an [`GraphClip`]
///
/// [`GraphClip`]: crate::prelude::GraphClip
#[derive(Asset, Reflect, Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
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

    pub fn linear_add(&self, other: &Pose) -> Self {
        self.combine(other, |l, r| l.linear_add(r))
    }

    pub fn scalar_mult(&self, alpha: f32) -> Self {
        self.map_bones(|bone| bone.scalar_mult(alpha))
    }

    pub fn normalize_quat(&self) -> Self {
        self.map_bones(|bone| bone.normalize_quat())
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

    pub fn map_bones(&self, func: impl Fn(&BonePose) -> BonePose) -> Self {
        let mut result = Pose::default();

        for (path, bone_id) in self.paths.iter() {
            result.add_bone(func(&self.bones[*bone_id]), *path);
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

#[cfg(test)]
mod test {
    use std::f32::consts::PI;

    use super::*;

    #[test]
    fn test_bone_pose_quat_scalar_mult() {
        let original_bone_pose = BonePose {
            rotation: Some(Quat::from_rotation_z(PI)),
            translation: None,
            scale: None,
            weights: None,
        };

        let result_bone_pose = original_bone_pose.scalar_mult(0.5);
        let renormalized_bone_pose = result_bone_pose.normalize_quat();

        println!("{:#?}", original_bone_pose);
        println!("{:#?}", result_bone_pose);
        println!("{:#?}", renormalized_bone_pose);

        assert!(false);
    }

    #[test]
    fn test_bone_pose_quat_add() {
        let original_bone_pose_a = BonePose {
            rotation: Some(Quat::from_rotation_z(PI / 4.)),
            translation: None,
            scale: None,
            weights: None,
        };

        let original_bone_pose_b = BonePose {
            rotation: Some(Quat::from_rotation_z(-PI / 8.)),
            translation: None,
            scale: None,
            weights: None,
        };

        let result = original_bone_pose_a
            .scalar_mult(0.5)
            .linear_add(&original_bone_pose_b.scalar_mult(0.5));

        println!("{:#?}", original_bone_pose_a);
        println!("{:#?}", original_bone_pose_b);
        println!("{:#?}", result);
        println!("{:#?}", result.normalize_quat());

        assert!(false);
    }
}
