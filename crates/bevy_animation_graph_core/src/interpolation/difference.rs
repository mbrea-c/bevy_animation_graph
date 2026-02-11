use crate::{edge_data::bone_mask::BoneMask, pose::Pose};

pub struct DifferenceInterpolator {
    pub bone_mask: BoneMask,
}

impl DifferenceInterpolator {
    pub fn interpolate_pose(&self, base: &mut Pose, overlay: &Pose) {
        for (bone_id, bone_index) in overlay.paths.iter() {
            if self.bone_mask.bone_weight(&bone_id) == 0. {
                continue;
            }

            let overlay_bone_pose = &overlay.bones[*bone_index];

            if let Some(base_index) = base.paths.get(bone_id) {
                let base_bone_pose = &mut base.bones[*base_index];
                base_bone_pose.difference_mut(overlay_bone_pose);
            } else {
                base.add_bone(overlay_bone_pose.clone(), *bone_id);
            }
        }
    }
}
