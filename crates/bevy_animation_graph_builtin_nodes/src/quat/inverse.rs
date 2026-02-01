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
pub struct InvertQuatNode;

impl InvertQuatNode {
    pub const INPUT: &'static str = "quat";
    pub const OUTPUT: &'static str = "inverse";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for InvertQuatNode {
    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let input: Quat = ctx.data_back(Self::INPUT)?.as_quat()?;
        let output: Quat = input.inverse();

        ctx.set_data_fwd(Self::OUTPUT, output);

        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx.add_input_data(Self::INPUT, DataSpec::Quat);
        ctx.add_output_data(Self::OUTPUT, DataSpec::Quat);

        Ok(())
    }

    fn display_name(&self) -> String {
        "Invert Quat".into()
    }
}
