use bevy::{
    asset::prelude::*, math::prelude::*, platform::collections::HashMap, reflect::prelude::*,
    transform::prelude::*,
};
use serde::{Deserialize, Serialize};

pub use super::id::BoneId;
use super::skeleton::Skeleton;

/// Controls how root motion is extracted from an animation clip.
///
/// Set this on a [`ClipNode`] to enable root motion extraction.
#[derive(Reflect, Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum RootMotionMode {
    /// No root motion extraction (default). The root bone's transform stays in the pose as-is.
    #[default]
    Disabled,
    /// Extract full translation and rotation delta from the root bone and zero it in the visual
    /// pose (replace with rest pose values).
    Full,
    /// Extract only XZ translation and Y-axis rotation from the root bone. Y translation (vertical
    /// bob) and XZ rotation (tilt) remain in the visual pose.
    GroundPlane,
}

/// Per-frame displacement of the root bone, extracted from an animation clip.
///
/// When root motion is enabled on a [`ClipNode`], the root bone's translation
/// and rotation are extracted as a delta between frames. This delta can be used
/// to drive character movement via physics or a character controller, while
/// the visual animation plays in-place.
///
/// Root motion deltas are automatically blended when poses are blended
/// (e.g., through [`BlendNode`] or animation transitions).
#[derive(Clone, Debug, Serialize, Deserialize, Reflect, PartialEq)]
#[reflect(Default)]
pub struct RootMotionDelta {
    /// Translation displacement this frame.
    pub translation: Vec3,
    /// Rotation displacement this frame.
    pub rotation: Quat,
}

impl Default for RootMotionDelta {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
        }
    }
}

impl RootMotionDelta {
    /// Linearly blend between two root motion deltas.
    /// Translation is lerped, rotation is slerped.
    pub fn linear_blend(&self, other: &Self, alpha: f32) -> Self {
        Self {
            translation: self.translation.lerp(other.translation, alpha),
            rotation: self.rotation.slerp(other.rotation, alpha),
        }
    }

    /// Additive blend: adds the other delta's contribution scaled by alpha.
    pub fn additive_blend(&self, other: &Self, alpha: f32) -> Self {
        Self {
            translation: self.translation + alpha * other.translation,
            rotation: additive_blend_quat(self.rotation, other.rotation, alpha),
        }
    }

    /// Scale the delta by a scalar factor.
    /// Translation is multiplied directly. Rotation is slerped from identity.
    pub fn scale(&self, factor: f32) -> Self {
        Self {
            translation: factor * self.translation,
            rotation: Quat::IDENTITY.slerp(self.rotation, factor),
        }
    }

    /// Add two deltas together: translations sum, rotations compose.
    pub fn linear_add(&self, other: &Self) -> Self {
        Self {
            translation: self.translation + other.translation,
            rotation: linear_add_quaternion(self.rotation, other.rotation),
        }
    }

    /// Compute the difference between two root motion deltas.
    pub fn difference(&self, other: &Self) -> Self {
        Self {
            translation: other.translation - self.translation,
            rotation: other.rotation * self.rotation.inverse(),
        }
    }
}

