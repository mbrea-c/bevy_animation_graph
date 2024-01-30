use crate::core::animation_graph::{PinMap, TimeUpdate};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::duration_data::DurationData;
use crate::core::frame::{BonePoseFrame, PoseFrame, PoseFrameData, PoseSpec};
use crate::core::parameters::BoneMask;
use crate::prelude::{OptParamSpec, ParamSpec, PassContext, SpecContext};
use crate::utils::unwrap::Unwrap;
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug)]
#[reflect(Default)]
pub struct RotationNode {}

impl Default for RotationNode {
    fn default() -> Self {
        Self::new()
    }
}

impl RotationNode {
    pub const INPUT: &'static str = "Pose In";
    pub const MASK: &'static str = "Bone Mask";
    pub const ROTATION: &'static str = "Rotation";
    pub const OUTPUT: &'static str = "Pose Out";

    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::Rotation(self))
    }
}

impl NodeLike for RotationNode {
    fn duration_pass(&self, mut ctx: PassContext) -> Option<DurationData> {
        Some(ctx.duration_back(Self::INPUT))
    }

    fn pose_pass(&self, input: TimeUpdate, mut ctx: PassContext) -> Option<PoseFrame> {
        let mask: BoneMask = ctx.parameter_back(Self::MASK).unwrap();
        let rotation: Quat = ctx.parameter_back(Self::ROTATION).unwrap();
        let pose = ctx.pose_back(Self::INPUT, input);
        let time = pose.timestamp;
        let mut pose: BonePoseFrame = pose.data.unwrap();
        let inner_pose = pose.inner_mut();

        for (bone_id, idx) in inner_pose.paths.iter() {
            let percent = mask.bone_weight(bone_id);
            if percent == 0. {
                continue;
            }

            let bone = inner_pose.bones.get_mut(*idx).unwrap();
            if let Some(rot) = bone.rotation.as_mut() {
                let rotation = (rotation * percent).normalize();
                rot.prev = rotation * rot.prev;
                rot.next = rotation * rot.next;
            }
        }

        Some(PoseFrame {
            data: PoseFrameData::BoneSpace(pose),
            timestamp: time,
        })
    }

    fn parameter_input_spec(&self, _ctx: SpecContext) -> PinMap<OptParamSpec> {
        [
            (Self::MASK.into(), ParamSpec::BoneMask.into()),
            (Self::ROTATION.into(), ParamSpec::Quat.into()),
        ]
        .into()
    }

    fn pose_input_spec(&self, _: SpecContext) -> PinMap<PoseSpec> {
        [(Self::INPUT.into(), PoseSpec::BoneSpace)].into()
    }

    fn pose_output_spec(&self, _: SpecContext) -> Option<PoseSpec> {
        Some(PoseSpec::BoneSpace)
    }

    fn display_name(&self) -> String {
        "⮩ Rotation".into()
    }
}
