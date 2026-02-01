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
pub struct DecomposeVec3Node;

impl DecomposeVec3Node {
    pub const INPUT: &'static str = "vec";
    pub const OUTPUT_X: &'static str = "x";
    pub const OUTPUT_Y: &'static str = "y";
    pub const OUTPUT_Z: &'static str = "z";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for DecomposeVec3Node {
    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let Vec3 { x, y, z } = ctx.data_back(Self::INPUT)?.as_vec3()?;

        ctx.set_data_fwd(Self::OUTPUT_X, x);
        ctx.set_data_fwd(Self::OUTPUT_Y, y);
        ctx.set_data_fwd(Self::OUTPUT_Z, z);

        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx.add_input_data(Self::INPUT, DataSpec::Vec3);
        ctx.add_output_data(Self::OUTPUT_X, DataSpec::F32)
            .add_output_data(Self::OUTPUT_Y, DataSpec::F32)
            .add_output_data(Self::OUTPUT_Z, DataSpec::F32);
        Ok(())
    }

    fn display_name(&self) -> String {
        "Decompose Vec3".into()
    }
}
