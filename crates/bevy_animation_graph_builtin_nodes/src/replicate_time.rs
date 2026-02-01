use bevy::prelude::*;
use bevy_animation_graph_core::{
    animation_node::{NodeLike, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    errors::GraphError,
};

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
pub struct ReplicateTimeNode {
    pub extra_count: u32,
}

impl ReplicateTimeNode {
    pub const MASTER_TIME: &'static str = "master";
    pub const SLAVE_PREFIX: &'static str = "slave";
}

impl NodeLike for ReplicateTimeNode {
    fn duration(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        ctx.set_duration_fwd(ctx.duration_back(Self::MASTER_TIME)?);
        Ok(())
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let input = ctx.time_update_fwd()?;

        ctx.set_time_update_back(Self::MASTER_TIME, input.clone());

        for i in 0..self.extra_count {
            ctx.set_time_update_back(format!("{}_{}", Self::SLAVE_PREFIX, i), input.clone());
        }

        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx.add_input_time(Self::MASTER_TIME);
        for i in 0..self.extra_count {
            ctx.add_input_time(format!("{}_{}", Self::SLAVE_PREFIX, i));
        }
        ctx.add_output_time();

        Ok(())
    }

    fn display_name(&self) -> String {
        "⏳ Replicate Time".into()
    }
}
