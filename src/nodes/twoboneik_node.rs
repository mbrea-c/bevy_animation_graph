use bevy::{
    math::{Quat, Vec3},
    reflect::Reflect,
    transform::components::Transform,
    utils::HashMap,
};

use crate::{
    core::{
        animation_clip::EntityPath,
        animation_graph::{PinId, TimeUpdate},
        animation_node::{AnimationNode, AnimationNodeType, NodeLike},
        duration_data::DurationData,
        frame::{BonePoseFrame, PoseFrame, PoseFrameData, PoseSpec},
        space_conversion::SpaceConversion,
    },
    prelude::{OptParamSpec, ParamSpec, PassContext, SampleLinearAt, SpecContext},
    utils::unwrap::Unwrap,
};

#[derive(Reflect, Clone, Debug, Default)]
pub struct TwoBoneIKNode {}

impl TwoBoneIKNode {
    pub const INPUT: &'static str = "Pose In";
    pub const TARGETBONE: &'static str = "Target Path";
    pub const TARGETPOS: &'static str = "Target Position";

    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::TwoBoneIK(self))
    }
}

impl NodeLike for TwoBoneIKNode {
    fn duration_pass(&self, mut ctx: PassContext) -> Option<DurationData> {
        Some(ctx.duration_back(Self::INPUT))
    }

    fn pose_pass(&self, input: TimeUpdate, mut ctx: PassContext) -> Option<PoseFrame> {
        let target: EntityPath = ctx.parameter_back(Self::TARGETBONE).unwrap();
        let target_pos_char: Vec3 = ctx.parameter_back(Self::TARGETPOS).unwrap();
        //let targetrotation: Quat = ctx.parameter_back(Self::TARGETROT).unwrap();
        let pose = ctx.pose_back(Self::INPUT, input);
        let mut bone_pose_data: BonePoseFrame = pose.data.unwrap();
        let inner_pose_data = bone_pose_data.inner_mut();

        for (bone_path, bone_id) in inner_pose_data.paths.iter() {
            if *bone_path == target {
                let bone = inner_pose_data.bones[*bone_id].clone();
                if let Some(parent_path) = bone_path.parent() {
                    if let Some(grandparent_path) = parent_path.parent() {
                        let target_gp = ctx.root_to_bone_space(
                            Transform::from_translation(target_pos_char),
                            inner_pose_data,
                            grandparent_path.parent().unwrap().clone(),
                            pose.timestamp,
                        );

                        let target_pos_gp = target_gp.translation;

                        let parent_id = inner_pose_data.paths.get(&parent_path).unwrap();
                        let parent_frame = {
                            let parent_bone = inner_pose_data.bones.get_mut(*parent_id).unwrap();
                            parent_bone.to_transform_frame_linear()
                        };
                        let parent_transform = parent_frame.sample_linear_at(pose.timestamp);

                        let grandparent_id = inner_pose_data.paths.get(&grandparent_path).unwrap();
                        let grandparent_bone =
                            inner_pose_data.bones.get_mut(*grandparent_id).unwrap();
                        let grandparent_frame = grandparent_bone.to_transform_frame_linear();
                        let grandparent_transform =
                            grandparent_frame.sample_linear_at(pose.timestamp);

                        let bone_frame = bone.to_transform_frame_linear();
                        let bone_transform = bone_frame.sample_linear_at(pose.timestamp);

                        let parent_gp_transform = grandparent_transform * parent_transform;
                        let bone_gp_transform = parent_gp_transform * bone_transform;

                        let (bone_gp_transform, parent_gp_transform, grandparent_transform) =
                            two_bone_ik(
                                bone_gp_transform,
                                parent_gp_transform,
                                grandparent_transform,
                                target_pos_gp,
                            );

                        let parent_transform = Transform::from_matrix(
                            grandparent_transform.compute_matrix().inverse(),
                        ) * parent_gp_transform;
                        let bone_transform =
                            Transform::from_matrix(parent_gp_transform.compute_matrix().inverse())
                                * bone_gp_transform;

                        inner_pose_data.bones[*grandparent_id]
                            .rotation
                            .as_mut()
                            .unwrap()
                            .map_mut(|_| grandparent_transform.rotation);

                        inner_pose_data.bones[*parent_id]
                            .rotation
                            .as_mut()
                            .unwrap()
                            .map_mut(|_| parent_transform.rotation);

                        inner_pose_data.bones[*bone_id]
                            .rotation
                            .as_mut()
                            .unwrap()
                            .map_mut(|_| bone_transform.rotation);
                    }
                }
            }
        }

        Some(PoseFrame {
            data: PoseFrameData::BoneSpace(bone_pose_data),
            timestamp: pose.timestamp,
        })
    }