/// Vertical slice of a [`Keyframes`] that represents an instant in an animation [`Transform`].
///
/// [`Keyframes`]: crate::core::animation_clip::Keyframes
/// [`Transform`]: bevy::transform::prelude::Transform
#[derive(Asset, Reflect, Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct BonePose {
    pub translation: Option<Vec3>,
    pub rotation: Option<Quat>,
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

    pub fn from_transform(transform: Transform) -> Self {
        Self {
            translation: Some(transform.translation),
            rotation: Some(transform.rotation),
            scale: Some(transform.scale),
            weights: None,
        }
    }

    pub fn additive_blend(&self, other: &BonePose, alpha: f32) -> Self {
        let mut result = self.clone();
        result.additive_blend_mut(other, alpha);
        result
    }

    pub fn additive_blend_mut(&mut self, other: &BonePose, alpha: f32) {
        self.rotation = either_or_mix(self.rotation, other.rotation, |a, b| {
            additive_blend_quat(a, b, alpha)
        });
        self.translation = either_or_mix(self.translation, other.translation, |a, b| a + alpha * b);
        self.scale = either_or_mix(self.scale, other.scale, |a, b| a + alpha * b);
        self.weights = either_or_mix(self.weights.clone(), other.weights.clone(), |a, b| {
            a.into_iter().zip(b).map(|(a, b)| a + alpha * b).collect()
        });
    }

    pub fn linear_blend_mut(&mut self, other: &BonePose, alpha: f32) {
        self.rotation = either_or_mix(self.rotation, other.rotation, |a, b| a.slerp(b, alpha));
        self.translation =
            either_or_mix(self.translation, other.translation, |a, b| a.lerp(b, alpha));
        self.scale = either_or_mix(self.scale, other.scale, |a, b| a.lerp(b, alpha));
        self.weights = either_or_mix(self.weights.clone(), other.weights.clone(), |a, b| {
            a.iter()
                .zip(b)
                .map(|(old, new)| (new - old) * alpha)
                .collect()
        });
    }

    pub fn difference_mut(&mut self, other: &BonePose) {
        self.rotation = either_or_mix(self.rotation, other.rotation, |a, b| b * a.inverse());
        self.translation = either_or_mix(self.translation, other.translation, |a, b| b - a);
        self.scale = either_or_mix(self.scale, other.scale, |a, b| b - a);
        self.weights = either_or_mix(self.weights.clone(), other.weights.clone(), |a, b| {
            a.into_iter().zip(b).map(|(a, b)| b - a).collect()
        });
    }
    pub fn difference(&self, other: &BonePose) -> Self {
        let mut result = self.clone();
        result.difference_mut(other);
        result
    }

    pub fn linear_add(&self, other: &BonePose) -> Self {
        Self {
            rotation: either_or_mix(self.rotation, other.rotation, |a, b| {
                linear_add_quaternion(a, b)
            }),
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

    pub fn overlay(&self, other: &BonePose) -> Self {
        Self {
            translation: either_or_mix(self.translation, other.translation, |_, b| b),
            rotation: either_or_mix(self.rotation, other.rotation, |_, b| b),
            scale: either_or_mix(self.scale, other.scale, |_, b| b),
            weights: either_or_mix(self.weights.clone(), other.weights.clone(), |_, b| b),
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
    /// Root motion delta for this frame, if root motion extraction is active.
    /// Automatically blended when poses are blended via interpolators.
    #[serde(default)]
    pub root_motion: Option<RootMotionDelta>,
}

impl Pose {
    pub fn add_bone(&mut self, pose: BonePose, bone_id: BoneId) {
        let id = self.bones.len();
        self.bones.insert(id, pose);
        self.paths.insert(bone_id, id);
    }

    pub fn additive_blend(&self, other: &Pose, alpha: f32) -> Self {
        self.combine(other, |ba, bb| ba.additive_blend(bb, alpha))
    }

    pub fn difference(&self, other: &Pose) -> Self {
        self.combine(other, |l, r| l.difference(r))
    }

    pub fn linear_add(&self, other: &Pose) -> Self {
        let mut result = self.combine(other, |l, r| l.linear_add(r));
        result.root_motion = match (&self.root_motion, &other.root_motion) {
            (Some(a), Some(b)) => Some(a.linear_add(b)),
            (Some(a), None) => Some(a.clone()),
            (None, Some(b)) => Some(b.clone()),
            (None, None) => None,
        };
        result
    }

    pub fn scalar_mult(&self, alpha: f32) -> Self {
        let mut result = self.map_bones(|bone| bone.scalar_mult(alpha));
        result.root_motion = self.root_motion.as_ref().map(|rm| rm.scale(alpha));
        result
    }

    pub fn normalize_quat(&self) -> Self {
        let mut result = self.map_bones(|bone| bone.normalize_quat());
        result.root_motion = self.root_motion.as_ref().map(|rm| RootMotionDelta {
            translation: rm.translation,
            rotation: rm.rotation.normalize(),
        });
        result
    }

    pub fn overlay(&self, other: &Pose) -> Self {
        self.combine(other, |l, r| l.overlay(r))
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
        result.root_motion = self.root_motion.clone();

        result
    }

    pub fn map_bones(&self, func: impl Fn(&BonePose) -> BonePose) -> Self {
        let mut result = Pose::default();

        for (path, bone_index) in self.paths.iter() {
            result.add_bone(func(&self.bones[*bone_index]), *path);
        }

        result.timestamp = self.timestamp;
        result.skeleton = self.skeleton.clone();
        result.root_motion = self.root_motion.clone();

        result
    }

    pub fn get_bone(&self, bone_id: BoneId) -> Option<&BonePose> {
        self.paths
            .get(&bone_id)
            .and_then(|idx| self.bones.get(*idx))
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

fn linear_add_quaternion(a: Quat, b: Quat) -> Quat {
    if a.dot(b) < 0. { a - b } else { a + b }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_2;

    use super::*;

    fn approx_eq_vec3(a: Vec3, b: Vec3, epsilon: f32) -> bool {
        (a - b).length() < epsilon
    }

    fn approx_eq_quat(a: Quat, b: Quat, epsilon: f32) -> bool {
        // Quaternions q and -q represent the same rotation
        let dot = a.dot(b).abs();
        (1.0 - dot) < epsilon
    }

    fn make_pose_with_root_motion(translation: Vec3, rotation: Quat) -> Pose {
        Pose {
            root_motion: Some(RootMotionDelta {
                translation,
                rotation,
            }),
            ..Pose::default()
        }
    }

    // --- RootMotionDelta tests ---

    #[test]
    fn test_root_motion_delta_default() {
        let d = RootMotionDelta::default();
        assert_eq!(d.translation, Vec3::ZERO);
        assert_eq!(d.rotation, Quat::IDENTITY);
    }

    #[test]
    fn test_root_motion_delta_linear_blend_endpoints() {
        let a = RootMotionDelta {
            translation: Vec3::new(1.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
        };
        let b = RootMotionDelta {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_y(FRAC_PI_2),
        };

        // alpha=0 should return a
        let r0 = a.linear_blend(&b, 0.0);
        assert!(approx_eq_vec3(r0.translation, a.translation, 1e-5));
        assert!(approx_eq_quat(r0.rotation, a.rotation, 1e-5));

        // alpha=1 should return b
        let r1 = a.linear_blend(&b, 1.0);
        assert!(approx_eq_vec3(r1.translation, b.translation, 1e-5));
        assert!(approx_eq_quat(r1.rotation, b.rotation, 1e-5));
    }

    #[test]
    fn test_root_motion_delta_linear_blend_midpoint() {
        let a = RootMotionDelta {
            translation: Vec3::new(2.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
        };
        let b = RootMotionDelta {
            translation: Vec3::new(0.0, 4.0, 0.0),
            rotation: Quat::from_rotation_y(FRAC_PI_2),
        };

        let mid = a.linear_blend(&b, 0.5);
        assert!(approx_eq_vec3(
            mid.translation,
            Vec3::new(1.0, 2.0, 0.0),
            1e-5
        ));
        // Midpoint rotation should be halfway between identity and 90deg Y
        let expected_rot = Quat::IDENTITY.slerp(Quat::from_rotation_y(FRAC_PI_2), 0.5);
        assert!(approx_eq_quat(mid.rotation, expected_rot, 1e-5));
    }

    #[test]
    fn test_root_motion_delta_additive_blend() {
        let a = RootMotionDelta {
            translation: Vec3::new(1.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
        };
        let b = RootMotionDelta {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::IDENTITY,
        };

        let result = a.additive_blend(&b, 0.5);
        // translation = a + 0.5 * b = (1, 0, 0) + (0, 1, 0) = (1, 1, 0)
        assert!(approx_eq_vec3(
            result.translation,
            Vec3::new(1.0, 1.0, 0.0),
            1e-5
        ));
    }

    #[test]
    fn test_root_motion_delta_scale() {
        let d = RootMotionDelta {
            translation: Vec3::new(2.0, 4.0, 6.0),
            rotation: Quat::from_rotation_y(FRAC_PI_2),
        };

        let zero = d.scale(0.0);
        assert!(approx_eq_vec3(zero.translation, Vec3::ZERO, 1e-5));
        assert!(approx_eq_quat(zero.rotation, Quat::IDENTITY, 1e-5));

        let half = d.scale(0.5);
        assert!(approx_eq_vec3(
            half.translation,
            Vec3::new(1.0, 2.0, 3.0),
            1e-5
        ));

        let full = d.scale(1.0);
        assert!(approx_eq_vec3(full.translation, d.translation, 1e-5));
        assert!(approx_eq_quat(full.rotation, d.rotation, 1e-5));
    }

    #[test]
    fn test_root_motion_delta_linear_add() {
        let a = RootMotionDelta {
            translation: Vec3::new(1.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
        };
        let b = RootMotionDelta {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::IDENTITY,
        };

        let result = a.linear_add(&b);
        assert!(approx_eq_vec3(
            result.translation,
            Vec3::new(1.0, 2.0, 0.0),
            1e-5
        ));
    }

    #[test]
    fn test_root_motion_delta_difference() {
        let a = RootMotionDelta {
            translation: Vec3::new(1.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
        };
        let b = RootMotionDelta {
            translation: Vec3::new(3.0, 2.0, 0.0),
            rotation: Quat::IDENTITY,
        };

        let diff = a.difference(&b);
        assert!(approx_eq_vec3(
            diff.translation,
            Vec3::new(2.0, 2.0, 0.0),
            1e-5
        ));
    }

    // --- Pose root motion propagation tests ---

    #[test]
    fn test_pose_combine_preserves_root_motion() {
        let a = make_pose_with_root_motion(Vec3::new(1.0, 0.0, 0.0), Quat::IDENTITY);
        let b = Pose::default(); // no root motion

        let result = a.combine(&b, |l, _r| l.clone());
        assert!(result.root_motion.is_some());
        assert!(approx_eq_vec3(
            result.root_motion.unwrap().translation,
            Vec3::new(1.0, 0.0, 0.0),
            1e-5
        ));
    }

    #[test]
    fn test_pose_map_bones_preserves_root_motion() {
        let p = make_pose_with_root_motion(Vec3::new(1.0, 2.0, 3.0), Quat::IDENTITY);
        let result = p.map_bones(|b| b.clone());
        assert!(result.root_motion.is_some());
        assert!(approx_eq_vec3(
            result.root_motion.unwrap().translation,
            Vec3::new(1.0, 2.0, 3.0),
            1e-5
        ));
    }

    #[test]
    fn test_pose_scalar_mult_scales_root_motion() {
        let p = make_pose_with_root_motion(Vec3::new(2.0, 4.0, 6.0), Quat::IDENTITY);
        let result = p.scalar_mult(0.5);
        assert!(result.root_motion.is_some());
        assert!(approx_eq_vec3(
            result.root_motion.unwrap().translation,
            Vec3::new(1.0, 2.0, 3.0),
            1e-5
        ));
    }

    #[test]
    fn test_pose_linear_add_adds_root_motion() {
        let a = make_pose_with_root_motion(Vec3::new(1.0, 0.0, 0.0), Quat::IDENTITY);
        let b = make_pose_with_root_motion(Vec3::new(0.0, 2.0, 0.0), Quat::IDENTITY);
        let result = a.linear_add(&b);
        assert!(result.root_motion.is_some());
        assert!(approx_eq_vec3(
            result.root_motion.unwrap().translation,
            Vec3::new(1.0, 2.0, 0.0),
            1e-5
        ));
    }

    #[test]
    fn test_pose_linear_add_one_side_none() {
        let a = make_pose_with_root_motion(Vec3::new(1.0, 0.0, 0.0), Quat::IDENTITY);
        let b = Pose::default(); // no root motion
        let result = a.linear_add(&b);
        assert!(result.root_motion.is_some());
        assert!(approx_eq_vec3(
            result.root_motion.unwrap().translation,
            Vec3::new(1.0, 0.0, 0.0),
            1e-5
        ));
    }

    #[test]
    fn test_pose_normalize_quat_normalizes_root_motion() {
        let p = Pose {
            root_motion: Some(RootMotionDelta {
                translation: Vec3::new(1.0, 2.0, 3.0),
                rotation: Quat::from_xyzw(0.0, 0.5, 0.0, 0.5), // not normalized
            }),
            ..Pose::default()
        };
        let result = p.normalize_quat();
        assert!(result.root_motion.is_some());
        let rm = result.root_motion.unwrap();
        assert!((rm.rotation.length() - 1.0).abs() < 1e-5);
        assert!(approx_eq_vec3(
            rm.translation,
            Vec3::new(1.0, 2.0, 3.0),
            1e-5
        ));
    }
}
