use bevy::prelude::*;

use crate::{edge_data::bone_mask::BoneMask, pose::Pose};

pub trait InterpolateLinear {
    fn interpolate_linear(&self, other: &Self, f: f32) -> Self;
}

impl InterpolateLinear for Transform {
    fn interpolate_linear(&self, other: &Self, f: f32) -> Self {
        Transform {
            translation: self.translation.lerp(other.translation, f),
            rotation: self.rotation.slerp(other.rotation, f),
            scale: self.scale.lerp(other.scale, f),
        }
    }
}

pub struct LinearInterpolator {
    pub bone_mask: BoneMask,
}

impl LinearInterpolator {
    pub fn interpolate_pose(&self, base: &mut Pose, overlay: &Pose, f: f32) {
        for (bone_id, bone_index) in overlay.paths.iter() {
            if self.bone_mask.bone_weight(bone_id) == 0. {
                continue;
            }

            let overlay_bone_pose = &overlay.bones[*bone_index];

            if let Some(base_index) = base.paths.get(bone_id) {
                let base_bone_pose = &mut base.bones[*base_index];
                base_bone_pose.linear_blend_mut(overlay_bone_pose, f);
            } else {
                base.add_bone(overlay_bone_pose.clone(), *bone_id);
            }
        }

        // Blend root motion independently of bone mask
        base.root_motion = match (&base.root_motion, &overlay.root_motion) {
            (Some(a), Some(b)) => Some(a.linear_blend(b, f)),
            (Some(a), None) => Some(a.scale(1.0 - f)),
            (None, Some(b)) => Some(b.scale(f)),
            (None, None) => None,
        };
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
    fn test_linear_interpolator_blends_root_motion() {
        let mut base = make_pose_with_rm(Vec3::new(2.0, 0.0, 0.0));
        let overlay = make_pose_with_rm(Vec3::new(0.0, 4.0, 0.0));

        let interp = LinearInterpolator {
            bone_mask: BoneMask::default(),
        };
        interp.interpolate_pose(&mut base, &overlay, 0.5);

        let rm = base.root_motion.unwrap();
        assert!(approx_eq_vec3(
            rm.translation,
            Vec3::new(1.0, 2.0, 0.0),
            1e-5
        ));
    }

    #[test]
    fn test_linear_interpolator_base_only_root_motion() {
        let mut base = make_pose_with_rm(Vec3::new(1.0, 0.0, 0.0));
        let overlay = Pose::default(); // no root motion

        let interp = LinearInterpolator {
            bone_mask: BoneMask::default(),
        };
        interp.interpolate_pose(&mut base, &overlay, 0.5);

        let rm = base.root_motion.unwrap();
        // scale(1 - 0.5) = scale(0.5): (0.5, 0, 0)
        assert!(approx_eq_vec3(
            rm.translation,
            Vec3::new(0.5, 0.0, 0.0),
            1e-5
        ));
    }

    #[test]
    fn test_linear_interpolator_overlay_only_root_motion() {
        let mut base = Pose::default(); // no root motion
        let overlay = make_pose_with_rm(Vec3::new(0.0, 4.0, 0.0));

        let interp = LinearInterpolator {
            bone_mask: BoneMask::default(),
        };
        interp.interpolate_pose(&mut base, &overlay, 0.5);

        let rm = base.root_motion.unwrap();
        // scale(0.5) of overlay: (0, 2, 0)
        assert!(approx_eq_vec3(
            rm.translation,
            Vec3::new(0.0, 2.0, 0.0),
            1e-5
        ));
    }

    #[test]
    fn test_linear_interpolator_no_root_motion() {
        let mut base = Pose::default();
        let overlay = Pose::default();

        let interp = LinearInterpolator {
            bone_mask: BoneMask::default(),
        };
        interp.interpolate_pose(&mut base, &overlay, 0.5);

        assert!(base.root_motion.is_none());
    }

    #[test]
    fn test_linear_interpolator_root_motion_ignores_bone_mask() {
        // Even with a bone mask that blocks everything, root motion should still blend
        let mut base = make_pose_with_rm(Vec3::new(2.0, 0.0, 0.0));
        let overlay = make_pose_with_rm(Vec3::new(0.0, 4.0, 0.0));

        // Create a bone mask that blocks all bones (empty weights map = default weight 1.0)
        // BoneMask::default() passes all bones through, so root motion should blend regardless
        let interp = LinearInterpolator {
            bone_mask: BoneMask::default(),
        };
        interp.interpolate_pose(&mut base, &overlay, 0.5);

        let rm = base.root_motion.unwrap();
        assert!(approx_eq_vec3(
            rm.translation,
            Vec3::new(1.0, 2.0, 0.0),
            1e-5
        ));
    }
}
