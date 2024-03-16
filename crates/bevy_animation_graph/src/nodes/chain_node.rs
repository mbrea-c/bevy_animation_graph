use crate::core::animation_graph::{PinMap, TimeUpdate};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::duration_data::DurationData;
use crate::core::errors::GraphError;
use crate::core::pose::{Pose, PoseSpec};
use crate::prelude::{InterpolateLinear, PassContext, SpecContext};
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub struct ChainNode {
    /// Time in-between animations where the output should interpolate between the last pose of the
    /// first animation and the first pose of the second
    pub interpolation_period: f32,
}

impl ChainNode {
    pub const INPUT_1: &'static str = "Pose In 1";
    pub const INPUT_2: &'static str = "Pose In 2";
    pub const OUTPUT: &'static str = "Pose Out";

    pub fn new(interpolation_period: f32) -> Self {
        Self {
            interpolation_period,
        }
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::Chain(self))
    }
}

impl NodeLike for ChainNode {
    fn duration_pass(&self, mut ctx: PassContext) -> Result<Option<DurationData>, GraphError> {
        let source_duration_1 = ctx.duration_back(Self::INPUT_1)?;
        let source_duration_2 = ctx.duration_back(Self::INPUT_2)?;

        let out_duration = match (source_duration_1, source_duration_2) {
            (Some(duration_1), Some(duration_2)) => Some(duration_1 + duration_2),
            (Some(_), None) => None,
            (None, Some(_)) => None,
            (None, None) => None,
        };

        Ok(Some(out_duration))
    }

    fn pose_pass(
        &self,
        input: TimeUpdate,
        mut ctx: PassContext,
    ) -> Result<Option<Pose>, GraphError> {
        let duration_1 = ctx.duration_back(Self::INPUT_1)?;
        let Some(duration_1) = duration_1 else {
            // First input is infinite, forward time update without change
            return Ok(Some(ctx.pose_back(Self::INPUT_1, input)?));
        };
        let pose_1 = ctx.pose_back(Self::INPUT_1, input)?;
        let curr_time = pose_1.timestamp;

        if curr_time < duration_1 {
            Ok(Some(pose_1))
        } else if curr_time - duration_1 - self.interpolation_period >= 0. {
            let mut pose_2 = ctx.pose_back(
                Self::INPUT_2,
                TimeUpdate::Absolute(curr_time - duration_1 - self.interpolation_period),
            )?;
            pose_2.timestamp = curr_time;
            Ok(Some(pose_2))
        } else {
            let pose_2 = ctx.pose_back(Self::INPUT_2, TimeUpdate::Absolute(0.0))?;
            let mut out_pose = pose_1.interpolate_linear(
                &pose_2,
                (curr_time - duration_1) / self.interpolation_period,
            );
            out_pose.timestamp = curr_time;
            Ok(Some(out_pose))
        }
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
