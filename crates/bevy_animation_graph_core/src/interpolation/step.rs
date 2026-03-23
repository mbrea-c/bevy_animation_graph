use bevy::prelude::*;

use crate::pose::{BonePose, Pose};

pub trait InterpolateStep {
    fn interpolate_step(&self, other: &Self, f: f32) -> Self;
}

/// Step interpolation between morph weights
impl InterpolateStep for Vec<f32> {
    fn interpolate_step(&self, _other: &Vec<f32>, _f: f32) -> Vec<f32> {
        self.clone()
    }
}

impl InterpolateStep for Vec3 {
    fn interpolate_step(&self, _other: &Self, _f: f32) -> Self {
        *self
    }
}

impl InterpolateStep for Quat {
    fn interpolate_step(&self, _other: &Self, _f: f32) -> Self {
        *self
    }
}

impl InterpolateStep for Transform {
    fn interpolate_step(&self, other: &Self, f: f32) -> Self {
        Transform {
            translation: self.translation.interpolate_step(&other.translation, f),
            rotation: self.rotation.interpolate_step(&other.rotation, f),
            scale: self.scale.interpolate_step(&other.scale, f),
        }
    }
}

impl InterpolateStep for BonePose {
    fn interpolate_step(&self, other: &Self, f: f32) -> Self {
        let mut result = Self::default();

        // TODO: Maybe we should blend with rest pose whenever one channel is missing?

        match (&self.rotation, &other.rotation) {
            (Some(a), Some(b)) => {
                result.rotation = Some(a.interpolate_step(b, f));
            }
            (None, None) => {}
            (None, Some(b)) => result.rotation = Some(*b),
            (Some(a), None) => result.rotation = Some(*a),
        }

        match (&self.translation, &other.translation) {
            (Some(a), Some(b)) => {
                result.translation = Some(a.interpolate_step(b, f));
            }
            (None, None) => {}
            (None, Some(b)) => result.translation = Some(*b),
            (Some(a), None) => result.translation = Some(*a),
        }

        match (&self.scale, &other.scale) {
            (Some(a), Some(b)) => {
                result.scale = Some(a.interpolate_step(b, f));
            }
            (None, None) => {}
            (None, Some(b)) => result.scale = Some(*b),
            (Some(a), None) => result.scale = Some(*a),
        }

        match (&self.weights, &other.weights) {
            (Some(a), Some(b)) => {
                result.weights = Some(a.interpolate_step(b, f));
            }
            (None, None) => {}
            (None, Some(b)) => result.weights = Some(b.clone()),
            (Some(a), None) => result.weights = Some(a.clone()),
        }

        result
    }
}

impl InterpolateStep for Pose {
    fn interpolate_step(&self, other: &Self, f: f32) -> Self {
        let mut result = Pose::default();

        for (path, bone_id) in self.paths.iter() {
            if let Some(other_bone_id) = other.paths.get(path) {
                result.add_bone(
                    self.bones[*bone_id].interpolate_step(&other.bones[*other_bone_id], f),
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
        // Step interpolation: pick one side's root motion based on threshold
        result.root_motion = if f < 0.5 {
            self.root_motion.clone()
        } else {
            other.root_motion.clone()
        };

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pose::RootMotionDelta;

    fn approx_eq_vec3(a: Vec3, b: Vec3, epsilon: f32) -> bool {
        (a - b).length() < epsilon
    }

    fn make_pose_with_rm(t: Vec3) -> Pose {
        Pose {
            root_motion: Some(RootMotionDelta {
                translation: t,
                rotation: Quat::IDENTITY,
            }),
            ..Pose::default()
        }
    }

    #[test]
    fn test_step_picks_self_below_half() {
        let a = make_pose_with_rm(Vec3::new(1.0, 0.0, 0.0));
        let b = make_pose_with_rm(Vec3::new(0.0, 2.0, 0.0));

        let result = a.interpolate_step(&b, 0.3);
        let rm = result.root_motion.unwrap();
        assert!(approx_eq_vec3(
            rm.translation,
            Vec3::new(1.0, 0.0, 0.0),
            1e-5
        ));
    }

    #[test]
    fn test_step_picks_other_at_or_above_half() {
        let a = make_pose_with_rm(Vec3::new(1.0, 0.0, 0.0));
        let b = make_pose_with_rm(Vec3::new(0.0, 2.0, 0.0));

        let result = a.interpolate_step(&b, 0.5);
        let rm = result.root_motion.unwrap();
        assert!(approx_eq_vec3(
            rm.translation,
            Vec3::new(0.0, 2.0, 0.0),
            1e-5
        ));
    }

    #[test]
    fn test_step_no_root_motion() {
        let a = Pose::default();
        let b = Pose::default();

        let result = a.interpolate_step(&b, 0.5);
        assert!(result.root_motion.is_none());
    }

    #[test]
    fn test_step_mixed_root_motion() {
        let a = make_pose_with_rm(Vec3::new(1.0, 0.0, 0.0));
        let b = Pose::default();

        // f < 0.5: picks self (has root motion)
        let result = a.interpolate_step(&b, 0.3);
        assert!(result.root_motion.is_some());

        // f >= 0.5: picks other (no root motion)
        let result = a.interpolate_step(&b, 0.7);
        assert!(result.root_motion.is_none());
    }
}
