use crate::{edge_data::bone_mask::BoneMask, pose::Pose};

pub struct AdditiveInterpolator {
    pub bone_mask: BoneMask,
}

impl AdditiveInterpolator {
    pub fn interpolate_pose(&self, base: &mut Pose, overlay: &Pose, f: f32) {
        for (bone_id, bone_index) in overlay.paths.iter() {
            let bone_weight = self.bone_mask.bone_weight(bone_id);
            if bone_weight == 0. {
                continue;
            }

            let scaled_f = f * bone_weight;
            let overlay_bone_pose = &overlay.bones[*bone_index];

            if let Some(base_index) = base.paths.get(bone_id) {
                let base_bone_pose = &mut base.bones[*base_index];
                base_bone_pose.additive_blend_mut(overlay_bone_pose, scaled_f);
            } else {
                base.add_bone(overlay_bone_pose.clone(), *bone_id);
            }
        }
    }
}
