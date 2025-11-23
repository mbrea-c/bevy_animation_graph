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
pub struct LerpVec3Node;

impl LerpVec3Node {
    pub const INPUT_A: &'static str = "a";
    pub const INPUT_B: &'static str = "b";
    pub const INPUT_FACTOR: &'static str = "factor";
    pub const OUTPUT: &'static str = "out";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for LerpVec3Node {
    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let a: Vec3 = ctx.data_back(Self::INPUT_A)?.as_vec3()?;
        let b: Vec3 = ctx.data_back(Self::INPUT_B)?.as_vec3()?;
        let factor: f32 = ctx.data_back(Self::INPUT_FACTOR)?.as_f32()?;

        let output = Vec3::lerp(a, b, factor);

        ctx.set_data_fwd(Self::OUTPUT, output);

        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx //
            .add_input_data(Self::INPUT_A, DataSpec::Vec3)
            .add_input_data(Self::INPUT_B, DataSpec::Vec3)
            .add_input_data(Self::INPUT_FACTOR, DataSpec::F32);
        ctx.add_output_data(Self::OUTPUT, DataSpec::Vec3);

        Ok(())
    }

    fn display_name(&self) -> String {
        "Lerp Vec3".into()
    }
}
