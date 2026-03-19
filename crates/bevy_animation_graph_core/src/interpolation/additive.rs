use crate::{
    edge_data::bone_mask::BoneMask,
    pose::{Pose, RootMotionDelta},
};

pub struct AdditiveInterpolator {
    pub bone_mask: BoneMask,
}

impl AdditiveInterpolator {
    pub fn interpolate_pose(&self, base: &mut Pose, overlay: &Pose, f: f32) {
        for (bone_id, bone_index) in overlay.paths.iter() {
            if self.bone_mask.bone_weight(bone_id) == 0. {
                continue;
            }

            let overlay_bone_pose = &overlay.bones[*bone_index];

            if let Some(base_index) = base.paths.get(bone_id) {
                let base_bone_pose = &mut base.bones[*base_index];
                base_bone_pose.additive_blend_mut(overlay_bone_pose, f);
            } else {
                base.add_bone(overlay_bone_pose.clone(), *bone_id);
            }
        }

        // Blend root motion independently of bone mask
        base.root_motion = match (&base.root_motion, &overlay.root_motion) {
            (Some(a), Some(b)) => Some(a.additive_blend(b, f)),
            (Some(a), None) => Some(a.clone()),
            (None, Some(b)) => Some(RootMotionDelta::default().additive_blend(b, f)),
            (None, None) => None,
        };
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::{Quat, Vec3};

    use super::*;

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
    fn test_additive_interpolator_blends_root_motion() {
        let mut base = make_pose_with_rm(Vec3::new(1.0, 0.0, 0.0));
        let overlay = make_pose_with_rm(Vec3::new(0.0, 2.0, 0.0));

        let interp = AdditiveInterpolator {
            bone_mask: BoneMask::default(),
        };
        interp.interpolate_pose(&mut base, &overlay, 0.5);

        let rm = base.root_motion.unwrap();
        // additive: base + alpha * overlay = (1,0,0) + 0.5*(0,2,0) = (1,1,0)
        assert!(approx_eq_vec3(
            rm.translation,
            Vec3::new(1.0, 1.0, 0.0),
            1e-5
        ));
    }

    #[test]
    fn test_additive_interpolator_base_only_root_motion() {
        let mut base = make_pose_with_rm(Vec3::new(1.0, 0.0, 0.0));
        let overlay = Pose::default();

        let interp = AdditiveInterpolator {
            bone_mask: BoneMask::default(),
        };
        interp.interpolate_pose(&mut base, &overlay, 0.5);

        let rm = base.root_motion.unwrap();
        assert!(approx_eq_vec3(
            rm.translation,
            Vec3::new(1.0, 0.0, 0.0),
            1e-5
        ));
    }

    #[test]
    fn test_additive_interpolator_overlay_only_root_motion() {
        let mut base = Pose::default();
        let overlay = make_pose_with_rm(Vec3::new(0.0, 4.0, 0.0));

        let interp = AdditiveInterpolator {
            bone_mask: BoneMask::default(),
        };
        interp.interpolate_pose(&mut base, &overlay, 0.5);

        let rm = base.root_motion.unwrap();
        // default + 0.5 * overlay = (0,0,0) + (0,2,0) = (0,2,0)
        assert!(approx_eq_vec3(
            rm.translation,
            Vec3::new(0.0, 2.0, 0.0),
            1e-5
        ));
    }

    #[test]
    fn test_additive_interpolator_no_root_motion() {
        let mut base = Pose::default();
        let overlay = Pose::default();

        let interp = AdditiveInterpolator {
            bone_mask: BoneMask::default(),
        };
        interp.interpolate_pose(&mut base, &overlay, 0.5);

        assert!(base.root_motion.is_none());
    }
}
