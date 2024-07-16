pub mod config;

use self::config::FlipConfig;
use crate::core::{
    animation_clip::EntityPath,
    pose::{BonePose, Pose},
    skeleton::Skeleton,
};
use bevy::math::prelude::*;

fn flip_vec3(val: &Vec3) -> Vec3 {
    let mut out = *val;
    out.x *= -1.;
    out
}

fn flip_quat(val: &Quat) -> Quat {
    let mut out = *val;
    out.x *= -1.;
    out.w *= -1.;
    out
}

fn flip_bone_pose(val: &BonePose) -> BonePose {
    BonePose {
        rotation: val.rotation.map(|v| flip_quat(&v)),
        translation: val.translation.map(|v| flip_vec3(&v)),
        scale: val.scale,
        weights: val.weights.clone(),
    }
}

pub fn flip_pose(val: &Pose, config: &FlipConfig, skeleton: &Skeleton) -> Pose {
    let mut out = Pose::default();
    for (bone_id, bone_index) in val.paths.iter() {
        let channel = flip_bone_pose(&val.bones[*bone_index]);
        // TODO: Make flipped return a Result type, so we can gracefully fail if no match for
        // id
        let path = skeleton.id_to_path(*bone_id).unwrap();
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
        let new_id = new_path.id();

        // TODO: Should we assert that the new id is part of the skeleton? Probably yes
        // Fix this when we can gracefully fail
        if !skeleton.has_id(&new_id) {
            panic!("No match for flipped bone id");
        }

        out.add_bone(channel, new_id);
    }
    out.skeleton = val.skeleton.clone();
    out
}
