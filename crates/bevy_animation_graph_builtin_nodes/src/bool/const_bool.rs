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
pub struct ConstBool {
    pub constant: bool,
}

impl ConstBool {
    pub const OUTPUT: &'static str = "out";

    pub fn new(constant: bool) -> Self {
        Self { constant }
    }
}

impl NodeLike for ConstBool {
    fn display_name(&self) -> String {
        "Bool".into()
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        ctx.set_data_fwd(Self::OUTPUT, self.constant);
        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx.add_output_data(Self::OUTPUT, DataSpec::Bool);

        Ok(())
    }
}
