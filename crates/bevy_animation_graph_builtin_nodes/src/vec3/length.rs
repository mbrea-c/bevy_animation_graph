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
pub struct LengthVec3Node;

impl LengthVec3Node {
    pub const INPUT: &'static str = "in";
    pub const OUTPUT: &'static str = "out";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for LengthVec3Node {
    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let input: Vec3 = ctx.data_back(Self::INPUT)?.as_vec3()?;
        let output = input.length();

        ctx.set_data_fwd(Self::OUTPUT, output);

        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx.add_input_data(Self::INPUT, DataSpec::Vec3);
        ctx.add_output_data(Self::OUTPUT, DataSpec::F32);

        Ok(())
    }

    fn display_name(&self) -> String {
        "Length Vec3".into()
    }
}
