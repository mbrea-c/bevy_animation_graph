use bevy::prelude::*;
use bevy_animation_graph_core::{
    animation_node::{NodeLike, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    edge_data::DataSpec,
    errors::GraphError,
};

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
pub struct AndBool;

impl AndBool {
    pub const INPUT_A: &'static str = "in_a";
    pub const INPUT_B: &'static str = "in_b";
    pub const OUTPUT: &'static str = "out";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for AndBool {
    fn display_name(&self) -> String {
        "&& And".into()
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let a = ctx.data_back(Self::INPUT_A)?.as_bool()?;
        let b = ctx.data_back(Self::INPUT_B)?.as_bool()?;
        ctx.set_data_fwd(Self::OUTPUT, a && b);
        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx.add_input_data(Self::INPUT_A, DataSpec::Bool)
            .add_input_data(Self::INPUT_B, DataSpec::Bool)
            .add_output_data(Self::OUTPUT, DataSpec::Bool);
        Ok(())
    }
}
