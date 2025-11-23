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
pub struct ConstVec3Node {
    pub constant: Vec3,
}

impl ConstVec3Node {
    pub const OUTPUT: &'static str = "out";

    pub fn new(constant: Vec3) -> Self {
        Self { constant }
    }
}

impl NodeLike for ConstVec3Node {
    fn display_name(&self) -> String {
        "Vec3".into()
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        ctx.set_data_fwd(Self::OUTPUT, self.constant);
        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx.add_output_data(Self::OUTPUT, DataSpec::Vec3);

        Ok(())
    }
}
