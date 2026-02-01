use bevy::prelude::*;
use bevy_animation_graph_core::{
    animation_graph::TimeUpdate,
    animation_node::{NodeLike, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    edge_data::DataSpec,
    errors::GraphError,
    interpolation::linear::InterpolateLinear,
};

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
pub struct ChainNode {
    /// Time in-between animations where the output should interpolate between the last pose of the
    /// first animation and the first pose of the second
    pub interpolation_period: f32,
}

impl ChainNode {
    pub const IN_POSE_A: &'static str = "pose_a";
    pub const IN_TIME_A: &'static str = "time_a";
    pub const IN_POSE_B: &'static str = "pose_b";
    pub const IN_TIME_B: &'static str = "time_b";
    pub const OUT_POSE: &'static str = "pose";

    pub fn new(interpolation_period: f32) -> Self {
        Self {
            interpolation_period,
        }
    }
}

impl NodeLike for ChainNode {
    fn duration(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let source_duration_1 = ctx.duration_back(Self::IN_TIME_A)?;
        let source_duration_2 = ctx.duration_back(Self::IN_TIME_B)?;

        let out_duration = match (source_duration_1, source_duration_2) {
            (Some(duration_1), Some(duration_2)) => {
                Some(duration_1 + duration_2 + self.interpolation_period)
            }
            (Some(_), None) => None,
            (None, Some(_)) => None,
            (None, None) => None,
        };

        ctx.set_duration_fwd(out_duration);
        Ok(())
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let input = ctx.time_update_fwd()?;
        let duration_1 = ctx.duration_back(Self::IN_TIME_A)?;
        let Some(duration_1) = duration_1 else {
            // First input is infinite, forward time update without change
            ctx.set_time_update_back(Self::IN_TIME_A, input);
            let pose_a = ctx.data_back(Self::IN_POSE_A)?.into_pose()?;
            ctx.set_time(pose_a.timestamp);
            ctx.set_data_fwd(Self::OUT_POSE, pose_a);
            return Ok(());
        };
        ctx.set_time_update_back(Self::IN_TIME_A, input);
        let pose_a = ctx.data_back(Self::IN_POSE_A)?.into_pose()?;
        let curr_time = pose_a.timestamp;
        ctx.set_time(curr_time);

        if curr_time < duration_1 {
            ctx.set_data_fwd(Self::OUT_POSE, pose_a);
        } else if curr_time - duration_1 - self.interpolation_period >= 0. {
            ctx.set_time_update_back(
                Self::IN_TIME_B,
                TimeUpdate::Absolute(curr_time - duration_1 - self.interpolation_period),
            );
            let mut pose_b = ctx.data_back(Self::IN_POSE_B)?.into_pose()?;
            pose_b.timestamp = curr_time;
            ctx.set_data_fwd(Self::OUT_POSE, pose_b);
        } else {
            ctx.set_time_update_back(Self::IN_TIME_B, TimeUpdate::Absolute(0.0));
            let pose_2 = ctx.data_back(Self::IN_POSE_B)?.into_pose()?;
            let mut out_pose = pose_a.interpolate_linear(
                &pose_2,
                (curr_time - duration_1) / self.interpolation_period,
            );
            out_pose.timestamp = curr_time;
            ctx.set_data_fwd(Self::OUT_POSE, out_pose);
        }

        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx //
            .add_input_data(Self::IN_POSE_A, DataSpec::Pose)
            .add_input_time(Self::IN_TIME_A)
            .add_input_data(Self::IN_POSE_B, DataSpec::Pose)
            .add_input_time(Self::IN_TIME_B);
        ctx //
            .add_output_data(Self::OUT_POSE, DataSpec::Pose)
            .add_output_time();
        Ok(())
    }

    fn display_name(&self) -> String {
        "⛓ Chain".into()
    }
}
