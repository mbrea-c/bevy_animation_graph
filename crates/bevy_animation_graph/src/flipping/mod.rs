pub mod config;

use self::config::FlipConfig;
use crate::core::{
    animation_clip::EntityPath,
    frame::{BoneFrame, BonePoseFrame, InnerPoseFrame, ValueFrame},
};
use bevy::math::prelude::*;

pub trait FlipXBySuffix {
    fn flipped(&self, config: &FlipConfig) -> Self;
}

impl FlipXBySuffix for ValueFrame<Vec3> {
    fn flipped(&self, _: &FlipConfig) -> Self {
        let mut out = self.clone();

        out.prev.x *= -1.;
        out.next.x *= -1.;

        out
    }
}

impl FlipXBySuffix for ValueFrame<Quat> {
    fn flipped(&self, _: &FlipConfig) -> Self {
        let mut out = self.clone();

        out.prev.x *= -1.;
        out.prev.w *= -1.;
        out.next.x *= -1.;
        out.next.w *= -1.;

        out
    }
}

impl FlipXBySuffix for BoneFrame {
    fn flipped(&self, config: &FlipConfig) -> Self {
        BoneFrame {
            rotation: self.rotation.clone().map(|v| v.flipped(config)),
            translation: self.translation.clone().map(|v| v.flipped(config)),
            scale: self.scale.clone(),
            weights: self.weights.clone(),
        }
    }
}

impl FlipXBySuffix for InnerPoseFrame {
    fn flipped(&self, config: &FlipConfig) -> Self {
        let mut out = InnerPoseFrame::default();
        for (path, bone_id) in self.paths.iter() {
            let channel = self.bones[*bone_id].flipped(config);
            let new_path = EntityPath {
                parts: path
                    .parts
                    .iter()
                    .map(|part| {
                        let mut part = part.to_string();
                        if let Some(flipped) = config.name_mapper.flip(&part) {
                            part = flipped;
                        }
                        part.into()
                    })
                    .collect(),
            };

            out.add_bone(channel, new_path);
        }
        out
    }
}

impl FlipXBySuffix for BonePoseFrame {
    fn flipped(&self, config: &FlipConfig) -> Self {
        BonePoseFrame(self.0.flipped(config))
    }
}
