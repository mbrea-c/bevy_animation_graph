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
pub struct BuildVec3Node;

impl BuildVec3Node {
    pub const INPUT_X: &'static str = "x";
    pub const INPUT_Y: &'static str = "y";
    pub const INPUT_Z: &'static str = "z";
    pub const OUTPUT: &'static str = "vec";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for BuildVec3Node {
    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let x = ctx.data_back(Self::INPUT_X)?.as_f32()?;
        let y = ctx.data_back(Self::INPUT_Y)?.as_f32()?;
        let z = ctx.data_back(Self::INPUT_Z)?.as_f32()?;

        ctx.set_data_fwd(Self::OUTPUT, Vec3::new(x, y, z));

        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx.add_input_data(Self::INPUT_X, DataSpec::F32)
            .add_input_data(Self::INPUT_Y, DataSpec::F32)
            .add_input_data(Self::INPUT_Z, DataSpec::F32);
        ctx.add_output_data(Self::OUTPUT, DataSpec::Vec3);

        Ok(())
    }

    fn display_name(&self) -> String {
        "Build Vec3".into()
    }
}
