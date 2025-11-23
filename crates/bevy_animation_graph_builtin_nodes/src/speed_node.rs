use bevy::reflect::{Reflect, std_traits::ReflectDefault};
use bevy_animation_graph_core::{
    animation_graph::TimeUpdate,
    animation_node::{NodeLike, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    edge_data::DataSpec,
    errors::GraphError,
};

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
pub struct SpeedNode;

impl SpeedNode {
    pub const IN_POSE: &'static str = "pose";
    pub const IN_TIME: &'static str = "time";
    pub const OUT_POSE: &'static str = "pose";
    pub const SPEED: &'static str = "speed";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for SpeedNode {
    fn duration(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let speed = ctx.data_back(Self::SPEED)?.as_f32()?;
        let out_duration = if speed == 0. {
            None
        } else {
            let duration = ctx.duration_back(Self::IN_TIME)?;
            duration.as_ref().map(|duration| duration / speed)
        };
        ctx.set_duration_fwd(out_duration);
        Ok(())
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let speed = ctx.data_back(Self::SPEED)?.as_f32()?;
        let input = ctx.time_update_fwd()?;

        let fw_upd = match input {
            TimeUpdate::Delta(dt) => TimeUpdate::Delta(dt * speed),
            // TODO: add warnings if input is not delta
            other => other,
        };

        ctx.set_time_update_back(Self::IN_TIME, fw_upd);
        let mut in_pose = ctx.data_back(Self::IN_POSE)?.into_pose()?;

        if speed != 0. {
            in_pose.timestamp /= speed.abs();
        }

        ctx.set_time(in_pose.timestamp);
        ctx.set_data_fwd(Self::OUT_POSE, in_pose);

        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx //
            .add_input_data(Self::SPEED, DataSpec::F32)
            .add_input_data(Self::IN_POSE, DataSpec::Pose)
            .add_input_time(Self::IN_TIME);
        ctx //
            .add_output_data(Self::OUT_POSE, DataSpec::Pose)
            .add_output_time();

        Ok(())
    }

    fn display_name(&self) -> String {
        "⌚ Speed".into()
    }
}
