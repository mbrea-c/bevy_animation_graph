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
    }
}
