pub mod config;
pub mod serial;

use self::config::SymmetryConfig;
use crate::core::{
    pose::{BonePose, Pose},
    skeleton::Skeleton,
};

fn flip_bone_pose(val: &BonePose, config: &SymmetryConfig) -> BonePose {
    BonePose {
        rotation: val.rotation.map(|v| config.mode.apply_quat(v)),
        translation: val.translation.map(|v| config.mode.apply_position(v)),
        scale: val.scale,
        weights: val.weights.clone(),
    }
}

pub fn flip_pose(val: &Pose, config: &SymmetryConfig, skeleton: &Skeleton) -> Pose {
    let mut out = Pose::default();
    for (bone_id, bone_index) in val.paths.iter() {
        let channel = flip_bone_pose(&val.bones[*bone_index], config);
        // TODO: Make flipped return a Result type, so we can gracefully fail if no match for
        // id
        let path = skeleton.id_to_path(*bone_id).unwrap();
        let new_path = config.name_mapper.flip(&path);
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
