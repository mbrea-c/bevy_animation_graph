pub mod config;

use self::config::FlipConfig;
use crate::core::{
    animation_clip::EntityPath,
    pose::{BonePose, Pose},
};
use bevy::math::prelude::*;

pub trait FlipXBySuffix {
    fn flipped(&self, config: &FlipConfig) -> Self;
}

impl FlipXBySuffix for Vec3 {
    fn flipped(&self, _: &FlipConfig) -> Self {
        let mut out = *self;
        out.x *= -1.;
        out
    }
}

impl FlipXBySuffix for Quat {
    fn flipped(&self, _: &FlipConfig) -> Self {
        let mut out = *self;
        out.x *= -1.;
        out.w *= -1.;
        out
    }
}

impl FlipXBySuffix for BonePose {
    fn flipped(&self, config: &FlipConfig) -> Self {
        BonePose {
            rotation: self.rotation.clone().map(|v| v.flipped(config)),
            translation: self.translation.clone().map(|v| v.flipped(config)),
            scale: self.scale.clone(),
            weights: self.weights.clone(),
        }
    }
}

impl FlipXBySuffix for Pose {
    fn flipped(&self, config: &FlipConfig) -> Self {
        let mut out = Pose::default();
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
