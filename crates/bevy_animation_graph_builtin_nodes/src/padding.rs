use bevy::prelude::*;
use bevy_animation_graph_core::{
    animation_graph::TimeUpdate,
    animation_node::{NodeLike, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    edge_data::DataSpec,
    errors::GraphError,
    interpolation::linear::InterpolateLinear,
};

/// This node pads the duration of an animation with a configurable period where
/// the last frame interpolates to the first
#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
pub struct PaddingNode {
    pub interpolation_period: f32,
}

impl PaddingNode {
    pub const IN_POSE: &'static str = "pose";
    pub const IN_TIME: &'static str = "time";
    pub const OUT_POSE: &'static str = "pose";

    pub fn new(interpolation_period: f32) -> Self {
        Self {
            interpolation_period,
        }
    }
}

impl NodeLike for PaddingNode {
    fn duration(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let duration = ctx
            .duration_back(Self::IN_TIME)?
            .map(|d| d + self.interpolation_period);
        ctx.set_duration_fwd(duration);
        Ok(())
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let input = ctx.time_update_fwd()?;

        ctx.set_time_update_back(Self::IN_TIME, input);
        let mut pose = ctx.data_back(Self::IN_POSE)?.into_pose()?;
        ctx.set_time(pose.timestamp);

        let Some(duration) = ctx.duration_back(Self::IN_TIME)? else {
            ctx.set_data_fwd(Self::OUT_POSE, pose);
            return Ok(());
        };

        if pose.timestamp > duration {
            // Need to get initial pose and interpolate
            let mut ctx_temp = ctx.clone().with_temp_state_key();
            ctx_temp.set_time_update_back(Self::IN_TIME, TimeUpdate::Absolute(0.));
            let start_pose = ctx_temp.data_back(Self::IN_POSE)?.into_pose()?;
            // TODO: How to clear cache? time? pose?
            // ctx.clear_temp_cache(Self::IN_POSE);
            let old_time = pose.timestamp;
            let alpha = ((pose.timestamp - duration) / self.interpolation_period).clamp(0., 1.);
            pose = pose.interpolate_linear(&start_pose, alpha);
            pose.timestamp = old_time;
        }

        ctx.set_data_fwd(Self::OUT_POSE, pose);

        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx //
            .add_input_data(Self::IN_POSE, DataSpec::Pose)
            .add_input_time(Self::IN_TIME);
        ctx //
            .add_output_data(Self::OUT_POSE, DataSpec::Pose)
            .add_output_time();

        Ok(())
    }

    fn display_name(&self) -> String {
        "Padding".into()
    }
}
