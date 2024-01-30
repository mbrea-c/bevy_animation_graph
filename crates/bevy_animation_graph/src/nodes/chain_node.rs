use crate::chaining::Chainable;
use crate::core::animation_graph::{PinId, PinMap, TimeUpdate};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::duration_data::DurationData;
use crate::core::frame::{PoseFrame, PoseSpec};
use crate::prelude::{PassContext, SpecContext};
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub struct ChainNode {}

impl ChainNode {
    pub const INPUT_1: &'static str = "Pose In 1";
    pub const INPUT_2: &'static str = "Pose In 2";
    pub const OUTPUT: &'static str = "Pose Out";

    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::Chain(self))
    }
}

impl NodeLike for ChainNode {
    fn duration_pass(&self, mut ctx: PassContext) -> Option<DurationData> {
        let source_duration_1 = ctx.duration_back(Self::INPUT_1);
        let source_duration_2 = ctx.duration_back(Self::INPUT_2);

        let out_duration = match (source_duration_1, source_duration_2) {
            (Some(duration_1), Some(duration_2)) => Some(duration_1 + duration_2),
            (Some(_), None) => None,
            (None, Some(_)) => None,
            (None, None) => None,
        };

        Some(out_duration)
    }

    fn pose_pass(&self, input: TimeUpdate, mut ctx: PassContext) -> Option<PoseFrame> {
        let duration_1 = ctx.duration_back(Self::INPUT_1);
        let Some(duration_1) = duration_1 else {
            // First input is infinite, forward time update without change
            return Some(ctx.pose_back(Self::INPUT_1, input));
        };

        let pose_1 = ctx.pose_back(Self::INPUT_1, input);
        let curr_time = pose_1.timestamp;

        let pose_2 = ctx.pose_back(Self::INPUT_2, TimeUpdate::Absolute(curr_time - duration_1));

        let duration_2 = ctx.duration_back(Self::INPUT_2);

        let out_pose = pose_1.chain(
            &pose_2,
            duration_1,
            duration_2.unwrap_or(f32::MAX),
            curr_time,
        );

        Some(out_pose)
    }

    fn pose_input_spec(&self, _: SpecContext) -> PinMap<PoseSpec> {
        [
            (Self::INPUT_1.into(), PoseSpec::Any),
            (Self::INPUT_2.into(), PoseSpec::Any),
        ]
        .into()
    }

    fn pose_output_spec(&self, _: SpecContext) -> Option<PoseSpec> {
        Some(PoseSpec::BoneSpace)
    }

    fn display_name(&self) -> String {
        "⛓ Chain".into()
    }
}
