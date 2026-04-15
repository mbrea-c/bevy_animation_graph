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
pub struct NotBool;

impl NotBool {
    pub const INPUT: &'static str = "in";
    pub const OUTPUT: &'static str = "out";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for NotBool {
    fn display_name(&self) -> String {
        "! Not".into()
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let input = ctx.data_back(Self::INPUT)?.as_bool()?;
        ctx.set_data_fwd(Self::OUTPUT, !input);
        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx.add_input_data(Self::INPUT, DataSpec::Bool)
            .add_output_data(Self::OUTPUT, DataSpec::Bool);
        Ok(())
    }
}
