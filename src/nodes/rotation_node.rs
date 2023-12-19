use crate::core::animation_graph::{PinId, TimeUpdate};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::duration_data::DurationData;
use crate::core::frame::PoseFrame;
use crate::core::parameters::BoneMask;
use crate::prelude::{OptParamSpec, ParamSpec, PassContext, SpecContext};
use crate::utils::unwrap::Unwrap;
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};

#[derive(Reflect, Clone, Debug)]
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
        let mut pose = ctx.pose_back(Self::INPUT, input);

        for (bone_id, idx) in pose.paths.iter() {
            let percent = mask.bone_weight(bone_id);
            if percent == 0. {
                continue;
            }

            let bone = pose.bones.get_mut(*idx).unwrap();
            if let Some(rot) = bone.rotation.as_mut() {
                // TODO: rotation needs to be scaled by `percent`
                rot.prev *= rotation;
                rot.next *= rotation;
            }
        }

        Some(pose)
    }

    fn parameter_input_spec(&self, _ctx: SpecContext) -> HashMap<PinId, OptParamSpec> {
        HashMap::from([
            (Self::MASK.into(), ParamSpec::BoneMask.into()),
            (Self::ROTATION.into(), ParamSpec::Quat.into()),
        ])
    }

    fn pose_input_spec(&self, _: SpecContext) -> HashSet<PinId> {
        HashSet::from([Self::INPUT.into()])
    }

    fn pose_output_spec(&self, _: SpecContext) -> bool {
        true
    }

    fn display_name(&self) -> String {
        "󰶘 Rotation".into()
    }
}
