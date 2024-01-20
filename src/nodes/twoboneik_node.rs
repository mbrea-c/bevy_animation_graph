use bevy::{
    math::{Quat, Vec3},
    reflect::Reflect,
    utils::HashMap,
};

use crate::{
    core::{
        animation_clip::EntityPath,
        animation_graph::{PinId, TimeUpdate},
        animation_node::{AnimationNode, AnimationNodeType, NodeLike},
        duration_data::DurationData,
        frame::{CharacterPoseFrame, PoseFrame, PoseFrameData, PoseSpec, BonePoseFrame},
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
    pub const TARGETROT: &'static str = "Target rotation of the target bone";

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
        let targetposition: Vec3 = ctx.parameter_back(Self::TARGETPOS).unwrap();
        //let targetrotation: Quat = ctx.parameter_back(Self::TARGETROT).unwrap();
        let pose = ctx.pose_back(Self::INPUT, input);
        let mut bone_pose_data: BonePoseFrame = pose.data.unwrap();
        let mut inner_pose_data = bone_pose_data.inner_mut();

        for (bone_path, bone_id) in inner_pose_data.paths.iter() {
            if *bone_path == target {
                let bone = inner_pose_data.bones[*bone_id].clone();
                if let Some(parent_path) = bone_path.parent() {
                    if let Some(grandparent_path) = parent_path.parent() {
                        // NOTE : targetposition and rotation need to be in the frame of reference used for that
                        // a is gp, b is p, c is target bones current state, t is target state
                        let parent_id = inner_pose_data.paths.get(&parent_path).unwrap();
                        let parent_frame = {
                            let parent_bone = inner_pose_data.bones.get_mut(*parent_id).unwrap();
                            parent_bone.to_transform_frame_linear()
                        };
                        let mut parent_transform = parent_frame.sample_linear_at(pose.timestamp);

                        let grandparent_id = inner_pose_data.paths.get(&grandparent_path).unwrap();
                        let grandparent_bone =
                            inner_pose_data.bones.get_mut(*grandparent_id).unwrap();
                        let grandparent_frame = grandparent_bone.to_transform_frame_linear();
                        let mut grandparent_transform =
                            grandparent_frame.sample_linear_at(pose.timestamp);

                        let bone_frame = bone.to_transform_frame_linear();
                        let bone_transform = bone_frame.sample_linear_at(pose.timestamp);

                        // TODO : transform the local space to grandparent space for the calculations
                        let parent_gp_transform = grandparent_transform * parent_transform;
                        let bone_gp_transform = parent_gp_transform * bone_transform;

                        let eps = 0.01;
                        let length_gp_p = (parent_gp_transform.translation
                            - grandparent_transform.translation)
                            .length();
                        let length_targetcurr_p =
                            (parent_gp_transform.translation - bone_gp_transform.translation).length();
                        let length_gp_target = (targetposition - grandparent_transform.translation)
                            .length()
                            .clamp(eps, length_gp_p + length_targetcurr_p - eps);

                        //get current interior angles
                        let curr_gp_int = (bone_gp_transform.translation
                            - grandparent_transform.translation)
                            .normalize()
                            .dot(
                                (parent_gp_transform.translation - grandparent_transform.translation)
                                    .normalize(),
                            )
                            .clamp(-1., 1.)
                            .acos();

                        let curr_p_int = (grandparent_transform.translation
                            - parent_gp_transform.translation)
                            .normalize()
                            .dot(
                                (bone_gp_transform.translation - parent_gp_transform.translation)
                                    .normalize(),
                            )
                            .clamp(-1., 1.)
                            .acos();

                        let curr_gp_target_int = (bone_gp_transform.translation
                            - grandparent_transform.translation)
                            .normalize()
                            .dot((targetposition - grandparent_transform.translation).normalize())
                            .clamp(-1., 1.)
                            .acos();

                        // get desired interior angles
                        let des_gp_int = (length_targetcurr_p * length_targetcurr_p
                            - length_gp_p * length_gp_p
                            - length_gp_target * length_gp_target)
                            / (-2. * length_gp_p * length_gp_target).clamp(-1., 1.).acos();
                        let des_p_int = (length_gp_target * length_gp_target
                            - length_gp_p * length_gp_p
                            - length_targetcurr_p * length_targetcurr_p)
                            / (-2. * length_gp_p * length_targetcurr_p)
                                .clamp(-1., 1.)
                                .acos();

                        // rotation axis and angles, gr are global rotations
                        // TODO check with formula
                        let axis0 = (bone_gp_transform.translation
                            - grandparent_transform.translation.cross(
                                parent_gp_transform.translation - grandparent_transform.translation,
                            ))
                        .normalize();

                        let axis1 = (bone_gp_transform.translation
                            - grandparent_transform.translation)
                            .cross(targetposition - grandparent_transform.translation)
                            .normalize();

                        let inverse_gp_global = grandparent_transform.rotation.inverse();
                        let inverse_p_global = parent_gp_transform.rotation.inverse();
                        let r0 = Quat::from_axis_angle(
                            (inverse_gp_global * axis0).normalize(),
                            des_gp_int - curr_gp_int,
                        );
                        let r1 = Quat::from_axis_angle(
                            (inverse_p_global * axis0).normalize(),
                            des_p_int - curr_p_int,
                        );

                        let r2 = Quat::from_axis_angle(
                            (inverse_gp_global * axis1).normalize(),
                            curr_gp_target_int,
                        );

                        // set grandparent and parent rotations
                        grandparent_transform.rotation = grandparent_transform.rotation * (r0 * r2);
                        parent_transform.rotation = parent_transform.rotation * r1;

                        grandparent_bone
                            .rotation
                            .as_mut()
                            .unwrap()
                            .map_mut(|_| grandparent_transform.rotation);

                        inner_pose_data.bones[*parent_id]
                            .rotation
                            .as_mut()
                            .unwrap()
                            .map_mut(|_| parent_transform.rotation);
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
        "TwoBoneIK".into()
    }
}
