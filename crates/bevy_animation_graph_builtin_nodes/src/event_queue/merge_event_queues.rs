use bevy::prelude::*;
use bevy_animation_graph_core::{
    animation_node::{NodeLike, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    edge_data::DataSpec,
    errors::GraphError,
};

/// Merges two event queues into one.
#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
pub struct MergeEventQueues;

impl MergeEventQueues {
    pub const IN_A: &'static str = "in_a";
    pub const IN_B: &'static str = "in_b";
    pub const OUTPUT: &'static str = "out";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for MergeEventQueues {
    fn display_name(&self) -> String {
        "Merge Events".into()
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let a = ctx.data_back(Self::IN_A)?.into_event_queue()?;
        let b = ctx.data_back(Self::IN_B)?.into_event_queue()?;
        ctx.set_data_fwd(Self::OUTPUT, a.concat(b));
        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx.add_input_data(Self::IN_A, DataSpec::EventQueue)
            .add_input_data(Self::IN_B, DataSpec::EventQueue)
            .add_output_data(Self::OUTPUT, DataSpec::EventQueue);
        Ok(())
    }
}
