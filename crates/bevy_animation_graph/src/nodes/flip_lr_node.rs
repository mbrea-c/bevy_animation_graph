use crate::core::animation_graph::{PinMap, TimeUpdate};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::duration_data::DurationData;
use crate::core::errors::GraphError;
use crate::core::frame::{BonePoseFrame, PoseFrame, PoseFrameData, PoseSpec};
use crate::flipping::FlipXBySuffix;
use crate::prelude::config::FlipConfig;
use crate::prelude::{PassContext, SpecContext};
use crate::utils::unwrap::Unwrap;
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug)]
#[reflect(Default)]
pub struct FlipLRNode {
    pub config: FlipConfig,
}

impl Default for FlipLRNode {
    fn default() -> Self {
        Self::new(FlipConfig::default())
    }
}

impl FlipLRNode {
    pub const INPUT: &'static str = "Pose In";
    pub const OUTPUT: &'static str = "Pose Out";

    pub fn new(config: FlipConfig) -> Self {
        Self { config }
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::FlipLR(self))
    }
}

impl NodeLike for FlipLRNode {
    fn duration_pass(&self, mut ctx: PassContext) -> Result<Option<DurationData>, GraphError> {
        Ok(Some(ctx.duration_back(Self::INPUT)?))
    }

    fn pose_pass(
        &self,
        input: TimeUpdate,
        mut ctx: PassContext,
    ) -> Result<Option<PoseFrame>, GraphError> {
        let in_pose_frame = ctx.pose_back(Self::INPUT, input)?;
        let bone_frame: BonePoseFrame = in_pose_frame.data.unwrap();
        let flipped_pose_frame = bone_frame.flipped(&self.config);

        Ok(Some(PoseFrame {
            data: PoseFrameData::BoneSpace(flipped_pose_frame),
            timestamp: in_pose_frame.timestamp,
        }))
    }

    fn pose_input_spec(&self, _: SpecContext) -> PinMap<PoseSpec> {
        [(Self::INPUT.into(), PoseSpec::BoneSpace)].into()
    }

    fn pose_output_spec(&self, _: SpecContext) -> Option<PoseSpec> {
        Some(PoseSpec::BoneSpace)
    }

    fn display_name(&self) -> String {
        "🚻 Flip Left/Right".into()
    }
}
