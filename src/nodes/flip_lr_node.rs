use crate::core::animation_graph::{PinId, TimeUpdate};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::duration_data::DurationData;
use crate::core::frame::InnerPoseFrame;
use crate::flipping::FlipXBySuffix;
use crate::prelude::{PassContext, SpecContext};
use bevy::prelude::*;
use bevy::utils::HashSet;

#[derive(Reflect, Clone, Debug)]
pub struct FlipLRNode {}

impl Default for FlipLRNode {
    fn default() -> Self {
        Self::new()
    }
}

impl FlipLRNode {
    pub const INPUT: &'static str = "Pose In";
    pub const OUTPUT: &'static str = "Pose Out";

    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::FlipLR(self))
    }
}

impl NodeLike for FlipLRNode {
    fn duration_pass(&self, mut ctx: PassContext) -> Option<DurationData> {
        Some(ctx.duration_back(Self::INPUT))
    }

    fn pose_pass(&self, input: TimeUpdate, mut ctx: PassContext) -> Option<InnerPoseFrame> {
        let in_pose_frame = ctx.pose_back(Self::INPUT, input);
        let flipped_pose_frame = in_pose_frame.flipped_by_suffix("R".into(), "L".into());
        Some(flipped_pose_frame)
    }

    fn pose_input_spec(&self, _: SpecContext) -> HashSet<PinId> {
        HashSet::from([Self::INPUT.into()])
    }

    fn pose_output_spec(&self, _: SpecContext) -> bool {
        true
    }

    fn display_name(&self) -> String {
        "🯈|🯇 Flip Left/Right".into()
    }
}
