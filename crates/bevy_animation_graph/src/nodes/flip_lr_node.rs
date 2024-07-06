use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::errors::GraphError;
use crate::core::pose::Pose;
use crate::core::prelude::DataSpec;
use crate::flipping::FlipXBySuffix;
use crate::prelude::config::FlipConfig;
use crate::prelude::{BoneDebugGizmos, PassContext, SpecContext};
use crate::utils::unwrap::UnwrapVal;
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
    pub const IN_POSE: &'static str = "pose";
    pub const IN_TIME: &'static str = "time";
    pub const OUT_POSE: &'static str = "pose";

    pub fn new(config: FlipConfig) -> Self {
        Self { config }
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::FlipLR(self))
    }
}

impl NodeLike for FlipLRNode {
    fn duration(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let duration = ctx.duration_back(Self::IN_TIME)?;
        ctx.set_duration_fwd(duration);
        Ok(())
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let input = ctx.time_update_fwd()?;
        ctx.set_time_update_back(Self::IN_TIME, input);
        let in_pose: Pose = ctx.data_back(Self::IN_POSE)?.val();
        ctx.set_time(in_pose.timestamp);
        ctx.pose_bone_gizmos(LinearRgba::RED, &in_pose);
        let flipped_pose = in_pose.flipped(&self.config);
        ctx.pose_bone_gizmos(LinearRgba::BLUE, &flipped_pose);
        ctx.set_data_fwd(Self::OUT_POSE, flipped_pose);

        Ok(())
    }

    fn data_input_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::IN_POSE.into(), DataSpec::Pose)].into()
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUT_POSE.into(), DataSpec::Pose)].into()
    }

    fn time_input_spec(&self, _: SpecContext) -> PinMap<()> {
        [(Self::IN_TIME.into(), ())].into()
    }

    fn time_output_spec(&self, _: SpecContext) -> Option<()> {
        Some(())
    }

    fn display_name(&self) -> String {
        "🚻 Flip Left/Right".into()
    }
}
