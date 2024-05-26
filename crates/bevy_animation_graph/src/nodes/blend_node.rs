use crate::core::animation_graph::{PinMap, TimeUpdate};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::errors::GraphError;
use crate::core::pose::Pose;
use crate::core::prelude::DataSpec;
use crate::prelude::{InterpolateLinear, PassContext, SpecContext};
use crate::utils::unwrap::UnwrapVal;
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub struct BlendNode;

impl BlendNode {
    pub const FACTOR: &'static str = "factor";
    pub const IN_POSE_A: &'static str = "pose A";
    pub const IN_TIME_A: &'static str = "time A";
    pub const IN_POSE_B: &'static str = "pose B";
    pub const IN_TIME_B: &'static str = "time B";
    pub const OUT_POSE: &'static str = "pose";

    pub fn new() -> Self {
        Self
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::Blend(self))
    }
}

impl NodeLike for BlendNode {
    fn duration(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let duration_1 = ctx.duration_back(Self::IN_TIME_A)?;
        let duration_2 = ctx.duration_back(Self::IN_TIME_B)?;

        let out_duration = match (duration_1, duration_2) {
            (Some(duration_1), Some(duration_2)) => Some(duration_1.max(duration_2)),
            (Some(duration_1), None) => Some(duration_1),
            (None, Some(duration_2)) => Some(duration_2),
            (None, None) => None,
        };

        ctx.set_duration_fwd(out_duration);
        Ok(())
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let input = ctx.time_update_fwd()?;
        ctx.set_time_update_back(Self::IN_TIME_A, input);
        let in_frame_1: Pose = ctx.data_back(Self::IN_POSE_A)?.val();
        ctx.set_time_update_back(Self::IN_TIME_B, TimeUpdate::Absolute(in_frame_1.timestamp));
        let in_frame_2: Pose = ctx.data_back(Self::IN_POSE_B)?.val();

        let alpha = ctx.data_back(Self::FACTOR)?.unwrap_f32();
        let out = in_frame_1.interpolate_linear(&in_frame_2, alpha);

        ctx.set_time(out.timestamp);
        ctx.set_data_fwd(Self::OUT_POSE, out);

        Ok(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [
            (Self::FACTOR.into(), DataSpec::F32),
            (Self::IN_POSE_A.into(), DataSpec::Pose),
            (Self::IN_POSE_B.into(), DataSpec::Pose),
        ]
        .into()
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUT_POSE.into(), DataSpec::Pose.into())].into()
    }

    fn time_input_spec(&self, _: SpecContext) -> PinMap<()> {
        [(Self::IN_TIME_A.into(), ()), (Self::IN_TIME_B.into(), ())].into()
    }

    fn time_output_spec(&self, _: SpecContext) -> Option<()> {
        Some(())
    }

    fn display_name(&self) -> String {
        "∑ Blend".into()
    }
}