    fn parameter_input_spec(&self, _: SpecContext) -> HashMap<PinId, OptParamSpec> {
        HashMap::from([
            (Self::TARGETBONE.into(), ParamSpec::EntityPath.into()),
            (Self::TARGETPOS.into(), ParamSpec::Vec3.into()),
        ])
    }

    fn pose_input_spec(&self, _: SpecContext) -> HashMap<PinId, PoseSpec> {
        HashMap::from([(Self::INPUT.into(), PoseSpec::BoneSpace)])
    }

    fn pose_output_spec(&self, _: SpecContext) -> Option<PoseSpec> {
        Some(PoseSpec::BoneSpace)
    }

    fn display_name(&self) -> String {
        "Two Bone IK".into()
    }
}

fn two_bone_ik(
    bone: Transform,
    parent: Transform,
    grandparent: Transform,
    target_pos: Vec3,
) -> (Transform, Transform, Transform) {
    const MAX_LEN_OFFSET: f32 = 0.01;

    // compute joint positions
    let in_end_loc = bone.translation;
    let in_mid_loc = parent.translation;
    let in_root_loc = grandparent.translation;

    // compute bone lengths
    let upper_len = in_root_loc.distance(in_mid_loc);
    let lower_len = in_mid_loc.distance(in_end_loc);
    let max_len = upper_len + lower_len - MAX_LEN_OFFSET;

    // compute input planar basis vectors
    let to_end = (in_end_loc - in_root_loc).normalize();
    let in_pole_vec = (in_mid_loc - in_root_loc).reject_from(to_end).normalize();

    // compute final planar basis vectors
    let to_target_offset = (target_pos - in_root_loc).clamp_length_max(max_len);
    let to_target_dist = to_target_offset.length();
    let to_target = to_target_offset / to_target_dist;

    let to_target_swing = Quat::from_rotation_arc(to_end, to_target);
    let out_pole_vec = to_target_swing * in_pole_vec;

    // apply law of cosines to get middle joint angle
    let denom = 2. * upper_len * to_target_dist;
    let mut cos_angle = 0.;
    if denom > f32::EPSILON {
        cos_angle = (to_target_dist * to_target_dist + upper_len * upper_len
            - lower_len * lower_len)
            / denom;
    }
    let angle = cos_angle.acos();

    // compute final joint positions
    let pole_dist = upper_len * angle.sin();
    let eff_dist = upper_len * cos_angle;
    let out_end_loc = in_root_loc + to_target_offset;
    let out_mid_loc = in_root_loc + eff_dist * to_target + pole_dist * out_pole_vec;

    // compute final rotations
    let in_to_mid = in_mid_loc - in_root_loc;
    let out_to_mid = out_mid_loc - in_root_loc;
    let root_swing = Quat::from_rotation_arc(in_to_mid.normalize(), out_to_mid.normalize());
    let in_end_loc_with_root_swing = in_root_loc + root_swing * (in_end_loc - in_root_loc);
    let to_in_end = in_end_loc_with_root_swing - out_mid_loc;
    let to_out_end = out_end_loc - out_mid_loc;
    let mid_swing =
        Quat::from_rotation_arc(to_in_end.normalize(), to_out_end.normalize()) * root_swing;

    // set up output transforms
    let out_grandparent = Transform {
        rotation: root_swing * grandparent.rotation,
        ..grandparent
    };

    let out_parent = Transform {
        translation: out_mid_loc,
        rotation: mid_swing * parent.rotation,
        ..parent
    };
    let out_bone = Transform {
        translation: out_end_loc,
        rotation: mid_swing * bone.rotation,
        ..bone
    };

    (out_bone, out_parent, out_grandparent)
}
