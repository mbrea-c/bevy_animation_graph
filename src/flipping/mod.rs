use crate::core::{
    animation_clip::EntityPath,
    frame::{BoneFrame, InnerPoseFrame, ValueFrame},
};
use bevy::math::prelude::*;

pub trait FlipXBySuffix {
    fn flipped_by_suffix(&self, suffix_1: String, suffix_2: String) -> Self;
}

impl FlipXBySuffix for ValueFrame<Vec3> {
    fn flipped_by_suffix(&self, _suffix_1: String, _suffix_2: String) -> Self {
        let mut out = self.clone();

        out.prev.x *= -1.;
        out.next.x *= -1.;

        out
    }
}

impl FlipXBySuffix for ValueFrame<Quat> {
    fn flipped_by_suffix(&self, _suffix_1: String, _suffix_2: String) -> Self {
        let mut out = self.clone();

        out.prev.x *= -1.;
        out.prev.w *= -1.;
        out.next.x *= -1.;
        out.next.w *= -1.;

        out
    }
}

impl FlipXBySuffix for BoneFrame {
    fn flipped_by_suffix(&self, suffix_1: String, suffix_2: String) -> Self {
        BoneFrame {
            rotation: self
                .rotation
                .clone()
                .map(|v| v.flipped_by_suffix(suffix_1.clone(), suffix_2.clone())),
            translation: self
                .translation
                .clone()
                .map(|v| v.flipped_by_suffix(suffix_1.clone(), suffix_2.clone())),
            scale: self.scale.clone(),
            weights: self.weights.clone(),
        }
    }
}

impl FlipXBySuffix for InnerPoseFrame {
    fn flipped_by_suffix(&self, suffix_1: String, suffix_2: String) -> Self {
        let mut out = InnerPoseFrame::default();
        for (path, bone_id) in self.paths.iter() {
            let channel =
                self.bones[*bone_id].flipped_by_suffix(suffix_1.clone(), suffix_2.clone());
            let new_path = EntityPath {
                parts: path
                    .parts
                    .iter()
                    .map(|part| {
                        let mut part = part.to_string();
                        if part.ends_with(&suffix_1) {
                            part = part.strip_suffix(&suffix_1).unwrap().into();
                            part.push_str(&suffix_2);
                        } else if part.ends_with(&suffix_2) {
                            part = part.strip_suffix(&suffix_2).unwrap().into();
                            part.push_str(&suffix_1);
                        }
                        part.into()
                    })
                    .collect(),
            };

            out.add_bone(channel, new_path);
        }
        out.timestamp = self.timestamp;

        out
    }
}
