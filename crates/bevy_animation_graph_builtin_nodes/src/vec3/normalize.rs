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
pub struct NormalizeVec3Node;

impl NormalizeVec3Node {
    pub const INPUT: &'static str = "in";
    pub const OUTPUT: &'static str = "out";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for NormalizeVec3Node {
    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let input: Vec3 = ctx.data_back(Self::INPUT)?.as_vec3()?;
        let output = input.normalize_or_zero();

        ctx.set_data_fwd(Self::OUTPUT, output);

        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx.add_input_data(Self::INPUT, DataSpec::Vec3);
        ctx.add_output_data(Self::OUTPUT, DataSpec::Vec3);
        Ok(())
    }

    fn display_name(&self) -> String {
        "Normalize Vec3".into()
    }
}
